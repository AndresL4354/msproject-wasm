//! Tabla hash con una funcion de dispersion rapida.
//!
//! El `HashMap` estandar usa SipHash, pensado para resistir entradas maliciosas. Aqui las
//! claves son numeros pequenos (identificadores de campo y de tarea) y se consultan cientos
//! de miles de veces por archivo, asi que conviene una dispersion multiplicativa simple.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default, Clone, Copy)]
pub struct FastHasher(u64);

impl FastHasher {
    #[inline]
    fn mix(&mut self, v: u64) {
        self.0 = (self.0.rotate_left(5) ^ v).wrapping_mul(0x517C_C1B7_2722_0A95);
    }
}

impl Hasher for FastHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.mix(b as u64);
        }
    }

    #[inline]
    fn write_u8(&mut self, v: u8) {
        self.mix(v as u64);
    }

    #[inline]
    fn write_u16(&mut self, v: u16) {
        self.mix(v as u64);
    }

    #[inline]
    fn write_u32(&mut self, v: u32) {
        self.mix(v as u64);
    }

    #[inline]
    fn write_i32(&mut self, v: i32) {
        self.mix(v as u32 as u64);
    }

    #[inline]
    fn write_u64(&mut self, v: u64) {
        self.mix(v);
    }

    #[inline]
    fn write_i64(&mut self, v: i64) {
        self.mix(v as u64);
    }

    #[inline]
    fn write_usize(&mut self, v: usize) {
        self.mix(v as u64);
    }
}

pub type FastMap<K, V> = HashMap<K, V, BuildHasherDefault<FastHasher>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapa_rapido() {
        let mut m: FastMap<(i32, u16), usize> = FastMap::default();
        for uid in 0..2000i32 {
            for key in 0..8u16 {
                m.insert((uid, key), uid as usize * 8 + key as usize);
            }
        }
        assert_eq!(m.len(), 16000);
        assert_eq!(m.get(&(1234, 5)), Some(&(1234 * 8 + 5)));
        assert_eq!(m.get(&(-1, 0)), None);
    }
}
