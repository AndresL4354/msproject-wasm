//! Interprete de los flujos de contenido de una pagina: ejecuta los operadores de texto y
//! devuelve tramos de texto con su posicion, tamano y direccion.

use super::document::Pdf;
use super::font::Font;
use super::object::{is_ws, Dict, Lexer, Obj};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type Matrix = [f64; 6];
const IDENTITY: Matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

/// Producto de matrices `a x b` (convencion de PDF: vectores fila).
pub fn mul(a: Matrix, b: Matrix) -> Matrix {
    [
        a[0] * b[0] + a[1] * b[2],
        a[0] * b[1] + a[1] * b[3],
        a[2] * b[0] + a[3] * b[2],
        a[2] * b[1] + a[3] * b[3],
        a[4] * b[0] + a[5] * b[2] + b[4],
        a[4] * b[1] + a[5] * b[3] + b[5],
    ]
}

/// Tramo de texto en coordenadas de pagina.
#[derive(Clone, Debug)]
pub struct Item {
    pub text: String,
    /// Origen del primer glifo y fin del ultimo, sobre la linea base.
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    /// Tamano de letra efectivo, en puntos.
    pub size: f64,
    /// Direccion de escritura (vector unitario).
    pub dx: f64,
    pub dy: f64,
    pub bold: bool,
}

/// Una imagen dibujada: su objeto y la transformacion con que se dibujo.
pub struct ImageUse {
    /// Numero del objeto: las imagenes se reconocen por el aunque se repitan. Sin la generacion,
    /// porque `Pdf::resolve` tampoco la usa (`7 0 R` y `7 1 R` son el mismo objeto).
    pub key: Option<u32>,
    /// El objeto compartido (el mismo `Rc` del cache del PDF): dibujar mil veces una imagen no
    /// copia mil veces su diccionario.
    pub obj: Rc<Obj>,
    pub ctm: Matrix,
}

#[derive(Default)]
pub struct PageText {
    pub items: Vec<Item>,
    pub images: usize,
    /// Imagenes de objeto dibujadas (`Do`), en orden.
    pub uses: Vec<ImageUse>,
    /// Imagenes incrustadas en el contenido (`BI ... EI`): se cuentan, no se extraen.
    pub inline_images: usize,
    /// Glifos sin correspondencia Unicode conocida.
    pub unmapped: usize,
}

#[derive(Clone)]
struct State {
    ctm: Matrix,
    font: Option<Rc<Font>>,
    size: f64,
    tc: f64,
    tw: f64,
    th: f64,
    tl: f64,
    ts: f64,
}

/// Fuentes ya cargadas, compartidas entre paginas (por numero de objeto).
pub type FontCache = RefCell<HashMap<u32, Rc<Font>>>;

pub struct Interp<'p, 'a> {
    pdf: &'p Pdf<'a>,
    fonts: &'p FontCache,
    pub out: PageText,
    cur: Option<Item>,
    depth: usize,
    /// Presupuesto de operaciones, para cortar contenidos patologicos.
    budget: usize,
}

fn num(o: Option<&Obj>) -> f64 {
    o.and_then(|v| v.as_f64()).unwrap_or(0.0)
}

impl<'p, 'a> Interp<'p, 'a> {
    pub fn new(pdf: &'p Pdf<'a>, fonts: &'p FontCache) -> Interp<'p, 'a> {
        Interp { pdf, fonts, out: PageText::default(), cur: None, depth: 0, budget: 20_000_000 }
    }

    fn font(&self, resources: &Dict, name: &str) -> Option<Rc<Font>> {
        let fonts = self.pdf.dict(resources, "Font")?;
        let entry = fonts.get(name)?;
        let key = match entry {
            Obj::Ref(n, _) => Some(*n),
            _ => None,
        };
        if let Some(k) = key {
            if let Some(f) = self.fonts.borrow().get(&k) {
                return Some(f.clone());
            }
        }
        let obj = self.pdf.resolve(entry);
        let font = Rc::new(Font::load(self.pdf, obj.as_dict()?));
        if let Some(k) = key {
            self.fonts.borrow_mut().insert(k, font.clone());
        }
        Some(font)
    }

    fn flush(&mut self) {
        if let Some(item) = self.cur.take() {
            if !item.text.trim().is_empty() {
                self.out.items.push(item);
            }
        }
    }

