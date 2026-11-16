//! Lector minimo de "Compound File Binary" (OLE2), el contenedor de los archivos .mpp.
//!
//! Solo lectura, sin dependencias. Implementa lo necesario de [MS-CFB]: cabecera, DIFAT, FAT,
//! directorio, mini-FAT y mini-stream.

const SIGNATURE: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
const NOSTREAM: u32 = 0xFFFF_FFFF;
const MAX_REG_SECT: u32 = 0xFFFF_FFFA;

pub struct Entry {
    pub name: String,
    /// 1 = carpeta (storage), 2 = stream, 5 = raiz.
    pub kind: u8,
    left: u32,
    right: u32,
    child: u32,
    start: u32,
    size: u64,
}

pub struct Cfb<'a> {
    data: &'a [u8],
    sector_shift: u32,
    mini_shift: u32,
    mini_cutoff: u64,
    fat: Vec<u32>,
    minifat: Vec<u32>,
    ministream: Vec<u8>,
    pub entries: Vec<Entry>,
}

fn u16_at(d: &[u8], o: usize) -> u16 {
    match d.get(o..o + 2) {
        Some(b) => u16::from_le_bytes([b[0], b[1]]),
        None => 0,
    }
}

fn u32_at(d: &[u8], o: usize) -> u32 {
    match d.get(o..o + 4) {
        Some(b) => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
        None => 0,
    }
}

