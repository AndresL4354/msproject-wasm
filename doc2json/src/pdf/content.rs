//! Interprete de los flujos de contenido de una pagina: ejecuta los operadores de texto y
//! devuelve tramos de texto con su posicion, tamano y direccion.

use super::document::Pdf;
use super::font::Font;
use super::object::{is_delim, is_ws, Dict, Lexer, Obj};
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
    /// Parte del avance del ultimo glifo que es espaciado (`Tc`, y `Tw` en un espacio), en puntos
    /// a lo largo de la linea: el glifo visible termina antes de `x1`.
    pub tail: f64,
    /// Direccion de escritura (vector unitario).
    pub dx: f64,
    pub dy: f64,
    pub bold: bool,
    /// PDF etiquetado: identificador del contenido marcado (MCID) al que pertenece el tramo.
    pub mcid: Option<i64>,
    /// PDF etiquetado: el tramo es un artefacto (encabezado, pie, numero de pagina), no contenido.
    pub artifact: bool,
    /// Tipo de artefacto: `ART_PAGINACION` (sin subtipo), `ART_ENCABEZADO`, `ART_PIE` u `ART_OTRO`.
    pub art: u8,
    /// Artefacto que envuelve al tramo (numero de su `BMC`/`BDC`, 0 = ninguno). Si ese artefacto
    /// nunca se cierra con `EMC`, el tramo vuelve a ser contenido.
    pub scope: u32,
}

/// Artefacto de paginacion sin subtipo declarado: se clasifica por su posicion en la pagina.
pub const ART_PAGINACION: u8 = 0;
pub const ART_ENCABEZADO: u8 = 1;
pub const ART_PIE: u8 = 2;
/// Diseno, fondo o pagina (lineas, marcas de agua): no es texto del documento.
pub const ART_OTRO: u8 = 3;

/// Contenido marcado abierto (`BMC`/`BDC` ... `EMC`).
#[derive(Clone, Copy, Default)]
struct Marked {
    mcid: Option<i64>,
    artifact: bool,
    art: u8,
    /// Artefacto mas externo abierto (0 = ninguno) y si este contenido es el que lo abrio.
    scope: u32,
    opens: bool,
}

/// Profundidad maxima del contenido marcado; lo que se abre mas adentro solo se cuenta.
const MAX_MARKED: usize = 256;

/// Una imagen dibujada: su objeto y la transformacion con que se dibujo.
pub struct ImageUse {
    /// Numero del objeto: las imagenes se reconocen por el aunque se repitan. Sin la generacion,
    /// porque `Pdf::resolve` tampoco la usa (`7 0 R` y `7 1 R` son el mismo objeto).
    pub key: u32,
    /// El objeto compartido (el mismo `Rc` del cache del PDF): dibujar mil veces una imagen no
    /// copia mil veces su diccionario. Por eso solo se registran imagenes indirectas.
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
    /// Huecos dudosos del tramo actual (posicion en el texto y ancho relativo al tamano de letra):
    /// mas anchos que el apretado entre letras pero menos que un espacio comun.
    doubt: Vec<(usize, f64)>,
    /// Uniones apretadas entre glifos del tramo actual.
    tight: usize,
    /// Huecos (relativos al tamano de letra) que ya pusieron un espacio en el tramo actual.
    inserted: Vec<f64>,
    /// Donde empieza el ultimo glifo del tramo actual.
    glyph: (f64, f64),
    /// Fuente del ultimo glifo del tramo actual (direccion de su `Rc`).
    glyph_font: usize,
    /// El ultimo glifo del tramo actual es una ligadura (fi, fl, ff...).
    ligature: bool,
    /// Uniones del tramo actual entre letras ubicadas aparte con anchos de fuente exagerados.
    wide: Vec<Wide>,
    /// Pila de contenido marcado; cruza los formularios invocados con `Do`.
    marked: Vec<Marked>,
    /// Contenido marcado abierto por encima de `MAX_MARKED` (sus `EMC` no tocan la pila).
    overflow: usize,
    /// Numero del ultimo artefacto abierto.
    scopes: u32,
    /// Largo de `marked` y valor de `overflow` al empezar el flujo en curso (pagina, anotacion o
    /// formulario): un `EMC` que sobra no cierra lo que abrio quien invoco el flujo.
    flow: (usize, usize),
}

