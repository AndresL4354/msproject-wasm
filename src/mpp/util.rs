//! Lectura segura de bytes y decodificacion de fechas, duraciones y cadenas del formato MPP.
//!
//! Todas las lecturas fuera de rango devuelven cero en vez de abortar: un archivo danado
//! produce datos incompletos, nunca un panic.

use crate::model::{Dur, Unit};

pub fn u8_at(d: &[u8], o: usize) -> u8 {
    d.get(o).copied().unwrap_or(0)
}

pub fn u16_at(d: &[u8], o: usize) -> u16 {
    match d.get(o..o.wrapping_add(2)) {
        Some(b) => u16::from_le_bytes([b[0], b[1]]),
        None => 0,
    }
}

pub fn u32_at(d: &[u8], o: usize) -> u32 {
    match d.get(o..o.wrapping_add(4)) {
        Some(b) => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
        None => 0,
    }
}

pub fn i32_at(d: &[u8], o: usize) -> i32 {
    u32_at(d, o) as i32
}

pub fn i64_at(d: &[u8], o: usize) -> i64 {
    match d.get(o..o.wrapping_add(8)) {
        Some(b) => i64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]),
        None => 0,
    }
}

pub fn f64_at(d: &[u8], o: usize) -> f64 {
    let v = f64::from_bits(i64_at(d, o) as u64);
    if v.is_nan() {
        0.0
    } else {
        v
    }
}

/// Cadena UTF-16LE terminada en doble cero (o hasta el final del bloque).
pub fn utf16(d: &[u8], off: usize) -> String {
    if off >= d.len() {
        return String::new();
    }
    let mut units = Vec::new();
    let mut i = off;
    while i + 1 < d.len() {
        let u = u16::from_le_bytes([d[i], d[i + 1]]);
        if u == 0 {
            break;
        }
        units.push(u);
        i += 2;
    }
    String::from_utf16_lossy(&units)
}

/// Cadena de un byte por caracter (Latin-1) terminada en cero.
pub fn latin1(d: &[u8], off: usize) -> String {
    d.iter().skip(off).take_while(|&&b| b != 0).map(|&b| b as char).collect()
}

/// Fecha y hora: dos enteros de 16 bits (decimas de minuto, dias desde 1983-12-31).
/// Devuelve segundos desde la epoca de Project.
pub fn timestamp(d: &[u8], off: usize) -> Option<i64> {
    if off.wrapping_add(4) > d.len() {
        return None;
    }
    let days = u16_at(d, off + 2) as i64;
    if days <= 1 || days == 65535 {
        return None;
    }
    let mut time = u16_at(d, off) as i64;
    if time == 65535 {
        time = 0;
    }
    let secs = time * 6;
    // Valores de dias muy pequenos con segundos sueltos se muestran como "NA" en Project.
    if days < 100 && secs % 60 != 0 {
        return None;
    }
    Some(days * 86400 + secs)
}

pub fn duration_units(code: u16, default: Option<Unit>) -> Unit {
    match code & 0x1F {
        3 => Unit::Minutes,
        4 => Unit::ElapsedMinutes,
        5 => Unit::Hours,
        6 => Unit::ElapsedHours,
        7 => Unit::Days,
        8 => Unit::ElapsedDays,
        9 => Unit::Weeks,
        10 => Unit::ElapsedWeeks,
        11 => Unit::Months,
        12 => Unit::ElapsedMonths,
        19 => Unit::Percent,
        20 => Unit::ElapsedPercent,
        21 => default.unwrap_or(Unit::Days),
        _ => Unit::Days,
    }
}

/// Parametros del proyecto necesarios para convertir duraciones.
#[derive(Clone, Copy)]
pub struct Scale {
    pub minutes_per_day: f64,
    pub minutes_per_week: f64,
    pub days_per_month: f64,
    pub default_units: Unit,
}

