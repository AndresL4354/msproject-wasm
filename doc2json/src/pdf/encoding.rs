//! Codificaciones de caracteres de PDF y nombres de glifos.
//!
//! En un PDF el texto se guarda como codigos de la fuente, no como Unicode. Cuando la fuente
//! no trae su tabla `ToUnicode`, el caracter se deduce de la codificacion (WinAnsi, MacRoman,
//! Standard) y de los nombres de glifo ("aacute", "ntilde", "quotedblleft"...).

const CP1252_HIGH: [char; 32] = [
    '\u{20AC}', '\u{2022}', '\u{201A}', '\u{192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}', '\u{2C6}', '\u{2030}', '\u{160}', '\u{2039}',
    '\u{152}', '\u{2022}', '\u{17D}', '\u{2022}', '\u{2022}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{2DC}', '\u{2122}', '\u{161}', '\u{203A}', '\u{153}', '\u{2022}', '\u{17E}', '\u{178}',
];

const MAC_HIGH: [char; 128] = [
    'Ä', 'Å', 'Ç', 'É', 'Ñ', 'Ö', 'Ü', 'á', 'à', 'â', 'ä', 'ã', 'å', 'ç', 'é', 'è', 'ê', 'ë', 'í', 'ì', 'î', 'ï', 'ñ', 'ó', 'ò', 'ô', 'ö', 'õ',
    'ú', 'ù', 'û', 'ü', '†', '°', '¢', '£', '§', '•', '¶', 'ß', '®', '©', '™', '´', '¨', '≠', 'Æ', 'Ø', '∞', '±', '≤', '≥', '¥', 'µ', '∂', '∑',
    '∏', 'π', '∫', 'ª', 'º', 'Ω', 'æ', 'ø', '¿', '¡', '¬', '√', 'ƒ', '≈', '∆', '«', '»', '…', ' ', 'À', 'Ã', 'Õ', 'Œ', 'œ', '–', '—', '“', '”',
    '‘', '’', '÷', '◊', 'ÿ', 'Ÿ', '⁄', '¤', '‹', '›', 'ﬁ', 'ﬂ', '‡', '·', '‚', '„', '‰', 'Â', 'Ê', 'Á', 'Ë', 'È', 'Í', 'Î', 'Ï', 'Ì', 'Ó', 'Ô',
    '\u{F8FF}', 'Ò', 'Ú', 'Û', 'Ù', 'ı', 'ˆ', '˜', '¯', '˘', '˙', '˚', '¸', '˝', '˛', 'ˇ',
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Base {
    Standard,
    WinAnsi,
    MacRoman,
}

pub fn base_char(base: Base, c: u8) -> Option<char> {
    match base {
        Base::WinAnsi => match c {
            0x20..=0x7E => Some(c as char),
            0x7F => Some('\u{2022}'),
            0x80..=0x9F => Some(CP1252_HIGH[(c - 0x80) as usize]),
            0xA0 => Some(' '),
            0xAD => Some('-'),
            0xA1..=0xFF => Some(c as char),
            _ => None,
        },
        Base::MacRoman => match c {
            0x20..=0x7E => Some(c as char),
            0x80..=0xFF => Some(MAC_HIGH[(c - 0x80) as usize]),
            _ => None,
        },
        Base::Standard => match c {
            0x27 => Some('\u{2019}'),
            0x60 => Some('\u{2018}'),
            0x20..=0x7E => Some(c as char),
            0xA1 => Some('¡'),
            0xA2 => Some('¢'),
            0xA3 => Some('£'),
            0xA4 => Some('\u{2044}'),
            0xA5 => Some('¥'),
            0xA6 => Some('\u{192}'),
            0xA7 => Some('§'),
            0xA8 => Some('¤'),
            0xA9 => Some('\''),
            0xAA => Some('\u{201C}'),
            0xAB => Some('«'),
            0xAC => Some('\u{2039}'),
            0xAD => Some('\u{203A}'),
            0xAE => Some('\u{FB01}'),
            0xAF => Some('\u{FB02}'),
            0xB1 => Some('\u{2013}'),
            0xB2 => Some('\u{2020}'),
            0xB3 => Some('\u{2021}'),
            0xB4 => Some('·'),
            0xB6 => Some('¶'),
            0xB7 => Some('\u{2022}'),
            0xB8 => Some('\u{201A}'),
            0xB9 => Some('\u{201E}'),
            0xBA => Some('\u{201D}'),
            0xBB => Some('»'),
            0xBC => Some('\u{2026}'),
            0xBD => Some('\u{2030}'),
            0xBF => Some('¿'),
            0xC1 => Some('`'),
            0xC2 => Some('´'),
            0xC3 => Some('\u{2C6}'),
            0xC4 => Some('\u{2DC}'),
            0xC5 => Some('¯'),
            0xC6 => Some('\u{2D8}'),
            0xC7 => Some('\u{2D9}'),
            0xC8 => Some('¨'),
            0xCA => Some('\u{2DA}'),
            0xCB => Some('¸'),
            0xCD => Some('\u{2DD}'),
            0xCE => Some('\u{2DB}'),
            0xCF => Some('\u{2C7}'),
            0xD0 => Some('\u{2014}'),
            0xE1 => Some('Æ'),
            0xE3 => Some('ª'),
            0xE8 => Some('\u{141}'),
            0xE9 => Some('Ø'),
            0xEA => Some('\u{152}'),
            0xEB => Some('º'),
            0xF1 => Some('æ'),
            0xF5 => Some('\u{131}'),
            0xF8 => Some('\u{142}'),
            0xF9 => Some('ø'),
            0xFA => Some('\u{153}'),
            0xFB => Some('ß'),
            _ => None,
        },
    }
}

/// Nombres de glifo de U+00A1 a U+00FF, en orden.
const LATIN1: &[&str] = &[
    "exclamdown", "cent", "sterling", "currency", "yen", "brokenbar", "section", "dieresis", "copyright", "ordfeminine", "guillemotleft",
    "logicalnot", "sfthyphen", "registered", "macron", "degree", "plusminus", "twosuperior", "threesuperior", "acute", "mu", "paragraph",
    "periodcentered", "cedilla", "onesuperior", "ordmasculine", "guillemotright", "onequarter", "onehalf", "threequarters", "questiondown",
    "Agrave", "Aacute", "Acircumflex", "Atilde", "Adieresis", "Aring", "AE", "Ccedilla", "Egrave", "Eacute", "Ecircumflex", "Edieresis",
    "Igrave", "Iacute", "Icircumflex", "Idieresis", "Eth", "Ntilde", "Ograve", "Oacute", "Ocircumflex", "Otilde", "Odieresis", "multiply",
    "Oslash", "Ugrave", "Uacute", "Ucircumflex", "Udieresis", "Yacute", "Thorn", "germandbls", "agrave", "aacute", "acircumflex", "atilde",
    "adieresis", "aring", "ae", "ccedilla", "egrave", "eacute", "ecircumflex", "edieresis", "igrave", "iacute", "icircumflex", "idieresis",
    "eth", "ntilde", "ograve", "oacute", "ocircumflex", "otilde", "odieresis", "divide", "oslash", "ugrave", "uacute", "ucircumflex",
    "udieresis", "yacute", "thorn", "ydieresis",
];

/// Nombres de la puntuacion ASCII (las letras y digitos se resuelven aparte).
const ASCII: &[(&str, char)] = &[
    ("space", ' '), ("exclam", '!'), ("quotedbl", '"'), ("numbersign", '#'), ("dollar", '$'), ("percent", '%'), ("ampersand", '&'),
    ("quotesingle", '\''), ("parenleft", '('), ("parenright", ')'), ("asterisk", '*'), ("plus", '+'), ("comma", ','), ("hyphen", '-'),
    ("period", '.'), ("slash", '/'), ("zero", '0'), ("one", '1'), ("two", '2'), ("three", '3'), ("four", '4'), ("five", '5'), ("six", '6'),
    ("seven", '7'), ("eight", '8'), ("nine", '9'), ("colon", ':'), ("semicolon", ';'), ("less", '<'), ("equal", '='), ("greater", '>'),
    ("question", '?'), ("at", '@'), ("bracketleft", '['), ("backslash", '\\'), ("bracketright", ']'), ("asciicircum", '^'),
    ("underscore", '_'), ("grave", '`'), ("braceleft", '{'), ("bar", '|'), ("braceright", '}'), ("asciitilde", '~'),
];

const OTHER: &[(&str, &str)] = &[
    ("nbspace", " "), ("nonbreakingspace", " "), ("quoteleft", "\u{2018}"), ("quoteright", "\u{2019}"), ("quotesinglbase", "\u{201A}"),
    ("quotedblleft", "\u{201C}"), ("quotedblright", "\u{201D}"), ("quotedblbase", "\u{201E}"), ("endash", "\u{2013}"), ("emdash", "\u{2014}"),
    ("bullet", "\u{2022}"), ("ellipsis", "\u{2026}"), ("dagger", "\u{2020}"), ("daggerdbl", "\u{2021}"), ("perthousand", "\u{2030}"),
    ("guilsinglleft", "\u{2039}"), ("guilsinglright", "\u{203A}"), ("Euro", "\u{20AC}"), ("euro", "\u{20AC}"), ("trademark", "\u{2122}"),
    ("florin", "\u{192}"), ("fraction", "\u{2044}"), ("minus", "\u{2212}"), ("fi", "fi"), ("fl", "fl"), ("ff", "ff"), ("ffi", "ffi"),
    ("ffl", "ffl"), ("dotlessi", "\u{131}"), ("dotlessj", "\u{237}"), ("Lslash", "\u{141}"), ("lslash", "\u{142}"), ("OE", "\u{152}"),
    ("oe", "\u{153}"), ("Scaron", "\u{160}"), ("scaron", "\u{161}"), ("Zcaron", "\u{17D}"), ("zcaron", "\u{17E}"), ("Ydieresis", "\u{178}"),
    ("circumflex", "\u{2C6}"), ("tilde", "\u{2DC}"), ("breve", "\u{2D8}"), ("dotaccent", "\u{2D9}"), ("ring", "\u{2DA}"),
    ("ogonek", "\u{2DB}"), ("caron", "\u{2C7}"), ("hungarumlaut", "\u{2DD}"), ("softhyphen", "-"), ("apostrophe", "'"),
    ("Aogonek", "Ą"), ("aogonek", "ą"), ("Abreve", "Ă"), ("abreve", "ă"), ("Amacron", "Ā"), ("amacron", "ā"), ("Cacute", "Ć"),
    ("cacute", "ć"), ("Ccaron", "Č"), ("ccaron", "č"), ("Dcaron", "Ď"), ("dcaron", "ď"), ("Dcroat", "Đ"), ("dcroat", "đ"),
    ("Eogonek", "Ę"), ("eogonek", "ę"), ("Ecaron", "Ě"), ("ecaron", "ě"), ("Emacron", "Ē"), ("emacron", "ē"), ("Gbreve", "Ğ"),
    ("gbreve", "ğ"), ("Idotaccent", "İ"), ("Imacron", "Ī"), ("imacron", "ī"), ("Nacute", "Ń"), ("nacute", "ń"), ("Ncaron", "Ň"),
    ("ncaron", "ň"), ("Ohungarumlaut", "Ő"), ("ohungarumlaut", "ő"), ("Omacron", "Ō"), ("omacron", "ō"), ("Racute", "Ŕ"),
    ("racute", "ŕ"), ("Rcaron", "Ř"), ("rcaron", "ř"), ("Sacute", "Ś"), ("sacute", "ś"), ("Scedilla", "Ş"), ("scedilla", "ş"),
    ("Tcaron", "Ť"), ("tcaron", "ť"), ("Uring", "Ů"), ("uring", "ů"), ("Uhungarumlaut", "Ű"), ("uhungarumlaut", "ű"),
    ("Umacron", "Ū"), ("umacron", "ū"), ("Zacute", "Ź"), ("zacute", "ź"), ("Zdotaccent", "Ż"), ("zdotaccent", "ż"),
    ("Alpha", "Α"), ("Beta", "Β"), ("Gamma", "Γ"), ("Delta", "Δ"), ("Epsilon", "Ε"), ("Zeta", "Ζ"), ("Eta", "Η"), ("Theta", "Θ"),
    ("Iota", "Ι"), ("Kappa", "Κ"), ("Lambda", "Λ"), ("Mu", "Μ"), ("Nu", "Ν"), ("Xi", "Ξ"), ("Omicron", "Ο"), ("Pi", "Π"), ("Rho", "Ρ"),
    ("Sigma", "Σ"), ("Tau", "Τ"), ("Upsilon", "Υ"), ("Phi", "Φ"), ("Chi", "Χ"), ("Psi", "Ψ"), ("Omega", "Ω"), ("alpha", "α"),
    ("beta", "β"), ("gamma", "γ"), ("delta", "δ"), ("epsilon", "ε"), ("zeta", "ζ"), ("eta", "η"), ("theta", "θ"), ("iota", "ι"),
    ("kappa", "κ"), ("lambda", "λ"), ("nu", "ν"), ("xi", "ξ"), ("omicron", "ο"), ("pi", "π"), ("rho", "ρ"), ("sigma", "σ"),
    ("sigma1", "ς"), ("tau", "τ"), ("upsilon", "υ"), ("phi", "φ"), ("chi", "χ"), ("psi", "ψ"), ("omega", "ω"), ("theta1", "ϑ"),
    ("phi1", "ϕ"), ("omega1", "ϖ"), ("infinity", "∞"), ("notequal", "≠"), ("lessequal", "≤"), ("greaterequal", "≥"),
    ("approxequal", "≈"), ("partialdiff", "∂"), ("summation", "∑"), ("product", "∏"), ("integral", "∫"), ("radical", "√"),
    ("lozenge", "◊"), ("arrowleft", "←"), ("arrowup", "↑"), ("arrowright", "→"), ("arrowdown", "↓"), ("arrowboth", "↔"),
    ("minute", "′"), ("second", "″"), ("proportional", "∝"), ("equivalence", "≡"), ("element", "∈"), ("logicaland", "∧"),
    ("logicalor", "∨"), ("intersection", "∩"), ("union", "∪"), ("therefore", "∴"), ("similar", "∼"), ("angle", "∠"),
    ("perpendicular", "⊥"), ("emptyset", "∅"), ("checkmark", "✓"), ("square", "□"), ("filledbox", "■"), ("circle", "○"),
    ("H22073", "□"), ("H18533", "●"), ("H18543", "▪"), ("H18551", "▫"), ("openbullet", "◦"), ("heart", "♥"), ("club", "♣"),
    ("diamond", "♦"), ("spade", "♠"), ("visiblespace", " "), ("Ohm", "Ω"), ("Omegagreek", "Ω"), ("increment", "∆"),
    ("mu1", "µ"), ("micro", "µ"),
];

/// Convierte un nombre de glifo en su texto Unicode.
pub fn glyph_to_unicode(name: &str) -> Option<String> {
    // Variantes ("a.sc", "one.oldstyle") comparten el caracter base.
    let base = name.split('.').next().unwrap_or("");
    if base.is_empty() {
        return None;
    }
    // Ligaduras descritas por sus componentes: "f_f_i".
    if base.contains('_') {
        let parts: Vec<String> = base.split('_').filter_map(glyph_to_unicode).collect();
        return if parts.is_empty() { None } else { Some(parts.concat()) };
    }
    let mut chars = base.chars();
    if let (Some(c), None) = (chars.next(), chars.next()) {
        return Some(c.to_string());
    }
    // "uni00F1", "uni00480069" (varios caracteres) y "u1F600".
    if let Some(hex) = base.strip_prefix("uni") {
        if hex.len() >= 4 && hex.len() % 4 == 0 && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            let units: Vec<u16> = (0..hex.len() / 4).filter_map(|i| u16::from_str_radix(&hex[i * 4..i * 4 + 4], 16).ok()).collect();
            let s = String::from_utf16_lossy(&units);
            return if s.is_empty() { None } else { Some(s) };
        }
    }
    if let Some(hex) = base.strip_prefix('u') {
        if (4..=6).contains(&hex.len()) && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return u32::from_str_radix(hex, 16).ok().and_then(char::from_u32).map(|c| c.to_string());
        }
    }
    if let Some((_, c)) = ASCII.iter().find(|(n, _)| *n == base) {
        return Some(c.to_string());
    }
    if let Some(i) = LATIN1.iter().position(|n| *n == base) {
        return match base {
            "sfthyphen" => Some("-".into()),
            _ => char::from_u32(0xA1 + i as u32).map(|c| c.to_string()),
        };
    }
    OTHER.iter().find(|(n, _)| *n == base).map(|(_, s)| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nombres_de_glifo() {
        assert_eq!(glyph_to_unicode("ntilde").as_deref(), Some("ñ"));
        assert_eq!(glyph_to_unicode("Aacute").as_deref(), Some("Á"));
        assert_eq!(glyph_to_unicode("questiondown").as_deref(), Some("¿"));
        assert_eq!(glyph_to_unicode("ydieresis").as_deref(), Some("ÿ"));
        assert_eq!(glyph_to_unicode("quotedblleft").as_deref(), Some("\u{201C}"));
        assert_eq!(glyph_to_unicode("A").as_deref(), Some("A"));
        assert_eq!(glyph_to_unicode("three").as_deref(), Some("3"));
        assert_eq!(glyph_to_unicode("uni00F1").as_deref(), Some("ñ"));
        assert_eq!(glyph_to_unicode("u00E9").as_deref(), Some("é"));
        assert_eq!(glyph_to_unicode("f_f_i").as_deref(), Some("ffi"));
        assert_eq!(glyph_to_unicode("a.sc").as_deref(), Some("a"));
        assert_eq!(glyph_to_unicode("g123"), None);
        assert_eq!(glyph_to_unicode(".notdef"), None);
    }

    #[test]
    fn codificaciones_base() {
        assert_eq!(base_char(Base::WinAnsi, 0xF1), Some('ñ'));
        assert_eq!(base_char(Base::WinAnsi, 0x93), Some('\u{201C}'));
        assert_eq!(base_char(Base::WinAnsi, 0x80), Some('\u{20AC}'));
        assert_eq!(base_char(Base::MacRoman, 0x96), Some('ñ'));
        assert_eq!(base_char(Base::MacRoman, 0x87), Some('á'));
        assert_eq!(base_char(Base::Standard, 0xD0), Some('\u{2014}'));
        assert_eq!(base_char(Base::Standard, 0x41), Some('A'));
    }
}
