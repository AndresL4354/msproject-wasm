//! Serializador JSON minimo (sin serde).

pub fn esc(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            // Separadores de linea Unicode: validos en JSON pero problematicos en JavaScript antiguo.
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c => out.push(c),
        }
    }
    out.push('"');
}

pub fn string(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    esc(&mut o, s);
    o
}

pub fn num(v: f64) -> String {
    if !v.is_finite() {
        return "null".into();
    }
    let r = (v * 100.0).round() / 100.0;
    if r == r.trunc() && r.abs() < 1e15 {
        format!("{}", r as i64)
    } else {
        format!("{}", r)
    }
}

pub fn array(items: &[String]) -> String {
    format!("[{}]", items.join(","))
}

pub struct Obj {
    buf: String,
    first: bool,
}

impl Obj {
    pub fn new() -> Obj {
        Obj { buf: String::from("{"), first: true }
    }

    fn key(&mut self, k: &str) {
        if !self.first {
            self.buf.push(',');
        }
        self.first = false;
        esc(&mut self.buf, k);
        self.buf.push(':');
    }

    pub fn raw(&mut self, k: &str, v: &str) -> &mut Obj {
        self.key(k);
        self.buf.push_str(v);
        self
    }

    pub fn str(&mut self, k: &str, v: &str) -> &mut Obj {
        self.key(k);
        esc(&mut self.buf, v);
        self
    }

    /// Solo escribe la clave si hay un valor no vacio.
    pub fn opt(&mut self, k: &str, v: Option<&str>) -> &mut Obj {
        if let Some(s) = v.filter(|s| !s.is_empty()) {
            self.str(k, s);
        }
        self
    }

    pub fn int(&mut self, k: &str, v: i64) -> &mut Obj {
        self.raw(k, &v.to_string())
    }

    pub fn num(&mut self, k: &str, v: f64) -> &mut Obj {
        self.raw(k, &num(v))
    }

    pub fn bool(&mut self, k: &str, v: bool) -> &mut Obj {
        self.raw(k, if v { "true" } else { "false" })
    }

    pub fn is_empty(&self) -> bool {
        self.first
    }

    pub fn end(mut self) -> String {
        self.buf.push('}');
        self.buf
    }
}

/// Cuenta palabras como secuencias separadas por espacios en blanco.
pub fn word_count(s: &str) -> usize {
    s.split_whitespace().count()
}