    /// Muestra una cadena: avanza la matriz de texto y acumula los glifos en tramos.
    fn show(&mut self, st: &State, tm: &mut Matrix, bytes: &[u8]) {
        let font = match &st.font {
            Some(f) => f.clone(),
            None => return,
        };
        // Espacio de texto -> pagina.
        let m = mul(*tm, st.ctm);
        let ux = (m[0] * m[0] + m[1] * m[1]).sqrt();
        let uy = (m[2] * m[2] + m[3] * m[3]).sqrt();
        if ux < 1e-9 || uy < 1e-9 {
            return;
        }
        let (dx, dy) = (m[0] / ux, m[1] / ux);
        let size = (st.size * uy).abs();
        let (mut ox, mut oy) = (m[4] + st.ts * m[2], m[5] + st.ts * m[3]);
        let mut total = 0.0f64;
        let mut glyphs: Vec<(String, f64, f64, f64, f64)> = Vec::new();
        let mut unmapped = 0usize;
        font.decode(bytes, |text, w, is_space| {
            let adv = (w * st.size + st.tc + if is_space { st.tw } else { 0.0 }) * st.th;
            let (ex, ey) = (ox + adv * m[0], oy + adv * m[1]);
            if text == "\u{FFFD}" {
                unmapped += 1;
            }
            // Algunas tablas de caracteres defectuosas devuelven codigos de control: no son texto.
            if text.bytes().any(|b| b < 0x20) {
                let clean: String = text.chars().filter(|c| !c.is_control()).collect();
                if !clean.is_empty() {
                    glyphs.push((clean, ox, oy, ex, ey));
                }
            } else if !text.is_empty() {
                glyphs.push((text.to_string(), ox, oy, ex, ey));
            }
            ox = ex;
            oy = ey;
            total += adv;
        });
        self.out.unmapped += unmapped;
        tm[4] += total * tm[0];
        tm[5] += total * tm[1];

        for (text, sx, sy, ex, ey) in glyphs {
            if let Some(cur) = self.cur.as_mut() {
                // Distancia del nuevo glifo al final del tramo, a lo largo y a traves de la linea.
                let along = (sx - cur.x1) * cur.dx + (sy - cur.y1) * cur.dy;
                let across = (-(sx - cur.x1) * cur.dy + (sy - cur.y1) * cur.dx).abs();
                let same = across < 0.2 * size && (size - cur.size).abs() < 0.08 * size.max(cur.size) && cur.dx * dx + cur.dy * dy > 0.98 && cur.bold == font.bold;
                if same && along > -0.3 * size && along < 1.2 * size {
                    // Un hueco mayor que un apretado entre letras es un espacio entre palabras.
                    if along > 0.15 * size && !cur.text.ends_with(' ') && !text.starts_with(' ') {
                        cur.text.push(' ');
                    }
                    cur.text.push_str(&text);
                    cur.x1 = ex;
                    cur.y1 = ey;
                    continue;
                }
                self.flush();
            }
            if text.trim().is_empty() {
                continue; // un tramo no empieza con espacios
            }
            self.cur = Some(Item { text, x0: sx, y0: sy, x1: ex, y1: ey, size, dx, dy, bold: font.bold });
        }
    }

    /// Ejecuta un flujo de contenido con los recursos dados.
    pub fn run(&mut self, content: &[u8], resources: &Dict, ctm: Matrix) {
        let mut st = State { ctm, font: None, size: 0.0, tc: 0.0, tw: 0.0, th: 1.0, tl: 0.0, ts: 0.0 };
        self.exec(content, resources, &mut st);
        self.flush();
    }