/// Una union entre dos letras ubicadas aparte, con anchos de fuente mayores que los glifos reales:
/// donde iria un espacio en el texto del tramo, cuanto avanzo la letra anterior y cuanto declara la
/// fuente para ella (en puntos), y la fuente de esa letra.
struct Wide {
    at: usize,
    forward: f64,
    declared: f64,
    font: usize,
}

/// Uniones con anchos exagerados que se recuerdan por tramo; las siguientes van pegadas.
const MAX_WIDE: usize = 1 << 16;

/// Espacios entre palabras de un tramo con anchos de fuente exagerados: (posicion, hueco relativo
/// al tamano de letra), con todas las uniones del tramo a la vista. Por cada fuente, las uniones
/// dentro de una palabra avanzan una misma proporcion del ancho declarado: la que reune mas
/// uniones dentro de un 5 % (la mayoria, al menos tres). Una union que avanza mas de `WORD_GAP`
/// por sobre esa proporcion es un espacio. Si la fuente declara el mismo ancho para todas las
/// letras, la proporcion solo dice algo si las letras reales tambien miden lo mismo: con una union
/// que no este ni en la proporcion ni claramente por encima, o con huecos que no son casi iguales
/// entre si (letras reales de distinto ancho), una letra ancha no se distingue de un espacio y no
/// se separa nada. Queda un caso que no se puede distinguir sin las medidas reales de los glifos:
/// un tramo cuyas unicas letras anchas miden justo una letra y un espacio (la W de Helvetica).
fn wide_spaces(wide: &[Wide], size: f64) -> Vec<(usize, f64)> {
    let mut out = Vec::new();
    let mut fonts: Vec<usize> = wide.iter().map(|w| w.font).collect();
    fonts.sort_unstable();
    fonts.dedup();
    for f in fonts {
        let joins: Vec<&Wide> = wide.iter().filter(|w| w.font == f).collect();
        let mut ratios: Vec<f64> = joins.iter().map(|w| w.forward / w.declared).collect();
        ratios.sort_by(|a, b| a.total_cmp(b));
        let (mut scale, mut count, mut j) = (0.0, 0usize, 0usize);
        for i in 0..ratios.len() {
            j = j.max(i);
            while j < ratios.len() && ratios[j] <= ratios[i] * 1.05 {
                j += 1;
            }
            if j - i > count {
                count = j - i;
                scale = ratios[i];
            }
        }
        if count < 3 || 2 * count <= joins.len() {
            continue;
        }
        let gap = |w: &Wide| w.forward - w.declared * scale;
        let (lo, hi) = joins.iter().fold((f64::MAX, 0.0f64), |(lo, hi), w| (lo.min(w.declared), hi.max(w.declared)));
        if hi <= lo * 1.01 {
            let mixed = joins.iter().any(|w| {
                let r = w.forward / w.declared;
                !(r >= scale * 0.95 && r <= scale * 1.05) && gap(w) <= WORD_GAP * size
            });
            let (g0, g1) = joins.iter().map(|w| gap(w)).filter(|g| *g > WORD_GAP * size).fold((f64::MAX, 0.0f64), |(lo, hi), g| (lo.min(g), hi.max(g)));
            if mixed || g1 > g0 * 1.15 {
                continue;
            }
        }
        out.extend(joins.iter().filter(|w| gap(w) > WORD_GAP * size).map(|w| (w.at, gap(w) / size)));
    }
    out.sort_by_key(|a| a.0);
    out
}

/// Un glifo que da varias letras de una ligadura tipografica, o la ligadura misma.
fn is_ligature(text: &str) -> bool {
    matches!(text, "ff" | "fi" | "fl" | "ffi" | "ffl" | "ft" | "st") || text.chars().any(|c| ('\u{FB00}'..='\u{FB06}').contains(&c))
}

/// Las ligaduras latinas de Unicode (U+FB00 a U+FB06) como sus letras.
fn unligature(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    for c in text.chars() {
        match c {
            '\u{FB00}' => out.push_str("ff"),
            '\u{FB01}' => out.push_str("fi"),
            '\u{FB02}' => out.push_str("fl"),
            '\u{FB03}' => out.push_str("ffi"),
            '\u{FB04}' => out.push_str("ffl"),
            '\u{FB05}' | '\u{FB06}' => out.push_str("st"),
            c => out.push(c),
        }
    }
    out
}

/// `text` con un espacio en cada posicion de `at` (crecientes, en el borde de un caracter), armado
/// en una sola pasada: insertar uno a uno desplaza el resto de la cadena cada vez.
fn with_spaces(text: &str, at: &[(usize, f64)]) -> String {
    let mut out = String::with_capacity(text.len() + at.len());
    let mut from = 0;
    for &(pos, _) in at {
        out.push_str(&text[from..pos]);
        out.push(' ');
        from = pos;
    }
    out.push_str(&text[from..]);
    out
}

