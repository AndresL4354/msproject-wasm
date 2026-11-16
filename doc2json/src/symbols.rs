//! Fuentes de simbolos (Symbol, Wingdings): sus caracteres no son Unicode, sino codigos propios
//! que Word y los PDF guardan en el area de uso privado (U+F020..U+F0FF). Aqui se traducen a
//! los caracteres Unicode equivalentes para que las vinetas y simbolos se lean bien.

const SYMBOL: &[(u8, char)] = &[
    (0x22, '∀'), (0x24, '∃'), (0x27, '∋'), (0x2A, '∗'), (0x2D, '−'), (0x40, '≅'), (0x41, 'Α'), (0x42, 'Β'), (0x43, 'Χ'),
    (0x44, 'Δ'), (0x45, 'Ε'), (0x46, 'Φ'), (0x47, 'Γ'), (0x48, 'Η'), (0x49, 'Ι'), (0x4A, 'ϑ'), (0x4B, 'Κ'), (0x4C, 'Λ'),
    (0x4D, 'Μ'), (0x4E, 'Ν'), (0x4F, 'Ο'), (0x50, 'Π'), (0x51, 'Θ'), (0x52, 'Ρ'), (0x53, 'Σ'), (0x54, 'Τ'), (0x55, 'Υ'),
    (0x56, 'ς'), (0x57, 'Ω'), (0x58, 'Ξ'), (0x59, 'Ψ'), (0x5A, 'Ζ'), (0x5C, '∴'), (0x5E, '⊥'), (0x61, 'α'), (0x62, 'β'),
    (0x63, 'χ'), (0x64, 'δ'), (0x65, 'ε'), (0x66, 'φ'), (0x67, 'γ'), (0x68, 'η'), (0x69, 'ι'), (0x6A, 'ϕ'), (0x6B, 'κ'),
    (0x6C, 'λ'), (0x6D, 'μ'), (0x6E, 'ν'), (0x6F, 'ο'), (0x70, 'π'), (0x71, 'θ'), (0x72, 'ρ'), (0x73, 'σ'), (0x74, 'τ'),
    (0x75, 'υ'), (0x76, 'ϖ'), (0x77, 'ω'), (0x78, 'ξ'), (0x79, 'ψ'), (0x7A, 'ζ'), (0x7E, '∼'), (0xA0, '€'), (0xA1, 'ϒ'),
    (0xA2, '′'), (0xA3, '≤'), (0xA4, '⁄'), (0xA5, '∞'), (0xA6, 'ƒ'), (0xA7, '♣'), (0xA8, '♦'), (0xA9, '♥'), (0xAA, '♠'),
    (0xAB, '↔'), (0xAC, '←'), (0xAD, '↑'), (0xAE, '→'), (0xAF, '↓'), (0xB0, '°'), (0xB1, '±'), (0xB2, '″'), (0xB3, '≥'),
    (0xB4, '×'), (0xB5, '∝'), (0xB6, '∂'), (0xB7, '•'), (0xB8, '÷'), (0xB9, '≠'), (0xBA, '≡'), (0xBB, '≈'), (0xBC, '…'),
    (0xBF, '↵'), (0xC0, 'ℵ'), (0xC1, 'ℑ'), (0xC2, 'ℜ'), (0xC3, '℘'), (0xC4, '⊗'), (0xC5, '⊕'), (0xC6, '∅'), (0xC7, '∩'),
    (0xC8, '∪'), (0xC9, '⊃'), (0xCA, '⊇'), (0xCB, '⊄'), (0xCC, '⊂'), (0xCD, '⊆'), (0xCE, '∈'), (0xCF, '∉'), (0xD0, '∠'),
    (0xD1, '∇'), (0xD2, '®'), (0xD3, '©'), (0xD4, '™'), (0xD5, '∏'), (0xD6, '√'), (0xD7, '⋅'), (0xD8, '¬'), (0xD9, '∧'),
    (0xDA, '∨'), (0xDB, '⇔'), (0xDC, '⇐'), (0xDD, '⇑'), (0xDE, '⇒'), (0xDF, '⇓'), (0xE0, '◊'), (0xE1, '〈'), (0xE2, '®'),
    (0xE3, '©'), (0xE4, '™'), (0xE5, '∑'), (0xF1, '〉'), (0xF2, '∫'),
];

