//! Fuentes de PDF: como pasar de los codigos de un texto a caracteres Unicode y a anchos.

use super::document::Pdf;
use super::encoding::{base_char, glyph_to_unicode, Base};
use super::object::{Dict, Lexer, Obj};
use crate::symbols::{classify, translate_str, SymbolFont};
use std::collections::HashMap;

// ---------------------------------------------------------------------------------------------
// CMap: tablas de correspondencia de codigos (ToUnicode y codificaciones de fuentes compuestas)
// ---------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct CMap {
    /// Rangos de codigos validos: (numero de bytes, minimo, maximo).
    pub spaces: Vec<(usize, u32, u32)>,
    chars: HashMap<u32, String>,
    ranges: Vec<(u32, u32, Vec<u16>)>,
    cid_chars: HashMap<u32, u32>,
    cid_ranges: Vec<(u32, u32, u32)>,
}

fn be(b: &[u8]) -> u32 {
    b.iter().take(4).fold(0u32, |acc, x| acc << 8 | *x as u32)
}

fn utf16be(b: &[u8]) -> Vec<u16> {
    b.chunks(2).map(|c| if c.len() == 2 { u16::from_be_bytes([c[0], c[1]]) } else { c[0] as u16 }).collect()
}

impl CMap {
    pub fn parse(data: &[u8]) -> CMap {
        let mut m = CMap::default();
        let mut lx = Lexer::new(data, 0);
        let mut args: Vec<Obj> = Vec::new();
        while let Some(o) = lx.object() {
            let op = match o {
                Obj::Op(op) => op,
                other => {
                    if args.len() < 100_000 {
                        args.push(other);
                    }
                    continue;
                }
            };
            match op.as_str() {
                "endcodespacerange" => {
                    for p in args.chunks(2) {
                        if let (Some(lo), Some(hi)) = (p[0].as_bytes(), p.get(1).and_then(|h| h.as_bytes())) {
                            if !lo.is_empty() && lo.len() <= 4 {
                                m.spaces.push((lo.len(), be(lo), be(hi)));
                            }
                        }
                    }
                }
                "endbfchar" => {
                    for p in args.chunks(2) {
                        if let (Some(src), Some(dst)) = (p[0].as_bytes(), p.get(1)) {
                            let text = match dst {
                                Obj::Str(s) => String::from_utf16_lossy(&utf16be(s)),
                                Obj::Name(n) => glyph_to_unicode(n).unwrap_or_default(),
                                _ => String::new(),
                            };
                            m.chars.insert(be(src), text);
                        }
                    }
                }
                "endbfrange" => {
                    for p in args.chunks(3) {
                        let (lo, hi) = match (p[0].as_bytes(), p.get(1).and_then(|h| h.as_bytes())) {
                            (Some(a), Some(b)) => (be(a), be(b)),
                            _ => continue,
                        };
                        match p.get(2) {
                            Some(Obj::Str(s)) => m.ranges.push((lo, hi, utf16be(s))),
                            Some(Obj::Array(a)) => {
                                for (i, item) in a.iter().enumerate() {
                                    if let Some(s) = item.as_bytes() {
                                        m.chars.insert(lo.wrapping_add(i as u32), String::from_utf16_lossy(&utf16be(s)));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "endcidchar" => {
                    for p in args.chunks(2) {
                        if let (Some(src), Some(cid)) = (p[0].as_bytes(), p.get(1).and_then(|c| c.as_i64())) {
                            m.cid_chars.insert(be(src), cid as u32);
                        }
                    }
                }
                "endcidrange" => {
                    for p in args.chunks(3) {
                        if let (Some(lo), Some(hi), Some(cid)) = (p[0].as_bytes(), p.get(1).and_then(|h| h.as_bytes()), p.get(2).and_then(|c| c.as_i64())) {
                            m.cid_ranges.push((be(lo), be(hi), cid as u32));
                        }
                    }
                }
                _ => {}
            }
            args.clear();
        }
        m
    }

    pub fn unicode(&self, code: u32) -> Option<String> {
        if let Some(s) = self.chars.get(&code) {
            return Some(s.clone());
        }
        for (lo, hi, dst) in &self.ranges {
            if code >= *lo && code <= *hi {
                let mut units = dst.clone();
                if let Some(last) = units.last_mut() {
                    *last = last.wrapping_add((code - lo) as u16);
                }
                return Some(String::from_utf16_lossy(&units));
            }
        }
        None
    }

    pub fn cid(&self, code: u32) -> Option<u32> {
        if let Some(c) = self.cid_chars.get(&code) {
            return Some(*c);
        }
        self.cid_ranges.iter().find(|(lo, hi, _)| code >= *lo && code <= *hi).map(|(lo, _, cid)| cid + (code - lo))
    }

    pub fn has_unicode(&self) -> bool {
        !self.chars.is_empty() || !self.ranges.is_empty()
    }
}

// ---------------------------------------------------------------------------------------------
// Anchos de las fuentes estandar (para PDF antiguos que no los declaran)
// ---------------------------------------------------------------------------------------------

/// Anchos de Helvetica para los codigos 32..126 (milesimas de em).
const HELVETICA: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278,
    584, 584, 584, 556, 1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667,
    944, 667, 667, 611, 278, 278, 278, 469, 556, 333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, 556, 556,
    333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584,
];

/// Anchos de Times-Roman para los codigos 32..126.
const TIMES: [u16; 95] = [
    250, 333, 408, 500, 500, 833, 778, 180, 333, 333, 500, 564, 250, 333, 250, 278, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 278, 278,
    564, 564, 564, 444, 921, 722, 667, 667, 722, 611, 556, 722, 722, 333, 389, 722, 611, 889, 722, 722, 556, 722, 667, 556, 611, 722, 722,
    944, 722, 722, 611, 333, 278, 333, 469, 500, 333, 444, 500, 444, 500, 444, 333, 500, 500, 278, 278, 500, 278, 778, 500, 500, 500, 500,
    333, 389, 278, 500, 500, 722, 500, 500, 444, 480, 200, 480, 541,
];

fn standard_widths(name: &str) -> [f64; 256] {
    let n = name.to_ascii_lowercase();
    let mut w = [0.0f64; 256];
    if n.contains("courier") || n.contains("mono") || n.contains("consolas") {
        return [600.0; 256];
    }
    let (table, other) = if n.contains("times") || n.contains("serif") && !n.contains("sans") || n.contains("georgia") || n.contains("garamond") {
        (&TIMES, 500.0)
    } else {
        (&HELVETICA, 556.0)
    };
    for (i, slot) in w.iter_mut().enumerate() {
        *slot = if (32..127).contains(&i) { table[i - 32] as f64 } else { other };
    }
    w
}

// ---------------------------------------------------------------------------------------------
// Fuente
// ---------------------------------------------------------------------------------------------

pub struct Font {
    pub name: String,
    pub composite: bool,
    pub bold: bool,
    /// Factor de espacio de glifo a espacio de texto (0.001, salvo en fuentes Type3).
    pub scale: f64,
    /// Fuentes simples: texto y ancho de cada codigo.
    table: Vec<String>,
    widths: Vec<f64>,
    /// Fuentes compuestas.
    to_unicode: Option<CMap>,
    enc: Option<CMap>,
    unicode_codes: bool,
    cid_widths: HashMap<u32, f64>,
    default_width: f64,
    symbol: SymbolFont,
}

impl Font {
    pub fn load(pdf: &Pdf, dict: &Dict) -> Font {
        let subtype = pdf.name(dict, "Subtype").unwrap_or_default();
        let raw_name = pdf.name(dict, "BaseFont").or_else(|| pdf.name(dict, "Name")).unwrap_or_default();
        // Los subconjuntos llevan un prefijo "ABCDEF+".
        let name = match raw_name.split_once('+') {
            Some((p, rest)) if p.len() == 6 && p.bytes().all(|b| b.is_ascii_uppercase()) => rest.to_string(),
            _ => raw_name,
        };
        let to_unicode = pdf.get(dict, "ToUnicode");
        let to_unicode = pdf.stream_data(&to_unicode).map(|d| CMap::parse(&d)).filter(|m| m.has_unicode());
        let symbol = classify(&name);

        let mut font = Font {
            name: name.clone(),
            composite: subtype == "Type0",
            bold: false,
            scale: 0.001,
            table: Vec::new(),
            widths: Vec::new(),
            to_unicode: None,
            enc: None,
            unicode_codes: false,
            cid_widths: HashMap::new(),
            default_width: 1000.0,
            symbol,
        };

        let mut descriptor = pdf.dict(dict, "FontDescriptor");
        if font.composite {
            let descendant = pdf.array(dict, "DescendantFonts").and_then(|a| a.first().cloned()).and_then(|o| o.as_dict().cloned()).unwrap_or_default();
            if descriptor.is_none() {
                descriptor = pdf.dict(&descendant, "FontDescriptor");
            }
            font.default_width = pdf.num(&descendant, "DW").unwrap_or(1000.0);
            // /W: c [w1 w2 ...]  |  c_primero c_ultimo w
            if let Some(w) = pdf.array(&descendant, "W") {
                let mut i = 0usize;
                while i < w.len() {
                    let first = w[i].as_i64().unwrap_or(0).max(0) as u32;
                    match w.get(i + 1).map(|o| &**o) {
                        Some(Obj::Array(list)) => {
                            for (k, v) in list.iter().enumerate() {
                                if let Some(x) = pdf.resolve(v).as_f64() {
                                    font.cid_widths.insert(first + k as u32, x);
                                }
                            }
                            i += 2;
                        }
                        Some(last) => {
                            let last = last.as_i64().unwrap_or(0).max(0) as u32;
                            if let Some(x) = w.get(i + 2).and_then(|o| o.as_f64()) {
                                for c in first..=last.min(first.saturating_add(70_000)) {
                                    font.cid_widths.insert(c, x);
                                }
                            }
                            i += 3;
                        }
                        None => break,
                    }
                }
            }
            let encoding = pdf.get(dict, "Encoding");
            match &*encoding {
                Obj::Name(n) => {
                    // Los CMap predefinidos "…-UCS2-…" y "…-UTF16-…" usan Unicode como codigo.
                    font.unicode_codes = n.contains("UCS2") || n.contains("UTF16");
                }
                o @ Obj::Stream { .. } => font.enc = pdf.stream_data(o).map(|d| CMap::parse(&d)),
                _ => {}
            }
            font.to_unicode = to_unicode;
        } else {
            // Fuente simple: 256 codigos.
            let is_type3 = subtype == "Type3";
            if is_type3 {
                if let Some(m) = pdf.array(dict, "FontMatrix") {
                    font.scale = m.first().and_then(|v| v.as_f64()).filter(|v| v.abs() > 1e-9).unwrap_or(0.001).abs();
                }
            }
            let flags = descriptor.as_ref().and_then(|d| pdf.get(d, "Flags").as_i64()).unwrap_or(0);
            let symbolic = flags & 4 != 0 && flags & 32 == 0;
            let encoding = pdf.get(dict, "Encoding");
            let (base_name, differences) = match &*encoding {
                Obj::Name(n) => (Some(n.clone()), None),
                Obj::Dict(d) => (pdf.name(d, "BaseEncoding"), pdf.array(d, "Differences")),
                _ => (None, None),
            };
            let base = match base_name.as_deref() {
                Some("WinAnsiEncoding") => Base::WinAnsi,
                Some("MacRomanEncoding") => Base::MacRoman,
                Some("StandardEncoding") => Base::Standard,
                // Sin codificacion declarada: TrueType suele usar WinAnsi; Type1, Standard.
                _ => {
                    if subtype == "TrueType" || symbolic {
                        Base::WinAnsi
                    } else {
                        Base::Standard
                    }
                }
            };
            let mut names: Vec<Option<String>> = vec![None; 256];
            if let Some(diff) = differences {
                let mut code = 0usize;
                for item in diff {
                    match &*item {
                        Obj::Int(n) => code = (*n).clamp(0, 255) as usize,
                        Obj::Name(n) => {
                            if code < 256 {
                                names[code] = Some(n.clone());
                            }
                            code += 1;
                        }
                        _ => {}
                    }
                }
            }
            let sym = if symbol == SymbolFont::None && symbolic && base_name.is_none() { SymbolFont::Other } else { symbol };
            font.table = (0..256usize)
                .map(|c| {
                    let from_cmap = to_unicode.as_ref().and_then(|m| m.unicode(c as u32)).filter(|s| !s.is_empty());
                    let text = from_cmap
                        .or_else(|| names[c].as_deref().and_then(glyph_to_unicode))
                        .or_else(|| {
                            // Fuente de simbolos sin tabla propia: el codigo se traduce directamente.
                            if sym == SymbolFont::Symbol || sym == SymbolFont::Wingdings {
                                char::from_u32(c as u32).filter(|_| c >= 0x20).map(|ch| translate_str(sym, &ch.to_string()))
                            } else {
                                base_char(base, c as u8).map(|ch| ch.to_string())
                            }
                        })
                        .unwrap_or_default();
                    translate_str(if sym == SymbolFont::None { SymbolFont::None } else { sym }, &text)
                })
                .collect();

            // Anchos.
            let missing = descriptor.as_ref().and_then(|d| pdf.num(d, "MissingWidth")).unwrap_or(0.0);
            match pdf.array(dict, "Widths") {
                Some(w) if !w.is_empty() => {
                    let first = pdf.get(dict, "FirstChar").as_i64().unwrap_or(0).max(0) as usize;
                    let fallback = if missing > 0.0 { missing } else if is_type3 { 0.0 } else { 500.0 };
                    font.widths = vec![fallback; 256];
                    for (i, v) in w.iter().enumerate() {
                        if first + i < 256 {
                            let x = v.as_f64().unwrap_or(0.0);
                            font.widths[first + i] = if x == 0.0 && !is_type3 { fallback } else { x };
                        }
                    }
                }
                _ => font.widths = standard_widths(&name).to_vec(),
            }
        }

        let weight = descriptor.as_ref().and_then(|d| pdf.num(d, "FontWeight")).unwrap_or(0.0);
        let flags = descriptor.as_ref().and_then(|d| pdf.get(d, "Flags").as_i64()).unwrap_or(0);
        let lower = name.to_ascii_lowercase();
        font.bold = weight >= 600.0 || flags & 0x40000 != 0 || ["bold", "black", "heavy", "semibold", "demi"].iter().any(|k| lower.contains(k));
        font
    }

    /// Recorre los glifos de una cadena: llama `f(texto, ancho, es_espacio_de_un_byte)`.
    /// El ancho ya esta en espacio de texto (para tamano de fuente 1).
    pub fn decode(&self, bytes: &[u8], mut f: impl FnMut(&str, f64, bool)) {
        if !self.composite {
            for &b in bytes {
                f(&self.table[b as usize], self.widths[b as usize] * self.scale, b == 32);
            }
            return;
        }
        let mut i = 0usize;
        while i < bytes.len() {
            // Cuantos bytes ocupa este codigo segun los rangos declarados (por defecto, 2).
            let mut len = 2usize.min(bytes.len() - i);
            // Solo cuenta la codificacion propia de la fuente; con "Identity-H" siempre son 2 bytes,
            // diga lo que diga la tabla ToUnicode.
            let spaces: &[(usize, u32, u32)] = match &self.enc {
                Some(e) if !e.spaces.is_empty() => &e.spaces,
                _ => &[],
            };
            for n in 1..=4usize.min(bytes.len() - i) {
                let code = be(&bytes[i..i + n]);
                if spaces.iter().any(|(sz, lo, hi)| *sz == n && code >= *lo && code <= *hi) {
                    len = n;
                    break;
                }
            }
            let code = be(&bytes[i..i + len]);
            i += len;
            let cid = self.enc.as_ref().and_then(|e| e.cid(code)).unwrap_or(code);
            let width = self.cid_widths.get(&cid).copied().unwrap_or(self.default_width) * self.scale;
            let text = self
                .to_unicode
                .as_ref()
                .and_then(|m| m.unicode(code))
                .or_else(|| if self.unicode_codes { char::from_u32(code).map(|c| c.to_string()) } else { None })
                // Sin tabla Unicode no hay forma de saber que letra es.
                .unwrap_or_else(|| "\u{FFFD}".to_string());
            let text = translate_str(self.symbol, &text);
            f(&text, width, len == 1 && code == 32);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmap_to_unicode() {
        let src = b"/CIDInit /ProcSet findresource begin 12 dict begin begincmap
1 begincodespacerange <0000> <FFFF> endcodespacerange
2 beginbfchar <0003> <0020> <0024> <00660069> endbfchar
2 beginbfrange <0041> <0043> <0061> <0050> <0051> [<00F1> <00E9>] endbfrange
endcmap";
        let m = CMap::parse(src);
        assert_eq!(m.spaces, vec![(2, 0, 0xFFFF)]);
        assert_eq!(m.unicode(3).as_deref(), Some(" "));
        assert_eq!(m.unicode(0x24).as_deref(), Some("fi"));
        assert_eq!(m.unicode(0x42).as_deref(), Some("b"));
        assert_eq!(m.unicode(0x51).as_deref(), Some("\u{e9}"));
        assert_eq!(m.unicode(0x99), None);
    }

    #[test]
    fn anchos_estandar() {
        assert_eq!(standard_widths("Helvetica-Bold")[b'A' as usize], 667.0);
        assert_eq!(standard_widths("Times-Roman")[b' ' as usize], 250.0);
        assert_eq!(standard_widths("Courier")[b'i' as usize], 600.0);
    }
}