impl<'a> Cfb<'a> {
    pub fn open(data: &'a [u8]) -> Result<Cfb<'a>, String> {
        if data.len() < 512 || data[..8] != SIGNATURE {
            return Err("El archivo no es un .mpp valido (no es un contenedor OLE2).".into());
        }
        let sector_shift = u16_at(data, 30) as u32;
        let mini_shift = u16_at(data, 32) as u32;
        if !(sector_shift == 9 || sector_shift == 12) || mini_shift == 0 || mini_shift >= sector_shift {
            return Err("Cabecera OLE2 no soportada.".into());
        }
        let mut cfb = Cfb {
            data,
            sector_shift,
            mini_shift,
            mini_cutoff: u32_at(data, 56) as u64,
            fat: Vec::new(),
            minifat: Vec::new(),
            ministream: Vec::new(),
            entries: Vec::new(),
        };
        let ss = 1usize << sector_shift;
        let per_sector = ss / 4;

        // DIFAT: 109 entradas en la cabecera + cadena de sectores DIFAT.
        let mut fat_sectors: Vec<u32> = (0..109).map(|i| u32_at(data, 76 + i * 4)).filter(|&s| s <= MAX_REG_SECT).collect();
        let mut difat = u32_at(data, 68);
        let mut guard = 0usize;
        while difat <= MAX_REG_SECT && guard < cfb.max_sectors() {
            let sec = match cfb.sector(difat) {
                Some(s) => s,
                None => break,
            };
            for i in 0..per_sector - 1 {
                let s = u32_at(sec, i * 4);
                if s <= MAX_REG_SECT {
                    fat_sectors.push(s);
                }
            }
            difat = u32_at(sec, (per_sector - 1) * 4);
            guard += 1;
        }
        fat_sectors.truncate(cfb.max_sectors());

        let mut fat = Vec::with_capacity(fat_sectors.len() * per_sector);
        for s in fat_sectors {
            if let Some(sec) = cfb.sector(s) {
                fat.extend((0..per_sector).map(|i| u32_at(sec, i * 4)));
            }
        }
        cfb.fat = fat;

        // Directorio.
        let dir = cfb.read_chain(u32_at(data, 48), u64::MAX);
        for chunk in dir.chunks_exact(128) {
            let name_len = (u16_at(chunk, 64) as usize).min(64);
            let chars = name_len.saturating_sub(2) / 2;
            let units: Vec<u16> = (0..chars).map(|i| u16_at(chunk, i * 2)).collect();
            let size = u32_at(chunk, 120) as u64 | if sector_shift == 12 { (u32_at(chunk, 124) as u64) << 32 } else { 0 };
            cfb.entries.push(Entry {
                name: String::from_utf16_lossy(&units),
                kind: chunk[66],
                left: u32_at(chunk, 68),
                right: u32_at(chunk, 72),
                child: u32_at(chunk, 76),
                start: u32_at(chunk, 116),
                size,
            });
        }
        if cfb.entries.is_empty() {
            return Err("Contenedor OLE2 sin directorio.".into());
        }

        // Mini-FAT y mini-stream (colgado de la entrada raiz).
        let mf = cfb.read_chain(u32_at(data, 60), u64::MAX);
        cfb.minifat = mf.chunks_exact(4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
        cfb.ministream = cfb.read_chain(cfb.entries[0].start, cfb.entries[0].size);
        Ok(cfb)
    }

    /// Sectores que caben en el archivo. Ninguna cadena valida es mas larga: usarlo como tope
    /// impide que un ciclo en un archivo malicioso haga reservar gigabytes.
    fn max_sectors(&self) -> usize {
        self.data.len() >> self.sector_shift
    }

    fn sector(&self, n: u32) -> Option<&'a [u8]> {
        let ss = 1usize << self.sector_shift;
        let start = (n as usize).checked_add(1)?.checked_mul(ss)?;
        self.data.get(start..start + ss).or_else(|| {
            // Ultimo sector truncado: se acepta lo que haya.
            if start < self.data.len() {
                self.data.get(start..)
            } else {
                None
            }
        })
    }

    fn read_chain(&self, start: u32, size: u64) -> Vec<u8> {
        let mut out = Vec::new();
        let mut s = start;
        let mut guard = 0usize;
        while s <= MAX_REG_SECT && guard < self.max_sectors() && (out.len() as u64) < size {
            match self.sector(s) {
                Some(sec) => out.extend_from_slice(sec),
                None => break,
            }
            s = self.fat.get(s as usize).copied().unwrap_or(NOSTREAM);
            guard += 1;
        }
        if (out.len() as u64) > size {
            out.truncate(size as usize);
        }
        out
    }

    fn read_mini_chain(&self, start: u32, size: u64) -> Vec<u8> {
        let ms = 1usize << self.mini_shift;
        let mut out = Vec::new();
        let mut s = start;
        let mut guard = 0usize;
        while s <= MAX_REG_SECT && guard < self.ministream.len() >> self.mini_shift && (out.len() as u64) < size {
            let off = s as usize * ms;
            match self.ministream.get(off..off + ms) {
                Some(b) => out.extend_from_slice(b),
                None => break,
            }
            s = self.minifat.get(s as usize).copied().unwrap_or(NOSTREAM);
            guard += 1;
        }
        if (out.len() as u64) > size {
            out.truncate(size as usize);
        }
        out
    }

    /// Hijos directos de una carpeta.
    pub fn children(&self, parent: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut stack = Vec::new();
        if let Some(p) = self.entries.get(parent) {
            stack.push(p.child);
        }
        let mut guard = 0usize;
        while let Some(n) = stack.pop() {
            guard += 1;
            if n == NOSTREAM || guard > self.entries.len() * 2 + 2 {
                continue;
            }
            let idx = n as usize;
            if let Some(e) = self.entries.get(idx) {
                if out.contains(&idx) {
                    continue;
                }
                out.push(idx);
                stack.push(e.left);
                stack.push(e.right);
            }
        }
        out
    }

    pub fn child(&self, parent: usize, name: &str) -> Option<usize> {
        self.children(parent).into_iter().find(|&i| self.entries[i].name.eq_ignore_ascii_case(name))
    }

    /// Busca una entrada por ruta desde la raiz, p. ej. `["   114", "TBkndTask", "VarMeta"]`.
    pub fn path(&self, parts: &[&str]) -> Option<usize> {
        let mut cur = 0usize;
        for p in parts {
            cur = self.child(cur, p)?;
        }
        Some(cur)
    }

    pub fn read(&self, idx: usize) -> Vec<u8> {
        match self.entries.get(idx) {
            Some(e) if e.kind == 2 => {
                if e.size < self.mini_cutoff {
                    self.read_mini_chain(e.start, e.size)
                } else {
                    self.read_chain(e.start, e.size)
                }
            }
            _ => Vec::new(),
        }
    }

    pub fn read_path(&self, parts: &[&str]) -> Option<Vec<u8>> {
        self.path(parts).map(|i| self.read(i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ciclos_no_amplifican() {
        // Cabecera valida y un unico sector de ceros: la cadena DIFAT, la FAT, el directorio y
        // la mini-FAT apuntan todos al sector 0, es decir, a si mismos.
        let mut d = vec![0u8; 1024];
        d[..8].copy_from_slice(&SIGNATURE);
        d[30] = 9;
        d[32] = 6;
        assert!(crate::read_mpp(&d).is_err());
        let cfb = Cfb::open(&d).unwrap();
        let fat = cfb.fat.len();
        let chain = cfb.read_chain(0, u64::MAX).len();
        let mini = Cfb { ministream: vec![0; 128], minifat: vec![0; 1000], ..cfb };
        let mini_chain = mini.read_mini_chain(0, u64::MAX).len();
        assert!(fat <= d.len() / 4 && chain <= d.len() && mini_chain <= 128, "fat={} cadena={} mini={}", fat, chain, mini_chain);
    }
}