const WINGDINGS: &[(u8, char)] = &[
    (0x21, '✏'), (0x22, '✂'), (0x28, '☎'), (0x2A, '✉'), (0x36, '⌛'), (0x3F, '✍'), (0x41, '✌'), (0x43, '👍'), (0x44, '👎'),
    (0x45, '☜'), (0x46, '☞'), (0x47, '☝'), (0x48, '☟'), (0x4A, '☺'), (0x4C, '☹'), (0x4E, '☠'), (0x51, '✈'), (0x52, '☼'),
    (0x54, '❄'), (0x58, '✠'), (0x6C, '●'), (0x6D, '❍'), (0x6E, '■'), (0x6F, '□'), (0x71, '❑'), (0x72, '❒'), (0x73, '⬧'),
    (0x74, '⧫'), (0x75, '◆'), (0x76, '❖'), (0x77, '⬥'), (0x9E, '·'), (0x9F, '•'), (0xA0, '▪'), (0xA1, '○'), (0xA4, '◉'),
    (0xA5, '◎'), (0xA7, '▪'), (0xA8, '◻'), (0xAA, '✦'), (0xAB, '★'), (0xAC, '✶'), (0xB6, '✩'), (0xD5, '⌫'), (0xD6, '⌦'),
    (0xD8, '➢'), (0xDC, '➲'), (0xDF, '←'), (0xE0, '→'), (0xE1, '↑'), (0xE2, '↓'), (0xE8, '➔'), (0xEF, '⇦'), (0xF0, '⇨'),
    (0xF1, '⇧'), (0xF2, '⇩'), (0xFB, '✗'), (0xFC, '✓'), (0xFD, '☒'), (0xFE, '☑'),
];

fn lookup(table: &[(u8, char)], code: u32) -> Option<char> {
    let code = u8::try_from(code).ok()?;
    table.iter().find(|(c, _)| *c == code).map(|(_, ch)| *ch)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SymbolFont {
    None,
    Symbol,
    Wingdings,
    Other,
}

pub fn classify(font: &str) -> SymbolFont {
    let f = font.to_ascii_lowercase();
    if f.contains("wingdings") || f.contains("webdings") || f.contains("dingbats") {
        SymbolFont::Wingdings
    } else if f.contains("symbol") {
        SymbolFont::Symbol
    } else {
        SymbolFont::None
    }
}

/// Traduce un caracter de una fuente de simbolos a Unicode. `c` puede venir como codigo
/// directo (0x20..0xFF) o en el area de uso privado (U+F020..U+F0FF).
pub fn translate(font: SymbolFont, c: char) -> char {
    let cp = c as u32;
    let private = (0xF020..=0xF0FF).contains(&cp);
    let code = if private { cp - 0xF000 } else { cp };
    match font {
        SymbolFont::Symbol => lookup(SYMBOL, code).unwrap_or(if private || cp < 0x100 { char::from_u32(code).unwrap_or(c) } else { c }),
        SymbolFont::Wingdings => {
            if private || cp < 0x100 {
                // Un simbolo decorativo sin equivalente conocido se muestra como vineta.
                lookup(WINGDINGS, code).unwrap_or('•')
            } else {
                c
            }
        }
        // Fuente desconocida con codigos en el area privada: casi siempre es una vineta de
        // Symbol o una fuente "simbolica" cuyos codigos son los de Latin-1 desplazados.
        SymbolFont::Other | SymbolFont::None => {
            if !private {
                c
            } else if code == 0xB7 || code == 0xA7 {
                '•'
            } else {
                char::from_u32(code).filter(|ch| !ch.is_control()).unwrap_or('•')
            }
        }
    }
}

pub fn translate_str(font: SymbolFont, s: &str) -> String {
    if font == SymbolFont::None && !s.chars().any(|c| (0xF020..=0xF0FF).contains(&(c as u32))) {
        return s.to_string();
    }
    s.chars().map(|c| translate(font, c)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vinetas() {
        assert_eq!(translate(SymbolFont::Symbol, '\u{F0B7}'), '•');
        assert_eq!(translate(SymbolFont::Wingdings, '\u{F0A7}'), '▪');
        assert_eq!(translate(SymbolFont::Wingdings, '\u{F0FC}'), '✓');
        assert_eq!(translate(SymbolFont::None, '\u{F0B7}'), '•');
        assert_eq!(translate(SymbolFont::None, 'a'), 'a');
        assert_eq!(translate(SymbolFont::Symbol, 'a'), 'α');
        assert_eq!(classify("Wingdings 2"), SymbolFont::Wingdings);
        assert_eq!(translate_str(SymbolFont::None, "x \u{F0B7} y"), "x • y");
    }
}
