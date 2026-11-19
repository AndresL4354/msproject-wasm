//! Descompresor DEFLATE con flujos armados bit a bit: codigos de mas de 10 bits (los que no
//! resuelve la tabla rapida), arboles de un solo codigo, bloques almacenados despues de uno
//! Huffman, cortes byte a byte y datos danados. Todo se compara ademas con el decodificador de
//! `main` (copiado en `referencia/inflate_main.rs`): la salida y `ok` tienen que ser iguales.

#[allow(dead_code, clippy::all)]
#[path = "referencia/inflate_main.rs"]
mod anterior;

use doc2json::inflate::{inflate, zlib};

/// Escribe bits en el orden de DEFLATE: desde el menos significativo de cada byte.
#[derive(Default)]
struct Escritor {
    out: Vec<u8>,
    acc: u64,
    n: u32,
}

impl Escritor {
    fn bits(&mut self, v: u32, n: u32) {
        self.acc |= (v as u64) << self.n;
        self.n += n;
        while self.n >= 8 {
            self.out.push(self.acc as u8);
            self.acc >>= 8;
            self.n -= 8;
        }
    }

    /// Un codigo Huffman va desde su bit mas significativo.
    fn codigo(&mut self, (code, len): (u32, u32)) {
        assert!(len > 0, "simbolo sin codigo");
        for i in (0..len).rev() {
            self.bits((code >> i) & 1, 1);
        }
    }

    fn alinear(&mut self) {
        if self.n > 0 {
            self.bits(0, 8 - self.n);
        }
    }

    fn fin(mut self) -> Vec<u8> {
        self.alinear();
        self.out
    }
}

/// Codigos canonicos por simbolo (RFC 1951, 3.2.2): (codigo, largo).
fn canonicos(largos: &[u8]) -> Vec<(u32, u32)> {
    let mut count = [0u32; 16];
    for &l in largos {
        count[l as usize] += 1;
    }
    count[0] = 0;
    let mut next = [0u32; 16];
    let mut code = 0;
    for len in 1..16 {
        code = (code + count[len - 1]) << 1;
        next[len] = code;
    }
    largos
        .iter()
        .map(|&l| {
            if l == 0 {
                return (0, 0);
            }
            let c = next[l as usize];
            next[l as usize] += 1;
            (c, l as u32)
        })
        .collect()
}

