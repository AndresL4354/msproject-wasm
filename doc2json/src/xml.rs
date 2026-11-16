//! Lector XML minimo "de arrastre": entrega etiquetas de apertura, cierre y texto en orden.
//! No valida ni resuelve espacios de nombres: los nombres se entregan tal cual ("w:p").

pub enum Ev<'a> {
    Start(Tag<'a>),
    End(&'a str),
    Text(String),
}

pub struct Tag<'a> {
    pub name: &'a str,
    attrs: &'a str,
    /// Etiqueta autocerrada (`<w:tab/>`): no tendra evento de cierre.
    pub empty: bool,
}

impl Tag<'_> {
    pub fn attr(&self, key: &str) -> Option<String> {
        let b = self.attrs.as_bytes();
        let mut i = 0usize;
        while i < b.len() {
            while i < b.len() && (b[i].is_ascii_whitespace() || b[i] == b'/') {
                i += 1;
            }
            let ks = i;
            while i < b.len() && b[i] != b'=' && !b[i].is_ascii_whitespace() {
                i += 1;
            }
            let k = &self.attrs[ks..i];
            while i < b.len() && (b[i].is_ascii_whitespace() || b[i] == b'=') {
                i += 1;
            }
            if i >= b.len() || (b[i] != b'"' && b[i] != b'\'') {
                continue;
            }
            let quote = b[i];
            i += 1;
            let vs = i;
            while i < b.len() && b[i] != quote {
                i += 1;
            }
            if k == key {
                return Some(unescape(&self.attrs[vs..i]));
            }
            i += 1;
        }
        None
    }

    /// Atributo booleano de WordprocessingML: presente sin valor = verdadero.
    pub fn on(&self, key: &str) -> bool {
        !matches!(self.attr(key).as_deref(), Some("0") | Some("false") | Some("off") | Some("none"))
    }
}

pub fn unescape(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(p) = rest.find('&') {
        out.push_str(&rest[..p]);
        rest = &rest[p..];
        let end = match rest.find(';') {
            Some(e) if e <= 12 => e,
            _ => {
                out.push('&');
                rest = &rest[1..];
                continue;
            }
        };
        let ent = &rest[1..end];
        let ch = match ent {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ => {
                let code = if let Some(h) = ent.strip_prefix("#x").or_else(|| ent.strip_prefix("#X")) {
                    u32::from_str_radix(h, 16).ok()
                } else if let Some(d) = ent.strip_prefix('#') {
                    d.parse().ok()
                } else {
                    None
                };
                code.and_then(char::from_u32)
            }
        };
        match ch {
            Some(c) => {
                out.push(c);
                rest = &rest[end + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

pub struct Xml<'a> {
    s: &'a str,
    pos: usize,
}

impl<'a> Xml<'a> {
    pub fn new(s: &'a str) -> Xml<'a> {
        Xml { s, pos: 0 }
    }

    pub fn next(&mut self) -> Option<Ev<'a>> {
        loop {
            let rest = &self.s[self.pos..];
            if rest.is_empty() {
                return None;
            }
            if !rest.starts_with('<') {
                let end = rest.find('<').unwrap_or(rest.len());
                self.pos += end;
                return Some(Ev::Text(unescape(&rest[..end])));
            }
            if rest.starts_with("<!--") {
                self.pos += rest.find("-->").map_or(rest.len(), |e| e + 3);
                continue;
            }
            if let Some(body) = rest.strip_prefix("<![CDATA[") {
                let end = body.find("]]>").unwrap_or(body.len());
                self.pos += 9 + end + 3.min(body.len() - end);
                return Some(Ev::Text(body[..end].to_string()));
            }
            if rest.starts_with("<?") {
                self.pos += rest.find("?>").map_or(rest.len(), |e| e + 2);
                continue;
            }
            if rest.starts_with("<!") {
                self.pos += rest.find('>').map_or(rest.len(), |e| e + 1);
                continue;
            }
            // Fin de la etiqueta, respetando comillas dentro de los atributos.
            let b = rest.as_bytes();
            let mut i = 1usize;
            let mut quote = 0u8;
            while i < b.len() {
                if quote != 0 {
                    if b[i] == quote {
                        quote = 0;
                    }
                } else if b[i] == b'"' || b[i] == b'\'' {
                    quote = b[i];
                } else if b[i] == b'>' {
                    break;
                }
                i += 1;
            }
            let inner = &rest[1..i.min(rest.len())];
            self.pos += (i + 1).min(rest.len());
            if let Some(name) = inner.strip_prefix('/') {
                return Some(Ev::End(name.trim()));
            }
            let empty = inner.ends_with('/');
            let inner = inner.strip_suffix('/').unwrap_or(inner);
            let cut = inner.find(|c: char| c.is_ascii_whitespace()).unwrap_or(inner.len());
            return Some(Ev::Start(Tag { name: &inner[..cut], attrs: &inner[cut..], empty }));
        }
    }

    /// Salta hasta el cierre del elemento `name` que se acaba de abrir (maneja anidamiento).
    pub fn skip(&mut self, name: &str) {
        let mut depth = 1usize;
        while let Some(ev) = self.next() {
            match ev {
                Ev::Start(t) if t.name == name && !t.empty => depth += 1,
                Ev::End(n) if n == name => {
                    depth -= 1;
                    if depth == 0 {
                        return;
                    }
                }
                _ => {}
            }
        }
    }

    /// Texto concatenado hasta el cierre del elemento `name` recien abierto.
    pub fn text_until(&mut self, name: &str) -> String {
        let mut out = String::new();
        while let Some(ev) = self.next() {
            match ev {
                Ev::Text(t) => out.push_str(&t),
                Ev::End(n) if n == name => break,
                _ => {}
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eventos_basicos() {
        let src = "<?xml version=\"1.0\"?><a x=\"1\" w:val='b&amp;c'><!-- nota --><b/>hola &lt;mundo&gt; &#241;</a>";
        let mut x = Xml::new(src);
        match x.next() {
            Some(Ev::Start(t)) => {
                assert_eq!(t.name, "a");
                assert!(!t.empty);
                assert_eq!(t.attr("x").as_deref(), Some("1"));
                assert_eq!(t.attr("w:val").as_deref(), Some("b&c"));
                assert_eq!(t.attr("val"), None);
            }
            _ => panic!("se esperaba <a>"),
        }
        assert!(matches!(x.next(), Some(Ev::Start(t)) if t.name == "b" && t.empty));
        assert!(matches!(x.next(), Some(Ev::Text(t)) if t == "hola <mundo> \u{f1}"));
        assert!(matches!(x.next(), Some(Ev::End("a"))));
        assert!(x.next().is_none());
    }

    #[test]
    fn saltar_elemento_anidado() {
        let mut x = Xml::new("<r><p><p>x</p></p><q>y</q></r>");
        x.next();
        x.next();
        x.skip("p");
        assert!(matches!(x.next(), Some(Ev::Start(t)) if t.name == "q"));
        assert_eq!(x.text_until("q"), "y");
    }
}
