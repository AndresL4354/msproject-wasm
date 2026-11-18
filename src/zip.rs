//! Escritor ZIP minimo, suficiente para empaquetar un .xlsx.
//!
//! Incluye un compresor DEFLATE propio y pequeno (LZ77 + codigos Huffman fijos): no es el mas
//! eficiente, pero reduce el XML de las hojas varias veces sin agregar dependencias.
//!
//! Licencia: ademas de la LGPL-2.1-or-later del resto de mpp2xlsx, este archivo se ofrece bajo
//! MIT o Apache-2.0, a eleccion, para que doc2json pueda reutilizar el compresor.

const fn crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            k += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
}

static CRC_TABLE: [u32; 256] = crc_table();

fn crc32(data: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c = CRC_TABLE[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}

// ---------------------------------------------------------------------------------------------
// DEFLATE (RFC 1951) con codigos Huffman fijos
// ---------------------------------------------------------------------------------------------

struct BitWriter {
    out: Vec<u8>,
    acc: u64,
    n: u32,
}

impl BitWriter {
    /// Escribe `count` bits, el menos significativo primero.
    fn bits(&mut self, value: u32, count: u32) {
        self.acc |= (value as u64) << self.n;
        self.n += count;
        while self.n >= 8 {
            self.out.push(self.acc as u8);
            self.acc >>= 8;
            self.n -= 8;
        }
    }

    /// Los codigos Huffman van con el bit mas significativo primero.
    fn code(&mut self, code: u32, len: u32) {
        self.bits(code.reverse_bits() >> (32 - len), len);
    }

    fn symbol(&mut self, sym: u32) {
        match sym {
            0..=143 => self.code(0x30 + sym, 8),
            144..=255 => self.code(0x190 + sym - 144, 9),
            256..=279 => self.code(sym - 256, 7),
            _ => self.code(0xC0 + sym - 280, 8),
        }
    }
}

const LEN_BASE: [u16; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
const LEN_EXTRA: [u8; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];

const WINDOW: usize = 32768;
const HASH_BITS: u32 = 15;
const MAX_CHAIN: usize = 16;
const MAX_MATCH: usize = 258;
/// Con una coincidencia de este largo ya no vale la pena seguir buscando otra mejor.
const NICE_MATCH: usize = 64;

fn hash3(d: &[u8], i: usize) -> usize {
    let v = (d[i] as u32) | (d[i + 1] as u32) << 8 | (d[i + 2] as u32) << 16;
    (v.wrapping_mul(0x9E37_79B1) >> (32 - HASH_BITS)) as usize
}

pub fn deflate(d: &[u8]) -> Vec<u8> {
    let mut w = BitWriter { out: Vec::with_capacity(d.len() / 6 + 64), acc: 0, n: 0 };
    w.bits(1, 1); // bloque final
    w.bits(1, 2); // Huffman fijo
    let n = d.len();
    // Cadenas de posiciones con el mismo hash de 3 bytes (0 = sin entrada; se guarda pos + 1).
    let mut head = vec![0u32; 1 << HASH_BITS];
    let mut prev = vec![0u32; n];
    let mut i = 0usize;
    while i < n {
        let mut best_len = 0usize;
        let mut best_dist = 0usize;
        if i + 3 <= n {
            let h = hash3(d, i);
            let mut cand = head[h] as usize;
            let limit = (n - i).min(MAX_MATCH);
            let mut chain = 0;
            while cand != 0 && chain < MAX_CHAIN {
                let c = cand - 1;
                if i - c > WINDOW {
                    break;
                }
                cand = prev[c] as usize;
                chain += 1;
                // Para superar a la mejor coincidencia tiene que calzar tambien en esa posicion.
                if d[c + best_len] != d[i + best_len] {
                    continue;
                }
                let mut l = 0usize;
                while l < limit && d[c + l] == d[i + l] {
                    l += 1;
                }
                if l > best_len {
                    best_len = l;
                    best_dist = i - c;
                    if l == limit || l >= NICE_MATCH {
                        break;
                    }
                }
            }
            prev[i] = head[h];
            head[h] = i as u32 + 1;
        }
        if best_len >= 3 {
            let li = LEN_BASE.iter().rposition(|&b| b as usize <= best_len).unwrap_or(0);
            w.symbol(257 + li as u32);
            w.bits((best_len - LEN_BASE[li] as usize) as u32, LEN_EXTRA[li] as u32);
            let di = DIST_BASE.iter().rposition(|&b| b as usize <= best_dist).unwrap_or(0);
            w.code(di as u32, 5);
            w.bits((best_dist - DIST_BASE[di] as usize) as u32, DIST_EXTRA[di] as u32);
            // Registra tambien las posiciones cubiertas por la coincidencia.
            for j in i + 1..i + best_len {
                if j + 3 <= n {
                    let h = hash3(d, j);
                    prev[j] = head[h];
                    head[h] = j as u32 + 1;
                }
            }
            i += best_len;
        } else {
            w.symbol(d[i] as u32);
            i += 1;
        }
    }
    w.symbol(256);
    if w.n > 0 {
        w.bits(0, 8 - w.n);
    }
    w.out
}

// ---------------------------------------------------------------------------------------------
// Contenedor ZIP
// ---------------------------------------------------------------------------------------------

// Fecha fija (2026-01-01 00:00) en formato DOS: la salida es reproducible byte a byte.
const DOS_DATE: u16 = ((2026 - 1980) << 9) | (1 << 5) | 1;
const DOS_TIME: u16 = 0;

struct Entry {
    name: String,
    crc: u32,
    size: u32,
    packed: u32,
    method: u16,
    offset: u32,
}

pub struct Zip {
    buf: Vec<u8>,
    entries: Vec<Entry>,
}

fn u16le(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn u32le(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

impl Zip {
    pub fn new() -> Zip {
        Zip { buf: Vec::new(), entries: Vec::new() }
    }

    pub fn add(&mut self, name: &str, data: &[u8]) {
        // Se comprime solo si de verdad reduce (los archivos diminutos quedan tal cual).
        let packed = deflate(data);
        let (method, body): (u16, &[u8]) = if packed.len() < data.len() { (8, &packed) } else { (0, data) };
        let e = Entry {
            name: name.to_string(),
            crc: crc32(data),
            size: data.len() as u32,
            packed: body.len() as u32,
            method,
            offset: self.buf.len() as u32,
        };
        let b = &mut self.buf;
        u32le(b, 0x0403_4B50);
        u16le(b, 20); // version necesaria
        u16le(b, 0); // indicadores
        u16le(b, e.method);
        u16le(b, DOS_TIME);
        u16le(b, DOS_DATE);
        u32le(b, e.crc);
        u32le(b, e.packed);
        u32le(b, e.size);
        u16le(b, e.name.len() as u16);
        u16le(b, 0);
        b.extend_from_slice(e.name.as_bytes());
        b.extend_from_slice(body);
        self.entries.push(e);
    }

    pub fn finish(mut self) -> Vec<u8> {
        let cd_offset = self.buf.len() as u32;
        let b = &mut self.buf;
        for e in &self.entries {
            u32le(b, 0x0201_4B50);
            u16le(b, 20); // creado por
            u16le(b, 20); // version necesaria
            u16le(b, 0);
            u16le(b, e.method);
            u16le(b, DOS_TIME);
            u16le(b, DOS_DATE);
            u32le(b, e.crc);
            u32le(b, e.packed);
            u32le(b, e.size);
            u16le(b, e.name.len() as u16);
            u16le(b, 0); // extra
            u16le(b, 0); // comentario
            u16le(b, 0); // disco
            u16le(b, 0); // atributos internos
            u32le(b, 0); // atributos externos
            u32le(b, e.offset);
            b.extend_from_slice(e.name.as_bytes());
        }
        let cd_size = b.len() as u32 - cd_offset;
        u32le(b, 0x0605_4B50);
        u16le(b, 0);
        u16le(b, 0);
        u16le(b, self.entries.len() as u16);
        u16le(b, self.entries.len() as u16);
        u32le(b, cd_size);
        u32le(b, cd_offset);
        u16le(b, 0);
        self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Descompresor minimo (solo bloques de Huffman fijo) para verificar `deflate`.
    fn inflate(src: &[u8]) -> Vec<u8> {
        struct R<'a> {
            d: &'a [u8],
            pos: usize,
        }
        impl R<'_> {
            fn bit(&mut self) -> u32 {
                let b = (self.d[self.pos / 8] >> (self.pos % 8)) & 1;
                self.pos += 1;
                b as u32
            }
            fn bits(&mut self, n: u32) -> u32 {
                (0..n).fold(0, |acc, i| acc | self.bit() << i)
            }
            fn symbol(&mut self) -> u32 {
                let mut code = 0u32;
                for len in 1..=9 {
                    code = code << 1 | self.bit();
                    match len {
                        7 if code <= 0x17 => return 256 + code,
                        8 if (0x30..=0xBF).contains(&code) => return code - 0x30,
                        8 if (0xC0..=0xC7).contains(&code) => return 280 + code - 0xC0,
                        9 if (0x190..=0x1FF).contains(&code) => return 144 + code - 0x190,
                        _ => {}
                    }
                }
                panic!("codigo invalido");
            }
        }
        let mut r = R { d: src, pos: 0 };
        assert_eq!(r.bits(1), 1);
        assert_eq!(r.bits(2), 1);
        let mut out = Vec::new();
        loop {
            let s = r.symbol();
            if s < 256 {
                out.push(s as u8);
            } else if s == 256 {
                return out;
            } else {
                let li = (s - 257) as usize;
                let len = LEN_BASE[li] as usize + r.bits(LEN_EXTRA[li] as u32) as usize;
                let di = (0..5).fold(0, |acc, _| acc << 1 | r.bit()) as usize;
                let dist = DIST_BASE[di] as usize + r.bits(DIST_EXTRA[di] as u32) as usize;
                for _ in 0..len {
                    out.push(out[out.len() - dist]);
                }
            }
        }
    }

    #[test]
    fn crc32_conocido() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn deflate_ida_y_vuelta() {
        let mut casos: Vec<Vec<u8>> = vec![Vec::new(), b"a".to_vec(), b"abcabcabcabcabcabc".to_vec(), vec![0u8; 100_000]];
        // XML repetitivo, como el de una hoja.
        let mut xml = String::new();
        for i in 0..5000 {
            xml.push_str(&format!("<c r=\"B{}\" s=\"3\" t=\"inlineStr\"><is><t>Tarea {}</t></is></c>", i, i * 7));
        }
        casos.push(xml.into_bytes());
        // Datos pseudoaleatorios (incompresibles) con todos los valores de byte.
        let mut x = 12345u32;
        let ruido: Vec<u8> = (0..70_000)
            .map(|_| {
                x = x.wrapping_mul(1664525).wrapping_add(1013904223);
                (x >> 24) as u8
            })
            .collect();
        casos.push(ruido);
        for caso in casos {
            assert_eq!(inflate(&deflate(&caso)), caso);
        }
    }

    #[test]
    fn zip_comprime_lo_repetitivo() {
        let data = vec![b'x'; 50_000];
        let mut z = Zip::new();
        z.add("a.txt", &data);
        assert!(z.finish().len() < 2_000);
    }
}