/// Desde este hueco (relativo al tamano de letra) hay un espacio entre palabras.
const WORD_GAP: f64 = 0.15;
/// Huecos entre este valor y `WORD_GAP` son dudosos: en una linea justificada apretada son
/// espacios; sueltos, suelen ser letras mal ubicadas de una exportacion.
const DOUBT_GAP: f64 = 0.09;
/// Hasta aqui una union entre glifos es apretada (pegados o con un ajuste de interletra).
const TIGHT_GAP: f64 = 0.05;

fn num(o: Option<&Obj>) -> f64 {
    o.and_then(|v| v.as_f64()).unwrap_or(0.0)
}

/// Un caracter que forma parte de un operador o un nombre (ni espacio ni delimitador).
fn is_regular(b: u8) -> bool {
    !is_ws(b) && !is_delim(b)
}

/// Operadores de dibujo: trazos, recortes, colores y el estado grafico que no toca al texto. Un
/// dibujo vectorial trae millones, y se descartan sin convertir sus operandos. Cualquier otro
/// operador, conocido o no, pasa por `exec`: uno que se le agregue despues (por ejemplo el
/// contenido marcado, `BMC`/`BDC`/`EMC`) funciona sin tocar esta lista. Ninguno de estos puede
/// tener una rama en `exec` (lo vigila una prueba).
fn drawing_op(w: &[u8]) -> bool {
    matches!(
        w,
        // Trazos y recortes.
        b"m" | b"l" | b"c" | b"v" | b"y" | b"h" | b"re" | b"S" | b"s" | b"f" | b"F" | b"f*" | b"B" | b"B*" | b"b" | b"b*" | b"n" | b"W" | b"W*" | b"sh"
        // Colores.
        | b"g" | b"G" | b"rg" | b"RG" | b"k" | b"K" | b"cs" | b"CS" | b"sc" | b"SC" | b"scn" | b"SCN"
        // Estado grafico que no toca al texto.
        | b"w" | b"J" | b"j" | b"M" | b"d" | b"ri" | b"i"
    )
}

/// Tope de operandos de un operador: los que sobran se descartan, asi un flujo que repite
/// millones de operandos no agota la memoria.
const MAX_ARGS: usize = 256;

/// Lo que sigue en un flujo de contenido, despues de sus operandos.
#[derive(Debug, PartialEq)]
enum Next<'c> {
    /// Un operador para `exec`.
    Op(&'c str),
    /// Un operador de dibujo, o un delimitador suelto: sus operandos se descartan.
    Skip,
    End,
}

/// Lee los operandos hasta el proximo operador y los deja en `args`, a lo sumo `MAX_ARGS`. Los
/// numeros quedan sin convertir (`Obj::Null`, con su posicion en `args` y en el flujo en `spans`):
/// solo los de los operadores que `exec` usa hacen falta.
fn next_op<'c>(lx: &mut Lexer<'c>, args: &mut Vec<Obj>, spans: &mut Vec<(usize, usize, usize)>) -> Next<'c> {
    let d: &'c [u8] = lx.d;
    loop {
        lx.skip_ws();
        let b = match lx.peek() {
            Some(b) => b,
            None => return Next::End,
        };
        let arg = if matches!(b, b'+' | b'-' | b'.' | b'0'..=b'9') {
            let start = lx.pos;
            while matches!(lx.peek(), Some(b'+' | b'-' | b'.' | b'0'..=b'9')) {
                lx.pos += 1;
            }
            if args.len() < MAX_ARGS {
                spans.push((args.len(), start, lx.pos));
                args.push(Obj::Null);
            }
            continue;
        } else if is_regular(b) {
            let start = lx.pos;
            while matches!(lx.peek(), Some(c) if is_regular(c)) {
                lx.pos += 1;
            }
            match &d[start..lx.pos] {
                b"true" => Obj::Bool(true),
                b"false" => Obj::Bool(false),
                b"null" => Obj::Null,
                w if drawing_op(w) => return Next::Skip,
                w => return std::str::from_utf8(w).map_or(Next::Skip, Next::Op),
            }
        } else {
            match lx.object() {
                // ">>", "]", ")", "{" o "}" sueltos: ningun operador de `exec` los usa.
                Some(Obj::Op(_)) => return Next::Skip,
                Some(o) => o,
                None => return Next::End,
            }
        };
        if args.len() < MAX_ARGS {
            args.push(arg);
        }
    }
}