const LEN_BASE: [usize; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
const LEN_EXTRA: [u32; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
const DIST_BASE: [usize; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u32; 30] = [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];

/// Un simbolo de un bloque Huffman: un literal o una copia (largo, distancia).
#[derive(Clone, Copy)]
enum Sim {
    Lit(u8),
    Copia(usize, usize),
}

use Sim::{Copia, Lit};

fn literales(s: &[u8]) -> Vec<Sim> {
    s.iter().map(|&b| Lit(b)).collect()
}

/// Lo que da descomprimir los simbolos, calculado aparte.
fn expandir(out: &mut Vec<u8>, sims: &[Sim]) {
    for s in sims {
        match *s {
            Lit(b) => out.push(b),
            Copia(len, dist) => {
                for _ in 0..len {
                    out.push(out[out.len() - dist]);
                }
            }
        }
    }
}

/// Los simbolos y el fin de bloque con los codigos dados.
fn cuerpo(w: &mut Escritor, lit: &[(u32, u32)], dist: &[(u32, u32)], sims: &[Sim]) {
    for s in sims {
        match *s {
            Lit(b) => w.codigo(lit[b as usize]),
            Copia(len, d) => {
                let i = (0..29).rev().find(|&i| LEN_BASE[i] <= len).unwrap();
                w.codigo(lit[257 + i]);
                w.bits((len - LEN_BASE[i]) as u32, LEN_EXTRA[i]);
                let j = (0..30).rev().find(|&j| DIST_BASE[j] <= d).unwrap();
                w.codigo(dist[j]);
                w.bits((d - DIST_BASE[j]) as u32, DIST_EXTRA[j]);
            }
        }
    }
    w.codigo(lit[256]);
}

/// Bloque con Huffman dinamico de los largos dados.
fn dinamico(w: &mut Escritor, ultimo: bool, lit: &[u8], dist: &[u8], sims: &[Sim]) {
    w.bits(ultimo as u32, 1);
    w.bits(2, 2);
    w.bits(lit.len() as u32 - 257, 5);
    w.bits(dist.len() as u32 - 1, 5);
    w.bits(19 - 4, 4);
    // Codigo de los largos: 0 a 15 con 4 bits cada uno (completo), sin repeticiones.
    let mut cl = [0u8; 19];
    cl[..16].fill(4);
    for o in [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15] {
        w.bits(cl[o] as u32, 3);
    }
    let clc = canonicos(&cl);
    for &l in lit.iter().chain(dist) {
        w.codigo(clc[l as usize]);
    }
    cuerpo(w, &canonicos(lit), &canonicos(dist), sims);
}

/// Bloque con el Huffman fijo.
fn fijo(w: &mut Escritor, ultimo: bool, sims: &[Sim]) {
    w.bits(ultimo as u32, 1);
    w.bits(1, 2);
    let mut l = [8u8; 288];
    l[144..256].fill(9);
    l[256..280].fill(7);
    cuerpo(w, &canonicos(&l), &canonicos(&[5u8; 30]), sims);
}

/// Bloque almacenado.
fn almacenado(w: &mut Escritor, ultimo: bool, datos: &[u8]) {
    w.bits(ultimo as u32, 1);
    w.bits(0, 2);
    w.alinear();
    w.bits(datos.len() as u32, 16);
    w.bits(!(datos.len() as u32) & 0xFFFF, 16);
    for &b in datos {
        w.bits(b as u32, 8);
    }
}

/// Largos de un codigo completo con codigos de 1 a 15 bits: `a` 1 bit, `b` 2... `l` 12, la copia
/// de 3 bytes 13, `m` 14, y el fin de bloque y la copia de 258 bytes 15.
fn largos_hasta_15() -> Vec<u8> {
    let mut lit = vec![0u8; 286];
    for (k, c) in (b'a'..=b'l').enumerate() {
        lit[c as usize] = k as u8 + 1;
    }
    lit[257] = 13;
    lit[b'm' as usize] = 14;
    lit[256] = 15;
    lit[285] = 15;
    lit
}

/// Los simbolos de la prueba de codigos largos: todos los literales, copias que no se solapan
/// (distancia 5 o 6 desde un arbol de distancias de un solo codigo) y una que si (258 bytes).
fn sims_largos() -> Vec<Sim> {
    let mut s = literales(b"abcdefghijklm");
    s.extend([Copia(3, 5), Lit(b'k'), Copia(258, 6), Lit(b'm'), Copia(3, 6)]);
    s.extend(literales(b"lmklmaaab"));
    s
}

/// Un solo codigo de distancia: el simbolo 4 (distancias 5 y 6), de 1 bit.
const DIST_UNO: [u8; 5] = [0, 0, 0, 0, 1];

/// Compara con el decodificador de `main`, crudo y con envoltura zlib.
fn igual_que_main(d: &[u8]) {
    let (a, b) = (inflate(d), anterior::inflate(d));
    assert_eq!((a.ok, &a.data), (b.ok, &b.data), "inflate distinto de main con {:02x?}", d);
    let (a, b) = (zlib(d), anterior::zlib(d));
    assert_eq!((a.ok, &a.data), (b.ok, &b.data), "zlib distinto de main con {:02x?}", d);
}

/// Descomprime, compara con lo esperado y con `main`, y prueba cada corte del flujo.
fn probar(d: &[u8], esperado: &[u8]) {
    let r = inflate(d);
    assert!(r.ok);
    assert_eq!(r.data, esperado);
    for corte in 0..=d.len() {
        igual_que_main(&d[..corte]);
        if corte < d.len() {
            assert!(!inflate(&d[..corte]).ok, "cortado en {} de {} dio ok", corte, d.len());
        }
    }
}

#[test]
fn codigos_de_mas_de_10_bits() {
    let sims = sims_largos();
    let mut w = Escritor::default();
    dinamico(&mut w, true, &largos_hasta_15(), &DIST_UNO, &sims);
    let mut esperado = Vec::new();
    expandir(&mut esperado, &sims);
    probar(&w.fin(), &esperado);
}

/// Largos de un codigo completo con el fin de bloque de 1 bit: `a` 2 bits, `b` 3... `i` y `j` 10.
fn largos_fin_corto() -> Vec<u8> {
    let mut lit = vec![0u8; 257];
    lit[256] = 1;
    for (k, c) in (b'a'..=b'i').enumerate() {
        lit[c as usize] = k as u8 + 2;
    }
    lit[b'j' as usize] = 10;
    lit
}

#[test]
fn bloque_almacenado_tras_uno_huffman() {
    // El bloque almacenado empieza en cada posicion de bit posible. Tras un fin de bloque corto,
    // la tabla rapida ya cargo un byte entero del bloque almacenado: tiene que volver al flujo.
    let guardado: Vec<u8> = (0..300u32).map(|i| (i * 7 + 3) as u8).collect();
    let mut solo_fin = vec![0u8; 257];
    solo_fin[256] = 1;
    for k in 0..16 {
        // Literales de 8 y 9 bits en el Huffman fijo, de 2 a 10 en el dinamico.
        let primero: Vec<Sim> = (0..k).map(|i| Lit(if i % 2 == 0 { b'a' + i as u8 } else { 0xC0 + i as u8 })).collect();
        let corto = literales(&b"abcdefghijjihgfe"[..k]);
        let mut w = Escritor::default();
        fijo(&mut w, false, &primero);
        almacenado(&mut w, false, &guardado);
        dinamico(&mut w, false, &largos_fin_corto(), &DIST_UNO, &corto);
        almacenado(&mut w, false, b"tras un fin de 1 bit");
        // Un bloque vacio de un solo codigo, otro almacenado vacio (como el de una descarga
        // completa de zlib) y uno con codigos largos.
        dinamico(&mut w, false, &solo_fin, &[1], &[]);
        almacenado(&mut w, false, &[]);
        almacenado(&mut w, false, b"tras un bloque vacio");
        dinamico(&mut w, true, &largos_hasta_15(), &DIST_UNO, &sims_largos());
        let mut esperado = Vec::new();
        expandir(&mut esperado, &primero);
        esperado.extend_from_slice(&guardado);
        expandir(&mut esperado, &corto);
        esperado.extend_from_slice(b"tras un fin de 1 bit");
        esperado.extend_from_slice(b"tras un bloque vacio");
        expandir(&mut esperado, &sims_largos());
        probar(&w.fin(), &esperado);
    }
}

#[test]
fn arboles_de_un_solo_codigo() {
    // Distancias con un solo codigo (distancia 1): una copia que se solapa con lo que agrega.
    // Literales y largos con un codigo completo: 256 de 9 bits, 2 de 5 y 28 de 6.
    let mut lit = vec![9u8; 286];
    lit[256..258].fill(5);
    lit[258..].fill(6);
    let sims = [Lit(b'x'), Copia(10, 1), Lit(b'y'), Copia(258, 1)];
    let mut w = Escritor::default();
    dinamico(&mut w, true, &lit, &[1], &sims);
    let mut esperado = Vec::new();
    expandir(&mut esperado, &sims);
    probar(&w.fin(), &esperado);

    // Literales con un solo codigo, el fin de bloque: un bloque vacio, y despues otro con datos.
    let mut solo_fin = vec![0u8; 257];
    solo_fin[256] = 1;
    let mut w = Escritor::default();
    dinamico(&mut w, false, &solo_fin, &[1], &[]);
    fijo(&mut w, true, &literales(b"sigue"));
    probar(&w.fin(), b"sigue");

    // El codigo que falta en un arbol de un solo codigo (un 1 donde solo existe el 0) es un dato
    // danado: se corta igual que en `main`.
    let mut w = Escritor::default();
    fijo(&mut w, false, &literales(b"antes"));
    dinamico(&mut w, true, &solo_fin, &[1], &[]);
    // El fin de bloque (un 0) es el ultimo bit escrito: se cambia por el codigo que no existe.
    let ultimo = w.out.len() * 8 + w.n as usize - 1;
    let mut d = w.fin();
    let r = inflate(&d);
    assert!(r.ok && r.data == b"antes");
    d[ultimo / 8] |= 1 << (ultimo % 8);
    let r = inflate(&d);
    assert!(!r.ok);
    assert_eq!(r.data, b"antes");
    igual_que_main(&d);
}

#[test]
fn igual_que_main_con_datos_danados() {
    // Flujos validos con bits cambiados, bytes reemplazados y cortes, de forma reproducible.
    let mut base = Vec::new();
    let mut w = Escritor::default();
    dinamico(&mut w, true, &largos_hasta_15(), &DIST_UNO, &sims_largos());
    base.push(w.fin());
    let mut w = Escritor::default();
    fijo(&mut w, false, &literales(b"fijo y luego almacenado"));
    almacenado(&mut w, false, b"guardado tal cual");
    dinamico(&mut w, true, &largos_hasta_15(), &DIST_UNO, &sims_largos());
    base.push(w.fin());
    // zlib.compress(b"hola hola hola hola hola", 9), sin la envoltura.
    base.push(vec![0xCB, 0xC8, 0xCF, 0x49, 0x54, 0xC8, 0xC0, 0x24, 0x00]);
    let mut x = 0x9E37_79B9_7F4A_7C15u64;
    let mut azar = move || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    };
    for d in &base {
        for _ in 0..2000 {
            let mut m = d.clone();
            for _ in 0..1 + azar() % 3 {
                let k = (azar() % m.len() as u64) as usize;
                match azar() % 3 {
                    0 => m[k] ^= 1 << (azar() % 8),
                    1 => m[k] = azar() as u8,
                    _ => m.truncate(k.max(1)),
                }
            }
            igual_que_main(&m);
        }
    }
}