/// Convierte una duracion almacenada (decimas de minuto) a las unidades indicadas.
pub fn adjusted_duration(sc: &Scale, raw: i32, unit: Unit) -> Option<Dur> {
    if raw == -1 {
        return None;
    }
    let v = raw as f64;
    let div = |x: f64| if x != 0.0 { v / x } else { 0.0 };
    let value = match unit {
        Unit::Days => div(sc.minutes_per_day * 10.0),
        Unit::ElapsedDays => v / 14400.0,
        Unit::Weeks => div(sc.minutes_per_week * 10.0),
        Unit::ElapsedWeeks => v / 100800.0,
        Unit::Months => div(sc.minutes_per_day * sc.days_per_month * 10.0),
        Unit::ElapsedMonths => v / 432000.0,
        Unit::Minutes | Unit::ElapsedMinutes => v / 10.0,
        Unit::Hours | Unit::ElapsedHours => v / 600.0,
        Unit::Percent | Unit::ElapsedPercent => v,
    };
    Some(Dur { value, unit })
}

/// Duracion expresada en horas de trabajo (para comparar y para el JSON).
pub fn to_hours(sc: &Scale, d: Dur) -> Option<f64> {
    Some(match d.unit {
        Unit::Minutes | Unit::ElapsedMinutes => d.value / 60.0,
        Unit::Hours | Unit::ElapsedHours => d.value,
        Unit::Days => d.value * sc.minutes_per_day / 60.0,
        Unit::ElapsedDays => d.value * 24.0,
        Unit::Weeks => d.value * sc.minutes_per_week / 60.0,
        Unit::ElapsedWeeks => d.value * 168.0,
        Unit::Months => d.value * sc.minutes_per_day * sc.days_per_month / 60.0,
        Unit::ElapsedMonths => d.value * 720.0,
        Unit::Percent | Unit::ElapsedPercent => return None,
    })
}

/// Quita el formato RTF de una nota y devuelve texto plano.
pub fn rtf_to_text(src: &str) -> String {
    if !src.trim_start().starts_with("{\\rtf") {
        return src.trim().to_string();
    }
    let b: Vec<char> = src.chars().collect();
    let mut out = String::new();
    // Pila de "este grupo se ignora" (tablas de fuentes, colores, estilos, etc.).
    let mut skip_stack: Vec<bool> = Vec::new();
    let mut skip = false;
    let mut uc_skip = 0usize;
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i];
        match c {
            '{' => {
                skip_stack.push(skip);
                i += 1;
            }
            '}' => {
                skip = skip_stack.pop().unwrap_or(false);
                i += 1;
            }
            '\\' => {
                i += 1;
                if i >= b.len() {
                    break;
                }
                let n = b[i];
                if n == '\\' || n == '{' || n == '}' {
                    if !skip {
                        out.push(n);
                    }
                    i += 1;
                } else if n == '\'' {
                    // \'hh : byte en la pagina de codigos (se asume Windows-1252).
                    let hex: String = b.iter().skip(i + 1).take(2).collect();
                    i += 3;
                    if uc_skip > 0 {
                        uc_skip -= 1;
                    } else if let Ok(v) = u8::from_str_radix(&hex, 16) {
                        if !skip {
                            out.push(cp1252(v));
                        }
                    }
                } else if n == '*' {
                    skip = true;
                    i += 1;
                } else if n == '~' {
                    if !skip {
                        out.push(' ');
                    }
                    i += 1;
                } else if n.is_ascii_alphabetic() {
                    let mut word = String::new();
                    while i < b.len() && b[i].is_ascii_alphabetic() {
                        word.push(b[i]);
                        i += 1;
                    }
                    let mut num = String::new();
                    if i < b.len() && (b[i] == '-' || b[i].is_ascii_digit()) {
                        num.push(b[i]);
                        i += 1;
                        while i < b.len() && b[i].is_ascii_digit() {
                            num.push(b[i]);
                            i += 1;
                        }
                    }
                    if i < b.len() && b[i] == ' ' {
                        i += 1;
                    }
                    match word.as_str() {
                        "par" | "line" | "sect" | "page" => {
                            if !skip {
                                out.push('\n');
                            }
                        }
                        "tab" => {
                            if !skip {
                                out.push('\t');
                            }
                        }
                        "u" => {
                            if let Ok(v) = num.parse::<i32>() {
                                let cp = if v < 0 { (v + 65536) as u32 } else { v as u32 };
                                if !skip {
                                    out.push(char::from_u32(cp).unwrap_or('?'));
                                }
                                uc_skip = 1;
                            }
                        }
                        "fonttbl" | "colortbl" | "stylesheet" | "info" | "pict" | "object" | "header" | "footer"
                        | "generator" | "listtable" | "listoverridetable" | "themedata" | "datastore" => skip = true,
                        _ => {}
                    }
                } else {
                    i += 1;
                }
            }
            '\r' | '\n' => i += 1,
            _ => {
                if uc_skip > 0 {
                    uc_skip -= 1;
                } else if !skip {
                    out.push(c);
                }
                i += 1;
            }
        }
    }
    out.trim().to_string()
}

