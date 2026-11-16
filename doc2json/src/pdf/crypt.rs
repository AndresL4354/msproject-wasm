//! Descifrado de PDF con el manejador de seguridad estandar (RC4 de 40 a 128 bits y AES-128).
//!
//! Cubre el caso habitual de documentos "protegidos" contra edicion o copia pero que se
//! abren sin pedir contrasena (contrasena de usuario vacia). No rompe ninguna proteccion: usa
//! la misma clave que cualquier visor deriva para mostrar el documento.

const PAD: [u8; 32] = [
    0x28, 0xBF, 0x4E, 0x5E, 0x4E, 0x75, 0x8A, 0x41, 0x64, 0x00, 0x4E, 0x56, 0xFF, 0xFA, 0x01, 0x08, 0x2E, 0x2E, 0x00, 0xB6, 0xD0, 0x68, 0x3E, 0x80,
    0x2F, 0x0C, 0xA9, 0xFE, 0x64, 0x53, 0x69, 0x7A,
];

// ---------------------------------------------------------------------------------------------
// MD5 (RFC 1321)
// ---------------------------------------------------------------------------------------------

const MD5_K: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be,
    0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c,
    0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1, 0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1,
    0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];
const MD5_S: [u32; 16] = [7, 12, 17, 22, 5, 9, 14, 20, 4, 11, 16, 23, 6, 10, 15, 21];

