//! Descompresor DEFLATE (RFC 1951) con envoltura zlib opcional (RFC 1950).
//!
//! Lo usan los .docx (son un ZIP) y los PDF (filtro FlateDecode). Es tolerante: si los datos
//! estan truncados o danados devuelve lo que alcanzo a descomprimir y lo indica en `ok`.
//! `zlib_strict`, en cambio, rechaza todo lo que rechazaria un lector estricto como el de PNG.

pub struct Inflated {
    pub data: Vec<u8>,
    pub ok: bool,
}

/// Tope de salida, para que un archivo malicioso no agote la memoria.
pub const MAX_OUTPUT: usize = 256 << 20;

struct Bits<'a> {
    d: &'a [u8],
    pos: usize,
    acc: u32,
    n: u32,
    /// Rechaza lo que zlib rechaza aunque se pueda leer: largo de bloque sin su complemento y
    /// codigos Huffman incompletos o sobrepasados.
    strict: bool,
}

impl Bits<'_> {
    #[inline]
    fn bits(&mut self, count: u32) -> Option<u32> {
        while self.n < count {
            let b = *self.d.get(self.pos)?;
            self.pos += 1;
            self.acc |= (b as u32) << self.n;
            self.n += 8;
        }
        let v = self.acc & ((1u32 << count) - 1);
        self.acc >>= count;
        self.n -= count;
        Some(v)
    }

    fn align(&mut self) {
        self.acc = 0;
        self.n = 0;
    }
}

/// Tabla de codigos Huffman canonicos: cantidad de codigos por longitud y simbolos en orden.
struct Huffman {
    count: [u16; 16],
    symbol: Vec<u16>,
}

impl Huffman {
    fn new(lengths: &[u8]) -> Huffman {
        let mut count = [0u16; 16];
        for &l in lengths {
            count[(l & 15) as usize] += 1;
        }
        count[0] = 0;
        let mut offs = [0u16; 16];
        for i in 1..16 {
            offs[i] = offs[i - 1] + count[i - 1];
        }
        let mut symbol = vec![0u16; lengths.len()];
        for (sym, &l) in lengths.iter().enumerate() {
            if l != 0 {
                let o = &mut offs[(l & 15) as usize];
                if (*o as usize) < symbol.len() {
                    symbol[*o as usize] = sym as u16;
                }
                *o += 1;
            }
        }
        Huffman { count, symbol }
    }