pub fn cp1252(b: u8) -> char {
    const HIGH: [char; 32] = [
        '\u{20AC}', '\u{81}', '\u{201A}', '\u{192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}', '\u{2C6}', '\u{2030}',
        '\u{160}', '\u{2039}', '\u{152}', '\u{8D}', '\u{17D}', '\u{8F}', '\u{90}', '\u{2018}', '\u{2019}', '\u{201C}',
        '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}', '\u{2DC}', '\u{2122}', '\u{161}', '\u{203A}', '\u{153}', '\u{9D}',
        '\u{17E}', '\u{178}',
    ];
    if (0x80..0xA0).contains(&b) {
        HIGH[(b - 0x80) as usize]
    } else {
        b as char
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fecha_y_hora() {
        // 02:00 = 1200 decimas de minuto; dia 15481 desde 1983-12-31 = 2026-05-20.
        assert_eq!(timestamp(&[0xB0, 0x04, 0x79, 0x3C], 0), Some(15481 * 86400 + 7200));
        assert_eq!(timestamp(&[0, 0, 0xFF, 0xFF], 0), None);
        assert_eq!(timestamp(&[0, 0, 0, 0], 0), None);
        assert_eq!(timestamp(&[0xB0, 0x04], 0), None);
    }

    #[test]
    fn duraciones() {
        let sc = Scale { minutes_per_day: 720.0, minutes_per_week: 2400.0, days_per_month: 31.0, default_units: Unit::Hours };
        assert_eq!(adjusted_duration(&sc, 14400, Unit::Hours), Some(Dur { value: 24.0, unit: Unit::Hours }));
        assert_eq!(adjusted_duration(&sc, 7200, Unit::Days), Some(Dur { value: 1.0, unit: Unit::Days }));
        assert_eq!(adjusted_duration(&sc, -1, Unit::Hours), None);
        assert_eq!(to_hours(&sc, Dur { value: 2.0, unit: Unit::Days }), Some(24.0));
        assert_eq!(duration_units(5, None), Unit::Hours);
        assert_eq!(duration_units(21, Some(Unit::Hours)), Unit::Hours);
    }

    #[test]
    fn cadenas() {
        assert_eq!(utf16(&[b'A', 0, 0xF1, 0, 0, 0, b'x', 0], 0), "Añ");
        assert_eq!(utf16(&[b'A', 0], 4), "");
        assert_eq!(u32_at(&[1, 2, 3], 0), 0);
        assert_eq!(rtf_to_text("{\\rtf1\\ansi\\deff0{\\fonttbl{\\f0 Arial;}}\\pard baj\\'e9 de 4 a 3.5\\par}"), "bajé de 4 a 3.5");
        assert_eq!(rtf_to_text("texto plano"), "texto plano");
    }
}