pub fn md5(data: &[u8]) -> [u8; 16] {
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&((data.len() as u64).wrapping_mul(8)).to_le_bytes());
    let mut h: [u32; 4] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476];
    for chunk in msg.chunks_exact(64) {
        let mut m = [0u32; 16];
        for (i, w) in chunk.chunks_exact(4).enumerate() {
            m[i] = u32::from_le_bytes([w[0], w[1], w[2], w[3]]);
        }
        let (mut a, mut b, mut c, mut d) = (h[0], h[1], h[2], h[3]);
        for i in 0..64 {
            let (f, g) = match i / 16 {
                0 => ((b & c) | (!b & d), i),
                1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                2 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let f = f.wrapping_add(a).wrapping_add(MD5_K[i]).wrapping_add(m[g]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(f.rotate_left(MD5_S[(i / 16) * 4 + i % 4]));
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
    }
    let mut out = [0u8; 16];
    for (i, v) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
    }
    out
}

// ---------------------------------------------------------------------------------------------
// RC4
// ---------------------------------------------------------------------------------------------

pub fn rc4(key: &[u8], data: &[u8]) -> Vec<u8> {
    if key.is_empty() {
        return data.to_vec();
    }
    let mut s: [u8; 256] = [0; 256];
    for (i, v) in s.iter_mut().enumerate() {
        *v = i as u8;
    }
    let mut j = 0u8;
    for i in 0..256 {
        j = j.wrapping_add(s[i]).wrapping_add(key[i % key.len()]);
        s.swap(i, j as usize);
    }
    let (mut i, mut j) = (0u8, 0u8);
    data.iter()
        .map(|b| {
            i = i.wrapping_add(1);
            j = j.wrapping_add(s[i as usize]);
            s.swap(i as usize, j as usize);
            b ^ s[s[i as usize].wrapping_add(s[j as usize]) as usize]
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// AES-128 (solo descifrado, modo CBC)
// ---------------------------------------------------------------------------------------------

fn sboxes() -> ([u8; 256], [u8; 256]) {
    // La caja S se genera con la aritmetica del campo GF(2^8), sin tablas escritas a mano.
    let mut sbox = [0u8; 256];
    let (mut p, mut q) = (1u8, 1u8);
    loop {
        p = p ^ (p << 1) ^ if p & 0x80 != 0 { 0x1B } else { 0 };
        q ^= q << 1;
        q ^= q << 2;
        q ^= q << 4;
        if q & 0x80 != 0 {
            q ^= 0x09;
        }
        sbox[p as usize] = q ^ q.rotate_left(1) ^ q.rotate_left(2) ^ q.rotate_left(3) ^ q.rotate_left(4) ^ 0x63;
        if p == 1 {
            break;
        }
    }
    sbox[0] = 0x63;
    let mut inv = [0u8; 256];
    for (i, v) in sbox.iter().enumerate() {
        inv[*v as usize] = i as u8;
    }
    (sbox, inv)
}

fn gmul(mut a: u8, mut b: u8) -> u8 {
    let mut r = 0u8;
    while b != 0 {
        if b & 1 != 0 {
            r ^= a;
        }
        a = (a << 1) ^ if a & 0x80 != 0 { 0x1B } else { 0 };
        b >>= 1;
    }
    r
}

fn aes128_decrypt_block(block: &mut [u8; 16], keys: &[[u8; 16]; 11], inv: &[u8; 256]) {
    let add = |b: &mut [u8; 16], k: &[u8; 16]| b.iter_mut().zip(k).for_each(|(x, y)| *x ^= y);
    add(block, &keys[10]);
    for round in (0..10).rev() {
        // Desplazamiento inverso de filas (el estado esta por columnas) y sustitucion inversa.
        let s = *block;
        for c in 0..4 {
            for r in 0..4 {
                block[c * 4 + r] = inv[s[((c + 4 - r) % 4) * 4 + r] as usize];
            }
        }
        add(block, &keys[round]);
        if round > 0 {
            for c in 0..4 {
                let col = [block[c * 4], block[c * 4 + 1], block[c * 4 + 2], block[c * 4 + 3]];
                for r in 0..4 {
                    block[c * 4 + r] = gmul(col[r], 0x0E) ^ gmul(col[(r + 1) % 4], 0x0B) ^ gmul(col[(r + 2) % 4], 0x0D) ^ gmul(col[(r + 3) % 4], 0x09);
                }
            }
        }
    }
}

/// Descifra AES-128-CBC: los primeros 16 bytes son el vector inicial; quita el relleno final.
pub fn aes128_cbc_decrypt(key: &[u8], data: &[u8]) -> Vec<u8> {
    if key.len() != 16 || data.len() < 32 {
        return Vec::new();
    }
    let (sbox, inv) = sboxes();
    let mut keys = [[0u8; 16]; 11];
    keys[0].copy_from_slice(key);
    let mut rcon = 1u8;
    for i in 1..11 {
        let prev = keys[i - 1];
        let mut t = [prev[13], prev[14], prev[15], prev[12]].map(|b| sbox[b as usize]);
        t[0] ^= rcon;
        rcon = (rcon << 1) ^ if rcon & 0x80 != 0 { 0x1B } else { 0 };
        for j in 0..16 {
            let x = if j < 4 { t[j] } else { keys[i][j - 4] };
            keys[i][j] = prev[j] ^ x;
        }
    }
    let mut out = Vec::with_capacity(data.len() - 16);
    let mut prev: [u8; 16] = data[..16].try_into().unwrap_or([0; 16]);
    for chunk in data[16..].chunks_exact(16) {
        let cipher: [u8; 16] = chunk.try_into().unwrap_or([0; 16]);
        let mut block = cipher;
        aes128_decrypt_block(&mut block, &keys, &inv);
        for (b, p) in block.iter_mut().zip(&prev) {
            *b ^= p;
        }
        out.extend_from_slice(&block);
        prev = cipher;
    }
    // Relleno PKCS#7.
    if let Some(&n) = out.last() {
        if (1..=16).contains(&n) && out.len() >= n as usize {
            out.truncate(out.len() - n as usize);
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------
// Manejador de seguridad estandar
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Method {
    None,
    Rc4,
    Aes,
}

pub struct Crypt {
    key: Vec<u8>,
    pub strings: Method,
    pub streams: Method,
}

/// Datos del diccionario `/Encrypt` ya extraidos.
pub struct Params<'a> {
    pub filter: &'a str,
    pub v: i64,
    pub r: i64,
    pub length: i64,
    pub o: &'a [u8],
    pub u: &'a [u8],
    pub p: i64,
    pub id: &'a [u8],
    pub encrypt_metadata: bool,
    /// Metodos de los filtros de cifrado para cadenas y flujos (solo version 4).
    pub strings: Option<&'a str>,
    pub streams: Option<&'a str>,
}

impl Crypt {
    pub fn new(p: &Params) -> Result<Crypt, String> {
        if p.filter != "Standard" {
            return Err("El PDF usa un sistema de proteccion no estandar (certificados o DRM) y no se puede leer.".into());
        }
        let method = |cfm: Option<&str>| match cfm {
            Some("V2") => Ok(Method::Rc4),
            Some("AESV2") => Ok(Method::Aes),
            Some("None") | Some("Identity") | None => Ok(Method::None),
            _ => Err("El PDF usa un cifrado no soportado (AES-256).".to_string()),
        };
        let (strings, streams, bits) = match p.v {
            1 => (Method::Rc4, Method::Rc4, 40),
            2 | 3 => (Method::Rc4, Method::Rc4, p.length.clamp(40, 128)),
            4 => (method(p.strings)?, method(p.streams)?, 128),
            _ => return Err("El PDF usa cifrado AES-256, que esta herramienta no abre. Guarde una copia sin proteccion y vuelva a intentar.".into()),
        };
        let n = if p.r <= 2 { 5 } else { (bits / 8) as usize };
        // Clave a partir de la contrasena de usuario vacia (algoritmo 2 de la especificacion).
        let mut input = PAD.to_vec();
        input.extend_from_slice(&p.o[..p.o.len().min(32)]);
        input.extend_from_slice(&(p.p as u32).to_le_bytes());
        input.extend_from_slice(p.id);
        if p.r >= 4 && !p.encrypt_metadata {
            input.extend_from_slice(&[0xFF; 4]);
        }
        let mut hash = md5(&input);
        if p.r >= 3 {
            for _ in 0..50 {
                hash = md5(&hash[..n]);
            }
        }
        let key = hash[..n].to_vec();

        // Comprobacion: con esa clave debe reproducirse la entrada /U.
        let ok = if p.r <= 2 {
            rc4(&key, &PAD) == p.u[..p.u.len().min(32)]
        } else {
            let mut seed = PAD.to_vec();
            seed.extend_from_slice(p.id);
            let mut x = rc4(&key, &md5(&seed));
            for i in 1..=19u8 {
                let k: Vec<u8> = key.iter().map(|b| b ^ i).collect();
                x = rc4(&k, &x);
            }
            p.u.len() >= 16 && x[..16] == p.u[..16]
        };
        if !ok {
            return Err("El PDF pide una contrasena para abrirlo. Quitele la contrasena y vuelva a intentar.".into());
        }
        Ok(Crypt { key, strings, streams })
    }

    /// Descifra los datos del objeto `num gen` (algoritmo 1).
    pub fn decrypt(&self, method: Method, num: u32, gen: u16, data: &[u8]) -> Vec<u8> {
        if method == Method::None {
            return data.to_vec();
        }
        let mut k = self.key.clone();
        k.extend_from_slice(&num.to_le_bytes()[..3]);
        k.extend_from_slice(&gen.to_le_bytes());
        if method == Method::Aes {
            k.extend_from_slice(b"sAlT");
        }
        let hash = md5(&k);
        let key = &hash[..(self.key.len() + 5).min(16)];
        match method {
            Method::Aes => aes128_cbc_decrypt(key, data),
            _ => rc4(key, data),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{:02x}", x)).collect()
    }

    #[test]
    fn md5_conocidos() {
        assert_eq!(hex(&md5(b"")), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(hex(&md5(b"abc")), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(hex(&md5(&[b'a'; 1000])), "cabe45dcc9ae5b66ba86600cca6b8ba8");
    }

    #[test]
    fn rc4_conocido() {
        assert_eq!(hex(&rc4(b"Key", b"Plaintext")), "bbf316e8d940af0ad3");
    }

    #[test]
    fn aes_conocido() {
        // Vector de FIPS-197: clave 00..0f, texto 00112233..ff -> 69c4e0d8...
        let key: Vec<u8> = (0u8..16).collect();
        let cipher = [0x69, 0xC4, 0xE0, 0xD8, 0x6A, 0x7B, 0x04, 0x30, 0xD8, 0xCD, 0xB7, 0x80, 0x70, 0xB4, 0xC5, 0x5A];
        // Vector inicial de ceros + el bloque cifrado.
        let mut data = vec![0u8; 16];
        data.extend_from_slice(&cipher);
        data.extend_from_slice(&cipher); // segundo bloque para que el relleno no afecte al primero
        let plain = aes128_cbc_decrypt(&key, &data);
        let esperado: Vec<u8> = (0u8..16).map(|i| i * 0x11).collect();
        assert_eq!(&plain[..16], &esperado[..]);
    }
}