    #[inline]
    fn decode(&self, b: &mut Bits) -> Option<u16> {
        let mut code = 0i32;
        let mut first = 0i32;
        let mut index = 0i32;
        for len in 1..16 {
            code |= b.bits(1)? as i32;
            let count = self.count[len] as i32;
            if code - count < first {
                return self.symbol.get((index + (code - first)) as usize).copied();
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }
        None
    }
}

const LEN_BASE: [u16; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
const LEN_EXTRA: [u8; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];
const CLEN_ORDER: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

/// Las longitudes forman un codigo Huffman que zlib acepta: ni sobrepasado (mas codigos de los
/// que caben) ni incompleto. Un codigo incompleto solo se admite con un unico simbolo de largo 1,
/// y nunca en el codigo de las longitudes (`lengths_code`). Sin ningun simbolo tambien se acepta:
/// falla al usarlo, como en zlib.
fn complete(lengths: &[u8], lengths_code: bool) -> bool {
    let mut count = [0i32; 16];
    for &l in lengths {
        count[(l & 15) as usize] += 1;
    }
    let max = (1..16).rev().find(|&l| count[l] > 0).unwrap_or(0);
    if max == 0 {
        return true;
    }
    let mut left = 1i32;
    for c in &count[1..] {
        left = (left << 1) - c;
        if left < 0 {
            return false;
        }
    }
    left == 0 || (!lengths_code && max == 1)
}

fn block(b: &mut Bits, out: &mut Vec<u8>, lit: &Huffman, dist: &Huffman) -> Option<()> {
    loop {
        let sym = lit.decode(b)? as usize;
        if sym < 256 {
            out.push(sym as u8);
        } else if sym == 256 {
            return Some(());
        } else {
            let i = sym - 257;
            if i >= 29 {
                return None;
            }
            let len = LEN_BASE[i] as usize + b.bits(LEN_EXTRA[i] as u32)? as usize;
            let d = dist.decode(b)? as usize;
            if d >= 30 {
                return None;
            }
            let distance = DIST_BASE[d] as usize + b.bits(DIST_EXTRA[d] as u32)? as usize;
            if distance > out.len() || out.len() + len > MAX_OUTPUT {
                return None;
            }
            let start = out.len() - distance;
            for k in 0..len {
                let v = out[start + k];
                out.push(v);
            }
        }
        if out.len() > MAX_OUTPUT {
            return None;
        }
    }
}

fn run(b: &mut Bits, out: &mut Vec<u8>) -> Option<()> {
    loop {
        let last = b.bits(1)?;
        match b.bits(2)? {
            0 => {
                b.align();
                let len = *b.d.get(b.pos)? as usize | (*b.d.get(b.pos + 1)? as usize) << 8;
                if b.strict {
                    let nlen = *b.d.get(b.pos + 2)? as usize | (*b.d.get(b.pos + 3)? as usize) << 8;
                    if nlen != !len & 0xFFFF {
                        return None;
                    }
                }
                b.pos += 4;
                let chunk = b.d.get(b.pos..b.pos + len)?;
                if out.len() + len > MAX_OUTPUT {
                    return None;
                }
                out.extend_from_slice(chunk);
                b.pos += len;
            }
            1 => {
                let mut l = [8u8; 288];
                l[144..256].fill(9);
                l[256..280].fill(7);
                block(b, out, &Huffman::new(&l), &Huffman::new(&[5u8; 30]))?;
            }
            2 => {
                let nlit = b.bits(5)? as usize + 257;
                let ndist = b.bits(5)? as usize + 1;
                let nclen = b.bits(4)? as usize + 4;
                if nlit > 286 || ndist > 30 {
                    return None;
                }
                let mut cl = [0u8; 19];
                for &o in CLEN_ORDER.iter().take(nclen) {
                    cl[o] = b.bits(3)? as u8;
                }
                if b.strict && !complete(&cl, true) {
                    return None;
                }
                let clh = Huffman::new(&cl);
                let mut lengths = vec![0u8; nlit + ndist];
                let mut i = 0usize;
                while i < nlit + ndist {
                    let sym = clh.decode(b)?;
                    let (value, repeat) = match sym {
                        0..=15 => (sym as u8, 1),
                        16 => {
                            if i == 0 {
                                return None;
                            }
                            (lengths[i - 1], 3 + b.bits(2)? as usize)
                        }
                        17 => (0, 3 + b.bits(3)? as usize),
                        _ => (0, 11 + b.bits(7)? as usize),
                    };
                    if i + repeat > nlit + ndist {
                        return None;
                    }
                    lengths[i..i + repeat].fill(value);
                    i += repeat;
                }
                // Sin codigo para el fin de bloque, zlib no lo lee.
                if b.strict && (lengths[256] == 0 || !complete(&lengths[..nlit], false) || !complete(&lengths[nlit..], false)) {
                    return None;
                }
                block(b, out, &Huffman::new(&lengths[..nlit]), &Huffman::new(&lengths[nlit..]))?;
            }
            _ => return None,
        }
        if last == 1 {
            return Some(());
        }
    }
}

/// Descomprime un flujo DEFLATE "crudo" (el de las entradas de un ZIP).
pub fn inflate(d: &[u8]) -> Inflated {
    let mut out = Vec::with_capacity(d.len().saturating_mul(3).min(MAX_OUTPUT));
    let mut b = Bits { d, pos: 0, acc: 0, n: 0, strict: false };
    let ok = run(&mut b, &mut out).is_some();
    Inflated { data: out, ok }
}

/// Descomprime un flujo zlib solo si un lector estricto (el de un PNG) lo aceptaria entero:
/// cabecera valida, ventana de hasta 32 KB y sin diccionario previo; bloques sin compresion con su
/// largo complementario (NLEN) y codigos Huffman completos; el flujo DEFLATE terminando justo
/// antes de la suma Adler-32, y esa suma correcta. `None` si no.
pub fn zlib_strict(d: &[u8]) -> Option<Vec<u8>> {
    if d.len() < 6 || d[0] & 0x0F != 8 || d[0] >> 4 > 7 || ((d[0] as u32) << 8 | d[1] as u32) % 31 != 0 || d[1] & 0x20 != 0 {
        return None;
    }
    // Sin los 4 bytes de la suma: si el flujo DEFLATE los necesitara, no termina donde debe.
    let body = &d[2..d.len() - 4];
    let mut out = Vec::with_capacity(body.len().saturating_mul(3).min(MAX_OUTPUT));
    let mut b = Bits { d: body, pos: 0, acc: 0, n: 0, strict: true };
    run(&mut b, &mut out)?;
    let adler = u32::from_be_bytes([d[d.len() - 4], d[d.len() - 3], d[d.len() - 2], d[d.len() - 1]]);
    (b.pos == body.len() && crate::png::adler32(&out) == adler).then_some(out)
}

/// Descomprime un flujo zlib (el del filtro FlateDecode de PDF). Si no trae cabecera zlib
/// valida, lo intenta como DEFLATE crudo.
pub fn zlib(d: &[u8]) -> Inflated {
    if d.len() >= 2 && d[0] & 0x0F == 8 && ((d[0] as u32) << 8 | d[1] as u32) % 31 == 0 {
        let skip = if d[1] & 0x20 != 0 { 6 } else { 2 };
        let r = inflate(d.get(skip..).unwrap_or(&[]));
        if r.ok || !r.data.is_empty() {
            return r;
        }
    }
    inflate(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bloque_almacenado() {
        // 1 bloque final sin compresion con "hola".
        let d = [0x01, 0x04, 0x00, 0xFB, 0xFF, b'h', b'o', b'l', b'a'];
        let r = inflate(&d);
        assert!(r.ok);
        assert_eq!(r.data, b"hola");
    }

    #[test]
    fn huffman_fijo_y_zlib() {
        // zlib.compress(b"hola hola hola hola hola", 9)
        let d = [0x78, 0xDA, 0xCB, 0xC8, 0xCF, 0x49, 0x54, 0xC8, 0xC0, 0x24, 0x00, 0x6D, 0x1E, 0x08, 0xB5];
        let r = zlib(&d);
        assert!(r.ok);
        assert_eq!(r.data, b"hola hola hola hola hola");
    }

    #[test]
    fn huffman_dinamico() {
        // zlib.compress(("Mantenimiento de chancador primario, cambio de poste y limpieza de taza. " * 6 + "Fin.").encode(), 9)
        let d = [
            0x78, 0xDA, 0xED, 0xCC, 0xB1, 0x0D, 0x80, 0x30, 0x0C, 0x44, 0xD1, 0x55, 0x3C, 0x00, 0xCA, 0x18, 0x74, 0x19, 0xE2, 0x48, 0x2C, 0x71,
            0x12, 0xB1, 0xA3, 0xE0, 0x86, 0x4C, 0x8F, 0x60, 0x08, 0x2A, 0xDA, 0xFF, 0xA5, 0x97, 0x61, 0xA1, 0xC6, 0x46, 0xB5, 0x70, 0xA9, 0x2A,
            0x65, 0x87, 0x15, 0x54, 0x1F, 0xD2, 0x07, 0x1B, 0x06, 0x7D, 0x91, 0x82, 0xB6, 0xF1, 0xBD, 0xDD, 0xCF, 0x50, 0xB9, 0xE4, 0x60, 0xEB,
            0xD4, 0x89, 0xA7, 0x05, 0x26, 0x92, 0xE4, 0x1F, 0xFA, 0x0E, 0x5A, 0x69, 0xE9, 0x06, 0xC7, 0x65, 0xA0, 0x9A,
        ];
        let r = zlib(&d);
        assert!(r.ok);
        let esperado = "Mantenimiento de chancador primario, cambio de poste y limpieza de taza. ".repeat(6) + "Fin.";
        assert_eq!(r.data, esperado.as_bytes());
        // Hecho por zlib: tambien lo acepta la lectura estricta.
        assert_eq!(zlib_strict(&d).as_deref(), Some(esperado.as_bytes()));
    }

    #[test]
    fn datos_truncados() {
        let d = [0x78, 0xDA, 0xCB, 0xC8];
        let r = zlib(&d);
        assert!(!r.ok);
    }

    /// Escribe bits como DEFLATE: los numeros desde el bit bajo; los codigos Huffman, desde el alto.
    #[derive(Default)]
    struct Escritor {
        out: Vec<u8>,
        acc: u8,
        n: u32,
    }

    impl Escritor {
        fn bit(&mut self, b: u32) {
            self.acc |= (b as u8) << self.n;
            self.n += 1;
            if self.n == 8 {
                self.out.push(self.acc);
                self.acc = 0;
                self.n = 0;
            }
        }
        fn bits(&mut self, v: u32, count: u32) {
            for i in 0..count {
                self.bit((v >> i) & 1);
            }
        }
        fn code(&mut self, code: &str) {
            for c in code.bytes() {
                self.bit((c - b'0') as u32);
            }
        }
        fn fin(mut self) -> Vec<u8> {
            if self.n > 0 {
                self.out.push(self.acc);
            }
            self.out
        }
    }

    /// zlib de "a" en un bloque de Huffman dinamico. `largo_b` y `largo_fin` son los largos de los
    /// codigos de 'b' y del fin de bloque ('a' mide 1), y `fin` el codigo de fin de bloque.
    fn dinamico(largo_b: u32, largo_fin: u32, fin: &str) -> Vec<u8> {
        let mut w = Escritor::default();
        w.bits(1, 1); // ultimo bloque
        w.bits(2, 2); // Huffman dinamico
        w.bits(0, 5); // 257 literales y largos
        w.bits(0, 5); // 1 distancia
        w.bits(14, 4); // 18 largos del codigo de largos
        // Codigo de largos (completo): 18 -> "0", 1 -> "10", 2 -> "11".
        for s in [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1] {
            w.bits(
                match s {
                    18 => 1,
                    1 | 2 => 2,
                    _ => 0,
                },
                3,
            );
        }
        let mut largos = [0u32; 258];
        largos[97] = 1;
        largos[98] = largo_b;
        largos[256] = largo_fin;
        largos[257] = 1; // una sola distancia de largo 1: incompleto, pero zlib lo admite
        let mut i = 0;
        while i < largos.len() {
            if largos[i] == 0 {
                let mut run = 0;
                while i + run < largos.len() && largos[i + run] == 0 && run < 138 {
                    run += 1;
                }
                assert!(run >= 11);
                w.code("0");
                w.bits(run as u32 - 11, 7);
                i += run;
            } else {
                w.code(if largos[i] == 1 { "10" } else { "11" });
                i += 1;
            }
        }
        w.code("0"); // 'a'
        w.code(fin);
        let mut z = vec![0x78, 0x01];
        z.extend_from_slice(&w.fin());
        z.extend_from_slice(&((98u32 << 16) | 98).to_be_bytes()); // Adler-32 de "a"
        z
    }

    #[test]
    fn zlib_estricto_acepta_flujos_validos() {
        let fijo = [0x78, 0xDA, 0xCB, 0xC8, 0xCF, 0x49, 0x54, 0xC8, 0xC0, 0x24, 0x00, 0x6D, 0x1E, 0x08, 0xB5];
        assert_eq!(zlib_strict(&fijo).as_deref(), Some(&b"hola hola hola hola hola"[..]));
        let mut almacenado = vec![0x78, 0x01, 0x01, 0x04, 0x00, 0xFB, 0xFF, b'h', b'o', b'l', b'a'];
        almacenado.extend_from_slice(&crate::png::adler32(b"hola").to_be_bytes());
        assert_eq!(zlib_strict(&almacenado).as_deref(), Some(&b"hola"[..]));
        // Codigos completos ('a' y fin de bloque de largo 1).
        assert_eq!(zlib_strict(&dinamico(0, 1, "1")).as_deref(), Some(&b"a"[..]));
    }

    #[test]
    fn zlib_estricto_rechaza_lo_que_el_tolerante_lee() {
        let mut almacenado = vec![0x78, 0x01, 0x01, 0x04, 0x00, 0xFB, 0xFF, b'h', b'o', b'l', b'a'];
        almacenado.extend_from_slice(&crate::png::adler32(b"hola").to_be_bytes());
        let mut casos: Vec<(&str, Vec<u8>)> = Vec::new();
        // NLEN que no es el complemento de LEN.
        let mut nlen = almacenado.clone();
        nlen[5] = 0;
        casos.push(("NLEN", nlen));
        // Un byte de mas entre el fin del flujo DEFLATE y la suma.
        let mut sobra = almacenado.clone();
        sobra.insert(sobra.len() - 4, 0);
        casos.push(("byte sobrante", sobra));
        // Suma Adler-32 equivocada.
        let mut adler = almacenado.clone();
        let n = adler.len();
        adler[n - 1] ^= 1;
        casos.push(("Adler-32", adler));
        // Ventana de 64 KB (CINFO 8), con la cabecera bien verificada.
        let mut ventana = almacenado.clone();
        ventana[..2].copy_from_slice(&[0x88, 0x1C]);
        casos.push(("ventana", ventana));
        // Fin de bloque de largo 2 junto a 'a' de largo 1: codigo incompleto.
        casos.push(("incompleto", dinamico(0, 2, "10")));
        // 'a' y fin de bloque de largo 1, y ademas 'b' de largo 2: codigo sobrepasado.
        casos.push(("sobrepasado", dinamico(2, 1, "1")));
        for (nombre, z) in casos {
            let r = zlib(&z);
            assert!(r.ok && !r.data.is_empty(), "{}: el lector tolerante deberia leerlo", nombre);
            assert_eq!(zlib_strict(&z), None, "{}", nombre);
        }
    }
}
