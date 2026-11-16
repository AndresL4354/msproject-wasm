//! Objetos de PDF y su analizador sintactico (ISO 32000-1, seccion 7.3).

#[derive(Clone, Debug, PartialEq)]
pub enum Obj {
    Null,
    Bool(bool),
    Int(i64),
    Real(f64),
    Name(String),
    Str(Vec<u8>),
    Array(Vec<Obj>),
    Dict(Dict),
    /// Flujo de datos: diccionario mas la posicion de los datos dentro del archivo.
    Stream { dict: Dict, start: usize, len: usize },
    Ref(u32, u16),
    /// Palabra clave u operador de un flujo de contenido (`BT`, `Tj`, `obj`...).
    Op(String),
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Dict(pub Vec<(String, Obj)>);

impl Dict {
    pub fn get(&self, key: &str) -> Option<&Obj> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn has(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}

impl Obj {
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Obj::Int(v) => Some(*v),
            Obj::Real(v) => Some(*v as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Obj::Int(v) => Some(*v as f64),
            Obj::Real(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_name(&self) -> Option<&str> {
        match self {
            Obj::Name(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_dict(&self) -> Option<&Dict> {
        match self {
            Obj::Dict(d) => Some(d),
            Obj::Stream { dict, .. } => Some(dict),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Obj]> {
        match self {
            Obj::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Obj::Str(s) => Some(s),
            _ => None,
        }
    }
}

pub fn is_ws(b: u8) -> bool {
    matches!(b, 0 | 9 | 10 | 12 | 13 | 32)
}

pub fn is_delim(b: u8) -> bool {
    matches!(b, b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%')
}

pub struct Lexer<'a> {
    pub d: &'a [u8],
    pub pos: usize,
    /// Profundidad de anidamiento de arreglos y diccionarios en curso.
    depth: u32,
}

/// Tope de anidamiento: un archivo malicioso no debe poder agotar la pila.
const MAX_DEPTH: u32 = 100;

impl<'a> Lexer<'a> {
    pub fn new(d: &'a [u8], pos: usize) -> Lexer<'a> {
        Lexer { d, pos, depth: 0 }
    }

    pub fn peek(&self) -> Option<u8> {
        self.d.get(self.pos).copied()
    }

    pub fn starts_with(&self, s: &[u8]) -> bool {
        self.d.get(self.pos..self.pos + s.len()) == Some(s)
    }

    /// Salta espacios en blanco y comentarios.
    pub fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            if is_ws(b) {
                self.pos += 1;
            } else if b == b'%' {
                while let Some(c) = self.peek() {
                    if c == b'\n' || c == b'\r' {
                        break;
                    }
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    /// Lee el siguiente objeto. `None` solo al llegar al final de los datos. Un delimitador de
    /// cierre suelto se devuelve como operador para que quien llama decida que hacer.
    pub fn object(&mut self) -> Option<Obj> {
        self.skip_ws();
        let b = self.peek()?;
        Some(match b {
            b'/' => {
                self.pos += 1;
                Obj::Name(self.name())
            }
            b'(' => Obj::Str(self.literal_string()),
            b'<' => {
                if self.d.get(self.pos + 1) == Some(&b'<') {
                    self.pos += 2;
                    if self.depth >= MAX_DEPTH {
                        return Some(Obj::Null);
                    }
                    self.depth += 1;
                    let d = self.dict();
                    self.depth -= 1;
                    Obj::Dict(d)
                } else {
                    self.pos += 1;
                    Obj::Str(self.hex_string())
                }
            }
            b'[' => {
                self.pos += 1;
                if self.depth >= MAX_DEPTH {
                    return Some(Obj::Null);
                }
                self.depth += 1;
                let mut items = Vec::new();
                loop {
                    self.skip_ws();
                    match self.peek() {
                        None => break,
                        Some(b']') => {
                            self.pos += 1;
                            break;
                        }
                        _ => {}
                    }
                    match self.object() {
                        // Un arreglo sin cerrar no debe tragarse el resto del archivo.
                        Some(Obj::Op(op)) if op == ">>" || op == "endobj" || op == "stream" => break,
                        Some(o) => items.push(o),
                        None => break,
                    }
                }
                self.depth -= 1;
                Obj::Array(items)
            }
            b'>' => {
                self.pos += if self.d.get(self.pos + 1) == Some(&b'>') { 2 } else { 1 };
                Obj::Op(">>".into())
            }
            b']' | b')' | b'{' | b'}' => {
                self.pos += 1;
                Obj::Op((b as char).to_string())
            }
            b'+' | b'-' | b'.' | b'0'..=b'9' => self.number(),
            _ => {
                let start = self.pos;
                while let Some(c) = self.peek() {
                    if is_ws(c) || is_delim(c) {
                        break;
                    }
                    self.pos += 1;
                }
                if self.pos == start {
                    self.pos += 1;
                }
                match &self.d[start..self.pos] {
                    b"true" => Obj::Bool(true),
                    b"false" => Obj::Bool(false),
                    b"null" => Obj::Null,
                    w => Obj::Op(String::from_utf8_lossy(w).into_owned()),
                }
            }
        })
    }

    fn name(&mut self) -> String {
        let mut out = Vec::new();
        while let Some(c) = self.peek() {
            if is_ws(c) || is_delim(c) {
                break;
            }
            self.pos += 1;
            if c == b'#' {
                let hex = self.d.get(self.pos..self.pos + 2).and_then(|h| std::str::from_utf8(h).ok()).and_then(|h| u8::from_str_radix(h, 16).ok());
                if let Some(v) = hex {
                    out.push(v);
                    self.pos += 2;
                    continue;
                }
            }
            out.push(c);
        }
        String::from_utf8_lossy(&out).into_owned()
    }

    fn number(&mut self) -> Obj {
        let start = self.pos;
        let mut real = false;
        while let Some(c) = self.peek() {
            match c {
                b'0'..=b'9' | b'+' | b'-' => {}
                b'.' => real = true,
                _ => break,
            }
            self.pos += 1;
        }
        let raw = std::str::from_utf8(&self.d[start..self.pos]).unwrap_or("0");
        // Algunos generadores escriben signos repetidos ("--5"): se tolera.
        let negative = raw.starts_with('-');
        let digits = raw.trim_start_matches(['+', '-']);
        if real {
            let v: f64 = digits.parse().unwrap_or(0.0);
            return Obj::Real(if negative { -v } else { v });
        }
        let v: i64 = match digits.parse() {
            Ok(v) => v,
            Err(_) => {
                let f: f64 = digits.parse().unwrap_or(0.0);
                return Obj::Real(if negative { -f } else { f });
            }
        };
        if negative {
            return Obj::Int(-v);
        }
        // "12 0 R": referencia a un objeto indirecto.
        let save = self.pos;
        self.skip_ws();
        let gs = self.pos;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if self.pos > gs && self.pos - gs <= 5 {
            let gen: u32 = std::str::from_utf8(&self.d[gs..self.pos]).ok().and_then(|g| g.parse().ok()).unwrap_or(0);
            self.skip_ws();
            if self.peek() == Some(b'R') && self.d.get(self.pos + 1).map_or(true, |c| is_ws(*c) || is_delim(*c)) && v <= u32::MAX as i64 && gen <= u16::MAX as u32 {
                self.pos += 1;
                return Obj::Ref(v as u32, gen as u16);
            }
        }
        self.pos = save;
        Obj::Int(v)
    }

    fn literal_string(&mut self) -> Vec<u8> {
        self.pos += 1;
        let mut out = Vec::new();
        let mut depth = 1usize;
        while let Some(c) = self.peek() {
            self.pos += 1;
            match c {
                b'(' => {
                    depth += 1;
                    out.push(c);
                }
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    out.push(c);
                }
                b'\\' => {
                    let e = match self.peek() {
                        Some(e) => e,
                        None => break,
                    };
                    self.pos += 1;
                    match e {
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'0'..=b'7' => {
                            let mut v = (e - b'0') as u32;
                            for _ in 0..2 {
                                match self.peek() {
                                    Some(d @ b'0'..=b'7') => {
                                        v = v * 8 + (d - b'0') as u32;
                                        self.pos += 1;
                                    }
                                    _ => break,
                                }
                            }
                            out.push(v as u8);
                        }
                        // Barra invertida al final de linea: continuacion, no agrega nada.
                        b'\r' => {
                            if self.peek() == Some(b'\n') {
                                self.pos += 1;
                            }
                        }
                        b'\n' => {}
                        other => out.push(other),
                    }
                }
                _ => out.push(c),
            }
        }
        out
    }

    fn hex_string(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut high: Option<u8> = None;
        while let Some(c) = self.peek() {
            self.pos += 1;
            if c == b'>' {
                break;
            }
            let v = match c {
                b'0'..=b'9' => c - b'0',
                b'a'..=b'f' => c - b'a' + 10,
                b'A'..=b'F' => c - b'A' + 10,
                _ => continue,
            };
            match high.take() {
                Some(h) => out.push(h << 4 | v),
                None => high = Some(v),
            }
        }
        if let Some(h) = high {
            out.push(h << 4);
        }
        out
    }

    fn dict(&mut self) -> Dict {
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.peek().is_none() {
                break;
            }
            if self.starts_with(b">>") {
                self.pos += 2;
                break;
            }
            let key = match self.object() {
                Some(Obj::Name(n)) => n,
                // Diccionario sin cerrar: no seguir consumiendo objetos ajenos.
                Some(Obj::Op(op)) if op == "endobj" || op == "stream" => {
                    self.pos -= op.len();
                    break;
                }
                Some(_) => continue,
                None => break,
            };
            self.skip_ws();
            if self.starts_with(b">>") {
                self.pos += 2;
                items.push((key, Obj::Null));
                break;
            }
            match self.object() {
                Some(v) => items.push((key, v)),
                None => break,
            }
        }
        Dict(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Obj {
        Lexer::new(s.as_bytes(), 0).object().unwrap()
    }

    #[test]
    fn objetos_basicos() {
        assert_eq!(parse("  % comentario\n 42 "), Obj::Int(42));
        assert_eq!(parse("-3.5"), Obj::Real(-3.5));
        assert_eq!(parse(".5"), Obj::Real(0.5));
        assert_eq!(parse("12 0 R"), Obj::Ref(12, 0));
        assert_eq!(parse("/Nombre#20con#20espacios"), Obj::Name("Nombre con espacios".into()));
        assert_eq!(parse("(a\\(b\\)c (anidado) \\101\\n)"), Obj::Str(b"a(b)c (anidado) A\n".to_vec()));
        assert_eq!(parse("<48 6f6C61>"), Obj::Str(b"Hola".to_vec()));
        assert_eq!(parse("<901FA>"), Obj::Str(vec![0x90, 0x1F, 0xA0]));
        assert_eq!(parse("[1 2 0 R /X (y) true null]"), Obj::Array(vec![Obj::Int(1), Obj::Ref(2, 0), Obj::Name("X".into()), Obj::Str(b"y".to_vec()), Obj::Bool(true), Obj::Null]));
    }

    #[test]
    fn diccionarios_y_operadores() {
        let d = parse("<< /Type /Page /Kids [3 0 R] /Count 1 /Sub << /A 1 >> >>");
        let d = d.as_dict().unwrap();
        assert_eq!(d.get("Type"), Some(&Obj::Name("Page".into())));
        assert_eq!(d.get("Count").and_then(|c| c.as_i64()), Some(1));
        assert!(d.get("Sub").unwrap().as_dict().unwrap().has("A"));

        let mut lx = Lexer::new(b"1 0 0 RG BT /F1 12 Tf (Hola) Tj T* [(a) -250 (b)] TJ ET", 0);
        let mut ops = Vec::new();
        while let Some(o) = lx.object() {
            if let Obj::Op(op) = o {
                ops.push(op);
            }
        }
        assert_eq!(ops, ["RG", "BT", "Tf", "Tj", "T*", "TJ", "ET"]);
    }
}