    fn exec(&mut self, content: &[u8], resources: &Dict, st: &mut State) {
        let mut lx = Lexer::new(content, 0);
        let mut args: Vec<Obj> = Vec::new();
        let mut saved: Vec<State> = Vec::new();
        let mut tm = IDENTITY;
        let mut tlm = IDENTITY;
        while let Some(o) = lx.object() {
            let op = match o {
                Obj::Op(op) => op,
                other => {
                    if args.len() < 256 {
                        args.push(other);
                    }
                    continue;
                }
            };
            if self.budget == 0 {
                break;
            }
            self.budget -= 1;
            match op.as_str() {
                "BT" => {
                    tm = IDENTITY;
                    tlm = IDENTITY;
                }
                "Tf" => {
                    if let Some(name) = args.first().and_then(|a| a.as_name()) {
                        st.font = self.font(resources, name);
                    }
                    st.size = num(args.get(1));
                }
                "Td" | "TD" => {
                    let (tx, ty) = (num(args.first()), num(args.get(1)));
                    if op == "TD" {
                        st.tl = -ty;
                    }
                    tlm = mul([1.0, 0.0, 0.0, 1.0, tx, ty], tlm);
                    tm = tlm;
                }
                "Tm" => {
                    if args.len() >= 6 {
                        tm = [num(args.first()), num(args.get(1)), num(args.get(2)), num(args.get(3)), num(args.get(4)), num(args.get(5))];
                        tlm = tm;
                    }
                }
                "T*" => {
                    tlm = mul([1.0, 0.0, 0.0, 1.0, 0.0, -st.tl], tlm);
                    tm = tlm;
                }
                "Tc" => st.tc = num(args.first()),
                "Tw" => st.tw = num(args.first()),
                "Tz" => st.th = num(args.first()) / 100.0,
                "TL" => st.tl = num(args.first()),
                "Ts" => st.ts = num(args.first()),
                "Tj" => {
                    if let Some(s) = args.first().and_then(|a| a.as_bytes()) {
                        self.show(st, &mut tm, s);
                    }
                }
                "'" | "\"" => {
                    if op == "\"" {
                        st.tw = num(args.first());
                        st.tc = num(args.get(1));
                    }
                    tlm = mul([1.0, 0.0, 0.0, 1.0, 0.0, -st.tl], tlm);
                    tm = tlm;
                    if let Some(s) = args.last().and_then(|a| a.as_bytes()) {
                        self.show(st, &mut tm, s);
                    }
                }
                "TJ" => {
                    if let Some(items) = args.first().and_then(|a| a.as_array()) {
                        for it in items {
                            match it {
                                Obj::Str(s) => self.show(st, &mut tm, s),
                                other => {
                                    if let Some(n) = other.as_f64() {
                                        let tx = -n / 1000.0 * st.size * st.th;
                                        tm[4] += tx * tm[0];
                                        tm[5] += tx * tm[1];
                                    }
                                }
                            }
                        }
                    }
                }
                "cm" => {
                    if args.len() >= 6 {
                        let m = [num(args.first()), num(args.get(1)), num(args.get(2)), num(args.get(3)), num(args.get(4)), num(args.get(5))];
                        st.ctm = mul(m, st.ctm);
                    }
                }
                "q" => {
                    if saved.len() < 256 {
                        saved.push(st.clone());
                    }
                }
                "Q" => {
                    if let Some(s) = saved.pop() {
                        *st = s;
                    }
                }
                "Do" => {
                    if let Some(name) = args.first().and_then(|a| a.as_name()) {
                        self.xobject(resources, name, st);
                    }
                }
                "BI" => {
                    // Imagen incrustada: se salta el diccionario y los datos binarios.
                    while let Some(t) = lx.object() {
                        if t == Obj::Op("ID".into()) {
                            break;
                        }
                    }
                    let mut p = lx.pos + 1;
                    let end = loop {
                        match content.get(p..).and_then(|rest| rest.windows(2).position(|w| w == b"EI")) {
                            Some(k) => {
                                let at = p + k;
                                let before = at == 0 || is_ws(content[at - 1]);
                                let after = content.get(at + 2).map_or(true, |c| is_ws(*c));
                                // Lo que sigue debe parecer contenido normal, no datos binarios.
                                let tail_ok = content[(at + 2).min(content.len())..].iter().take(12).all(|c| *c < 128 && (*c >= 32 || is_ws(*c)));
                                if before && after && tail_ok {
                                    break at + 2;
                                }
                                p = at + 2;
                            }
                            None => break content.len(),
                        }
                    };
                    lx.pos = end;
                    self.out.images += 1;
                    self.out.inline_images += 1;
                }
                _ => {}
            }
            args.clear();
        }
    }

    fn xobject(&mut self, resources: &Dict, name: &str, st: &State) {
        // Sin copiar el diccionario de recursos: se dibujan muchas veces.
        let xobjects = self.pdf.get(resources, "XObject");
        let (obj, key) = match xobjects.as_dict().and_then(|x| x.get(name)) {
            Some(o) => (self.pdf.resolve(o), if let Obj::Ref(n, _) = o { Some(*n) } else { None }),
            None => return,
        };
        let dict = match obj.as_dict() {
            Some(d) => d,
            None => return,
        };
        match dict.get("Subtype").and_then(|s| s.as_name()) {
            Some("Image") => {
                self.out.images += 1;
                if self.out.uses.len() < 100_000 {
                    self.out.uses.push(ImageUse { key, obj: obj.clone(), ctm: st.ctm });
                }
            }
            Some("Form") if self.depth < 12 => {
                let data = match self.pdf.stream_data(&obj) {
                    Some(d) => d,
                    None => return,
                };
                let matrix = match self.pdf.array(dict, "Matrix") {
                    Some(m) if m.len() >= 6 => [num(Some(&m[0])), num(Some(&m[1])), num(Some(&m[2])), num(Some(&m[3])), num(Some(&m[4])), num(Some(&m[5]))],
                    _ => IDENTITY,
                };
                // Un formulario sin recursos propios usa los de quien lo invoca.
                let res = self.pdf.dict(dict, "Resources").unwrap_or_else(|| resources.clone());
                let mut inner = st.clone();
                inner.ctm = mul(matrix, st.ctm);
                self.depth += 1;
                self.exec(&data, &res, &mut inner);
                self.depth -= 1;
            }
            _ => {}
        }
    }
}