/// Un numero de un flujo de contenido, como lo lee `Lexer` (tolera signos repetidos, "--5").
fn number(raw: &[u8]) -> Obj {
    let raw = std::str::from_utf8(raw).unwrap_or("0");
    let negative = raw.starts_with('-');
    let digits = raw.trim_start_matches(['+', '-']);
    let sign = |v: f64| if negative { -v } else { v };
    if digits.contains('.') {
        return Obj::Real(sign(digits.parse().unwrap_or(0.0)));
    }
    match digits.parse::<i64>() {
        Ok(v) => Obj::Int(if negative { -v } else { v }),
        Err(_) => Obj::Real(sign(digits.parse().unwrap_or(0.0))),
    }
}

impl<'p, 'a> Interp<'p, 'a> {
    pub fn new(pdf: &'p Pdf<'a>, fonts: &'p FontCache) -> Interp<'p, 'a> {
        Interp {
            pdf,
            fonts,
            out: PageText::default(),
            cur: None,
            depth: 0,
            budget: 20_000_000,
            doubt: Vec::new(),
            tight: 0,
            inserted: Vec::new(),
            glyph: (0.0, 0.0),
            glyph_font: 0,
            ligature: false,
            wide: Vec::new(),
            marked: Vec::new(),
            overflow: 0,
            scopes: 0,
            flow: (0, 0),
        }
    }

    /// Cierra el contenido marcado que haya quedado abierto (antes de las anotaciones, que van aparte).
    pub fn end_marked(&mut self) {
        self.flush();
        self.close_unbalanced(0, 0, 0);
    }

    /// Cierra el contenido marcado que quedo abierto por encima de `base` al terminar un flujo
    /// (pagina, anotacion o formulario). Un artefacto sin su `EMC` convertiria en artefacto todo lo
    /// que sigue, y ese texto desapareceria del contenido: lo escrito dentro de un artefacto que
    /// nunca se cerro vuelve a ser contenido (desde el tramo `from`, el primero de ese flujo).
    fn close_unbalanced(&mut self, base: usize, overflow: usize, from: usize) {
        self.overflow = overflow;
        if self.marked.len() <= base {
            return;
        }
        let open: Vec<u32> = self.marked[base..].iter().filter(|m| m.opens).map(|m| m.scope).collect();
        self.marked.truncate(base);
        if open.is_empty() {
            return;
        }
        self.flush();
        for it in self.out.items.iter_mut().skip(from) {
            if it.scope != 0 && open.contains(&it.scope) {
                it.artifact = false;
                it.art = ART_PAGINACION;
                it.scope = 0;
            }
        }
    }

    /// Contenido marcado vigente: el MCID mas interno y si alguno de los abiertos es artefacto.
    fn marked_now(&self) -> Marked {
        self.marked.last().copied().unwrap_or_default()
    }

    /// Abre contenido marcado. `props` es el diccionario de propiedades de `BDC` (en linea o por
    /// nombre en /Properties); `BMC` no lo trae. Un MCID ausente hereda el del contenido que lo envuelve.
    fn begin_marked(&mut self, tag: Option<&str>, props: Option<&Obj>, resources: &Dict) {
        let parent = self.marked_now();
        let props: Option<Dict> = match props {
            Some(Obj::Dict(d)) => Some(d.clone()),
            Some(Obj::Name(n)) => self.pdf.dict(resources, "Properties").and_then(|p| p.get(n).map(|o| self.pdf.resolve(o))).and_then(|o| o.as_dict().cloned()),
            _ => None,
        };
        let mcid = props.as_ref().and_then(|p| p.get("MCID")).and_then(|v| v.as_i64()).or(parent.mcid);
        let own = tag == Some("Artifact");
        let artifact = parent.artifact || own;
        // Solo el artefacto mas externo abre un ambito: los de adentro se cierran con el.
        let opens = own && !parent.artifact;
        let art = if own {
            let field = |k: &str| props.as_ref().and_then(|p| p.get(k)).and_then(|v| v.as_name()).map(String::from);
            match (field("Type").as_deref(), field("Subtype").as_deref()) {
                (_, Some("Header")) => ART_ENCABEZADO,
                (_, Some("Footer")) => ART_PIE,
                (Some("Layout") | Some("Page") | Some("Background"), _) | (_, Some("Watermark")) => ART_OTRO,
                _ => ART_PAGINACION,
            }
        } else {
            parent.art
        };
        if self.marked.len() >= MAX_MARKED {
            self.overflow += 1;
            return;
        }
        let scope = if opens {
            self.scopes = self.scopes.wrapping_add(1).max(1);
            self.scopes
        } else {
            parent.scope
        };
        self.marked.push(Marked { mcid, artifact, art, scope, opens });
    }

    fn font(&self, resources: &Dict, name: &str) -> Option<Rc<Font>> {
        let (obj, key) = self.pdf.sub_entry(resources, "Font", name)?;
        if let Some(k) = key {
            if let Some(f) = self.fonts.borrow().get(&k) {
                return Some(f.clone());
            }
        }
        let font = Rc::new(Font::load(self.pdf, obj.as_dict()?));
        if let Some(k) = key {
            self.fonts.borrow_mut().insert(k, font.clone());
        }
        Some(font)
    }

    fn flush(&mut self) {
        let doubt = std::mem::take(&mut self.doubt);
        let tight = std::mem::replace(&mut self.tight, 0);
        let mut inserted = std::mem::take(&mut self.inserted);
        let wide = std::mem::take(&mut self.wide);
        if let Some(mut item) = self.cur.take() {
            // Los espacios de las letras con anchos exagerados cuentan como los que puso un hueco
            // ancho (esas uniones se habian contado como apretadas).
            let mut at = wide_spaces(&wide, item.size);
            let tight = tight.saturating_sub(at.len());
            inserted.extend(at.iter().map(|a| a.1));
            // Una linea justificada que separa las palabras solo por su posicion (sin caracteres
            // de espacio; los que hay los puso un hueco ancho) puede apretarlas por debajo del
            // espacio comun. Sus huecos son casi iguales entre si y las letras de cada palabra van
            // juntas; un hueco dudoso suelto, o letras todas separadas (interletra amplia), no son
            // espacios.
            if doubt.len() >= 3 && item.text.matches(' ').count() + at.len() == inserted.len() && tight >= 2 * doubt.len() {
                let (lo, hi) = doubt.iter().fold((f64::MAX, 0.0f64), |(lo, hi), d| (lo.min(d.1), hi.max(d.1)));
                // Los espacios que ya hay, de la misma familia: la linea entera va justificada igual.
                if hi <= lo * 1.15 && inserted.iter().all(|g| *g <= hi * 1.15) {
                    at.extend(doubt);
                    at.sort_by_key(|a| a.0);
                }
            }
            if !at.is_empty() {
                item.text = with_spaces(&item.text, &at);
            }
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
        let font_id = Rc::as_ptr(&font) as usize;
        let (mut ox, mut oy) = (m[4] + st.ts * m[2], m[5] + st.ts * m[3]);
        let mut total = 0.0f64;
        let mut glyphs: Vec<(String, f64, f64, f64, f64, f64)> = Vec::new();
        let mut unmapped = 0usize;
        font.decode(bytes, |text, w, is_space| {
            let spacing = (st.tc + if is_space { st.tw } else { 0.0 }) * st.th;
            let adv = w * st.size * st.th + spacing;
            // El espaciado en puntos de pagina (la matriz puede escalar el texto).
            let tail = (spacing * ux).max(0.0);
            let (ex, ey) = (ox + adv * m[0], oy + adv * m[1]);
            if text == "\u{FFFD}" {
                unmapped += 1;
            }
            // Algunas tablas de caracteres defectuosas devuelven codigos de control: no son texto.
            if text.bytes().any(|b| b < 0x20) {
                let clean: String = text.chars().filter(|c| !c.is_control()).collect();
                if !clean.is_empty() {
                    glyphs.push((clean, ox, oy, ex, ey, tail));
                }
            } else if text.chars().any(|c| ('\u{FB00}'..='\u{FB06}').contains(&c)) {
                // Las ligaduras tipograficas como letras sueltas: "ﬁ" no coincide con "fi" al buscar.
                glyphs.push((unligature(text), ox, oy, ex, ey, tail));
            } else if !text.is_empty() {
                glyphs.push((text.to_string(), ox, oy, ex, ey, tail));
            }
            ox = ex;
            oy = ey;
            total += adv;
        });
        self.out.unmapped += unmapped;
        tm[4] += total * tm[0];
        tm[5] += total * tm[1];

        let mk = self.marked_now();
        for (text, sx, sy, ex, ey, tail) in glyphs {
            if let Some(cur) = self.cur.as_mut() {
                // Distancia del nuevo glifo al final del tramo, a lo largo y a traves de la linea.
                let mut along = (sx - cur.x1) * cur.dx + (sy - cur.y1) * cur.dy;
                // Un glifo ubicado aparte (`Td`, `Tm`) dentro del espaciado del anterior: quien
                // escribio el PDF declaro un `Tc` que no usa y ubica cada letra donde termina la
                // anterior. Se mide desde el final visible del glifo, no desde su avance.
                let mut wide = None;
                if along < -0.3 * size {
                    // Avance desde el comienzo del glifo anterior.
                    let forward = (sx - self.glyph.0) * cur.dx + (sy - self.glyph.1) * cur.dy;
                    if along + cur.tail >= -0.3 * size {
                        along += cur.tail;
                    } else if forward > 0.2 * size && forward < 1.2 * size {
                        // Anchos de la fuente mayores que los glifos reales: cada letra, ubicada
                        // aparte, parece montarse en la anterior aunque avanza. Va pegada a ella;
                        // si en realidad hay un espacio entre palabras se decide al cerrar el tramo
                        // (`wide_spaces`), con todas sus uniones a la vista.
                        let declared = (cur.x1 - self.glyph.0) * cur.dx + (cur.y1 - self.glyph.1) * cur.dy - cur.tail;
                        along = 0.0;
                        if declared > 0.0 {
                            wide = Some((forward, declared));
                        }
                    }
                }
                let across = (-(sx - cur.x1) * cur.dy + (sy - cur.y1) * cur.dx).abs();
                // Un tramo no mezcla contenido marcado distinto: cada uno pertenece a otro elemento.
                let same = across < 0.2 * size && (size - cur.size).abs() < 0.08 * size.max(cur.size) && cur.dx * dx + cur.dy * dy > 0.98 && cur.bold == font.bold && cur.mcid == mk.mcid && cur.artifact == mk.artifact && cur.art == mk.art;
                if same && along > -0.3 * size && along < 1.2 * size {
                    // Un espacio dibujado dentro del glifo anterior no separa nada (algunos
                    // programas lo agregan tras una ligadura); un espacio de verdad avanza.
                    if text.trim().is_empty() && along < -WORD_GAP * size && self.ligature {
                        continue;
                    }
                    // Un hueco mayor que un apretado entre letras es un espacio entre palabras.
                    if along > WORD_GAP * size && !cur.text.ends_with(' ') && !text.starts_with(' ') {
                        cur.text.push(' ');
                        self.inserted.push(along / size);
                    } else if along > DOUBT_GAP * size && !cur.text.ends_with(' ') && !text.starts_with(' ') {
                        // Entre dos cifras, un hueco fino agrupa los miles ("1 234 567") o las
                        // partes de un codigo: no separa palabras.
                        let digits = cur.text.ends_with(|c: char| c.is_ascii_digit()) && text.starts_with(|c: char| c.is_ascii_digit());
                        if !digits {
                            self.doubt.push((cur.text.len(), along / size));
                        }
                    } else if along.abs() <= TIGHT_GAP * size {
                        self.tight += 1;
                    }
                    if let Some((forward, declared)) = wide {
                        if !cur.text.ends_with(' ') && !text.starts_with(' ') && self.wide.len() < MAX_WIDE {
                            self.wide.push(Wide { at: cur.text.len(), forward, declared, font: self.glyph_font });
                        }
                    }
                    cur.text.push_str(&text);
                    cur.x1 = ex;
                    cur.y1 = ey;
                    cur.tail = tail;
                    // Dos artefactos seguidos en la misma linea forman un tramo; queda el ambito del
                    // ultimo, asi un artefacto sin cerrar devuelve el tramo entero al contenido.
                    cur.scope = mk.scope;
                    self.glyph = (sx, sy);
                    self.glyph_font = font_id;
                    self.ligature = is_ligature(&text);
                    continue;
                }
                self.flush();
            }
            if text.trim().is_empty() {
                continue; // un tramo no empieza con espacios
            }
            self.glyph = (sx, sy);
            self.glyph_font = font_id;
            self.ligature = is_ligature(&text);
            self.cur = Some(Item { text, x0: sx, y0: sy, x1: ex, y1: ey, size, tail, dx, dy, bold: font.bold, mcid: mk.mcid, artifact: mk.artifact, art: mk.art, scope: mk.scope });
        }
    }

    /// Ejecuta un flujo de contenido con los recursos dados.
    pub fn run(&mut self, content: &[u8], resources: &Dict, ctm: Matrix) {
        let mut st = State { ctm, font: None, size: 0.0, tc: 0.0, tw: 0.0, th: 1.0, tl: 0.0, ts: 0.0 };
        let (base, overflow, from) = (self.marked.len(), self.overflow, self.out.items.len());
        let outer = std::mem::replace(&mut self.flow, (base, overflow));
        self.exec(content, resources, &mut st);
        self.flow = outer;
        self.flush();
        self.close_unbalanced(base, overflow, from);
    }

    fn exec(&mut self, content: &[u8], resources: &Dict, st: &mut State) {
        let mut lx = Lexer::new(content, 0);
        let mut args: Vec<Obj> = Vec::new();
        // Numeros de `args` aun sin convertir (posicion en `args`, inicio y fin en el flujo).
        let mut spans: Vec<(usize, usize, usize)> = Vec::new();
        let mut saved: Vec<State> = Vec::new();
        let mut tm = IDENTITY;
        let mut tlm = IDENTITY;
        loop {
            let op = match next_op(&mut lx, &mut args, &mut spans) {
                Next::Op(op) => op,
                Next::Skip => {
                    // Un trazo o un color: se descarta sin armar nada.
                    if self.budget == 0 {
                        break;
                    }
                    self.budget -= 1;
                    args.clear();
                    spans.clear();
                    continue;
                }
                Next::End => break,
            };
            if self.budget == 0 {
                break;
            }
            self.budget -= 1;
            for (k, a, z) in spans.drain(..) {
                args[k] = number(&content[a..z]);
            }
            match op {
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
                "BMC" => self.begin_marked(args.first().and_then(|a| a.as_name()), None, resources),
                "BDC" => self.begin_marked(args.first().and_then(|a| a.as_name()), args.get(1), resources),
                "EMC" => {
                    // Un EMC que sobra en este flujo se ignora: no cierra el contenido marcado de
                    // la pagina que invoca al formulario.
                    let (base, overflow) = self.flow;
                    if self.overflow > overflow {
                        self.overflow -= 1;
                    } else if self.marked.len() > base {
                        self.marked.pop();
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
        // Sin copiar diccionarios: se dibujan muchas veces, y `Pdf::resolve` copia los objetos
        // directos. Uno indirecto se toma del cache del PDF; uno directo, prestado.
        let held;
        let xobjects: &Obj = match resources.get("XObject") {
            Some(o @ Obj::Ref(..)) => {
                held = self.pdf.resolve(o);
                &held
            }
            Some(o) => o,
            None => return,
        };
        let entry = match xobjects.as_dict().and_then(|x| x.get(name)) {
            Some(o) => o,
            None => return,
        };
        let shared = match entry {
            Obj::Ref(n, _) => Some((*n, self.pdf.resolve(entry))),
            _ => None,
        };
        let obj: &Obj = shared.as_ref().map_or(entry, |(_, o)| o);
        let dict = match obj.as_dict() {
            Some(d) => d,
            None => return,
        };
        match dict.get("Subtype").and_then(|s| s.as_name()) {
            Some("Image") => {
                self.out.images += 1;
                // Solo se registra una imagen indirecta (un flujo, como pide la norma), que comparte
                // el `Rc` del cache. Un diccionario directo no es una imagen valida, y guardarlo
                // pediria una copia por cada `Do`.
                if let Some((n, o)) = &shared {
                    if self.out.uses.len() < 100_000 {
                        self.out.uses.push(ImageUse { key: *n, obj: o.clone(), ctm: st.ctm });
                    }
                }
            }
            Some("Form") if self.depth < 12 => {
                let data = match self.pdf.stream_data(obj) {
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
                // El contenido marcado de un formulario tiene que cerrarse dentro de el.
                let (base, overflow, from) = (self.marked.len(), self.overflow, self.out.items.len());
                let outer = std::mem::replace(&mut self.flow, (base, overflow));
                self.depth += 1;
                self.exec(&data, &res, &mut inner);
                self.depth -= 1;
                self.flow = outer;
                self.close_unbalanced(base, overflow, from);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Los operandos y el operador que siguen en `src`.
    fn leer(src: &str) -> (Vec<Obj>, Next<'_>) {
        let mut lx = Lexer::new(src.as_bytes(), 0);
        let (mut args, mut spans) = (Vec::new(), Vec::new());
        let next = next_op(&mut lx, &mut args, &mut spans);
        for (k, a, z) in spans {
            args[k] = number(&src.as_bytes()[a..z]);
        }
        (args, next)
    }

    #[test]
    fn operandos_con_tope() {
        // `true`, `false` y `null` cuentan para el tope igual que los numeros y las cadenas.
        for relleno in ["null", "true", "false", "1", "(x)"] {
            let src = format!("{} 7 (fin) Tj", format!("{} ", relleno).repeat(3 * MAX_ARGS));
            let (args, next) = leer(&src);
            assert_eq!(next, Next::Op("Tj"));
            assert_eq!(args.len(), MAX_ARGS, "{}", relleno);
        }
        let (args, _) = leer("true false null 2.5 -3 /F1 (a) Tj");
        assert_eq!(args, vec![Obj::Bool(true), Obj::Bool(false), Obj::Null, Obj::Real(2.5), Obj::Int(-3), Obj::Name("F1".into()), Obj::Str(b"a".to_vec())]);
    }

    #[test]
    fn solo_el_dibujo_se_descarta() {
        assert_eq!(leer("0 0 10 10 re").1, Next::Skip);
        assert_eq!(leer("1 0 0 rg").1, Next::Skip);
        assert_eq!(leer("]").1, Next::Skip);
        // Los operadores que `exec` no usa hoy (contenido marcado, modo de relleno del texto, fin
        // de texto) le llegan igual, con sus operandos.
        assert_eq!(leer("/P << /MCID 3 >> BDC"), (vec![Obj::Name("P".into()), Obj::Dict(Dict(vec![("MCID".to_string(), Obj::Int(3))]))], Next::Op("BDC")));
        assert_eq!(leer("/Artifact BMC").1, Next::Op("BMC"));
        assert_eq!(leer("EMC").1, Next::Op("EMC"));
        assert_eq!(leer("3 Tr").1, Next::Op("Tr"));
        assert_eq!(leer("ET").1, Next::Op("ET"));
        assert_eq!(leer("  ").1, Next::End);
    }

    #[test]
    fn ningun_operador_de_dibujo_tiene_rama_en_exec() {
        // Una rama de `exec` para un operador de `drawing_op` nunca se ejecutaria. Las ramas se
        // buscan por sus cadenas: las que, solas o unidas por `|`, van seguidas de `=>` (con o sin
        // guarda `if`), sin importar los saltos de linea ni la sangria.
        let src = include_str!("content.rs");
        let exec = src[src.find("fn exec(").unwrap()..src.find("fn xobject(").unwrap()].as_bytes();
        let (mut ramas, mut patron) = (Vec::new(), Vec::new());
        let mut i = 0;
        while i < exec.len() {
            match exec[i] {
                b'/' if exec.get(i + 1) == Some(&b'/') => {
                    // Un comentario, hasta el fin de la linea.
                    while i < exec.len() && exec[i] != b'\n' {
                        i += 1;
                    }
                    continue;
                }
                b'"' => {
                    let mut lit = Vec::new();
                    i += 1;
                    while exec[i] != b'"' {
                        if exec[i] == b'\\' {
                            i += 1;
                        }
                        lit.push(exec[i]);
                        i += 1;
                    }
                    patron.push(String::from_utf8(lit).unwrap());
                }
                b'=' if exec.get(i + 1) == Some(&b'>') => {
                    ramas.append(&mut patron);
                    i += 1;
                }
                b'i' if !patron.is_empty() && exec[i..].starts_with(b"if") && exec.get(i + 2).map_or(false, |c| is_ws(*c)) => {
                    // Una guarda (`"re" if x =>`): la rama sigue siendo de esas cadenas.
                    while i + 1 < exec.len() && &exec[i..i + 2] != b"=>" {
                        i += 1;
                    }
                    continue;
                }
                b'|' => {}
                c if is_ws(c) => {}
                _ => patron.clear(),
            }
            i += 1;
        }
        for op in ["BT", "Tj", "TD", "\"", "Do"] {
            assert!(ramas.iter().any(|o| o == op), "{} no aparece en {:?}", op, ramas);
        }
        for op in &ramas {
            assert!(!drawing_op(op.as_bytes()), "`{}` tiene una rama en exec pero drawing_op lo descarta", op);
        }
    }

    #[test]
    fn espacios_en_una_pasada() {
        assert_eq!(with_spaces("ÉsteNiñoVa", &[(5, 0.1), (10, 0.1)]), "Éste Niño Va");
        assert_eq!(with_spaces("ab", &[]), "ab");
        assert_eq!(with_spaces("ab", &[(0, 0.1), (2, 0.1)]), " ab ");
    }
}
