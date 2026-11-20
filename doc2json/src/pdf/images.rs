//! Imagenes de PDF: formato con que se entregan, ubicacion en la pagina y extraccion.
//!
//! - JPEG (DCTDecode) y JPEG 2000 (JPXDecode) se entregan tal cual, sin recomprimir.
//! - Fax (CCITTFaxDecode) se envuelve en un TIFF de una tira, sin decodificar.
//! - Los pixeles (Flate, LZW, RunLength, sin compresion...) pasan a PNG: gris, RGB, CMYK
//!   (convertido a RGB), indexado (con su paleta), tinta plana (Separation) y mascaras de un bit;
//!   con transparencia si traen mascara suave (SMask) del mismo tamano. Si el flujo ya es un PNG
//!   por dentro (Flate con predictor PNG y parametros coincidentes) se copia sin recomprimir.
//! - JBIG2 y espacios de color como Lab no se convierten: la imagen figura con una nota.

use super::document::Pdf;
use super::filters::{self, Parms};
use super::object::{Dict, Obj};
use crate::inflate;
use crate::png::{self, Color, Spec};
use std::collections::HashMap;
use std::rc::Rc;

/// Tope de pixeles por imagen al extraer (5.000 x 5.000): mas alla, la memoria no compensa.
pub const MAX_PIXELS: u64 = 25_000_000;

/// Bytes de la tabla de una paleta que se pueden usar: 256 entradas de hasta 4 componentes.
const MAX_LOOKUP: usize = 256 * 4;

/// Mascara suave decodificada: transparencia en 8 bits y color `/Matte` (0-255), si lo trae.
struct Mask {
    alpha: Vec<u8>,
    matte: Option<Vec<f64>>,
}

/// Lo que varias imagenes comparten y se decodifica una sola vez por documento: las tablas de
/// las paletas `/Indexed` y las mascaras suaves, por numero de objeto.
pub struct Shared {
    /// Tablas de paletas, recortadas a `MAX_LOOKUP` bytes.
    lookups: HashMap<u32, Rc<Vec<u8>>>,
    /// Mascaras ya decodificadas (`None` si no sirven). Cada una se guarda solo mientras le
    /// queden imagenes por usarla, segun `mask_uses`.
    masks: HashMap<u32, Option<Rc<Mask>>>,
    mask_uses: HashMap<u32, usize>,
}

impl Shared {
    /// Cuenta cuantas de las imagenes `objs` usan cada mascara suave.
    pub fn new(objs: &[Rc<Obj>]) -> Shared {
        let mut mask_uses = HashMap::new();
        for o in objs {
            if let Some(Obj::Ref(n, _)) = o.as_dict().and_then(|d| d.get("SMask")) {
                *mask_uses.entry(*n).or_insert(0) += 1;
            }
        }
        Shared { lookups: HashMap::new(), masks: HashMap::new(), mask_uses }
    }

    /// Tabla de colores de una paleta. La de un objeto indirecto se lee una sola vez.
    fn lookup(&mut self, pdf: &Pdf, l: &Obj) -> Rc<Vec<u8>> {
        let key = match l {
            Obj::Ref(n, _) => Some(*n),
            _ => None,
        };
        if let Some(t) = key.and_then(|n| self.lookups.get(&n)) {
            return t.clone();
        }
        let table = resolved(pdf, l, |o| match o {
            Obj::Str(s) => s[..s.len().min(MAX_LOOKUP)].to_vec(),
            s @ Obj::Stream { .. } => pdf.stream_data(s).map_or_else(Vec::new, |t| t[..t.len().min(MAX_LOOKUP)].to_vec()),
            _ => Vec::new(),
        });
        let table = Rc::new(table);
        if let Some(n) = key {
            self.lookups.insert(n, table.clone());
        }
        table
    }
}

/// Aplica `f` al objeto final de `o`. A diferencia de `Pdf::resolve`, no copia un objeto directo.
fn resolved<R>(pdf: &Pdf, o: &Obj, f: impl FnOnce(&Obj) -> R) -> R {
    match o {
        Obj::Ref(..) => f(&pdf.resolve(o)),
        other => f(other),
    }
}

const NOTE_JBIG2: &str = "Imagen JBIG2 (tipica de escaneos en blanco y negro): no se convierte.";

/// Filtro que ya es un formato de imagen: los datos se entregan desde ahi.
fn image_filter(name: &str) -> Option<&'static str> {
    match name {
        "DCTDecode" | "DCT" => Some("jpeg"),
        "JPXDecode" => Some("jp2"),
        "CCITTFaxDecode" | "CCF" => Some("tiff"),
        "JBIG2Decode" => Some("jbig2"),
        _ => None,
    }
}

#[derive(Clone, Debug)]
enum Space {
    Gray,
    Rgb,
    Cmyk,
    /// Paleta ya convertida a RGB (3 bytes por entrada).
    Indexed(Vec<u8>),
    /// Una tinta (Separation o DeviceN de un componente): 1 = tinta plena (oscuro).
    Tint,
    Other(String),
}

impl Space {
    fn components(&self) -> usize {
        match self {
            Space::Rgb => 3,
            Space::Cmyk => 4,
            _ => 1,
        }
    }
}

fn int(pdf: &Pdf, d: &Dict, key: &str) -> Option<i64> {
    pdf.get(d, key).as_i64()
}

fn flag(pdf: &Pdf, d: &Dict, key: &str) -> bool {
    matches!(&*pdf.get(d, key), Obj::Bool(true))
}

/// El diccionario pide invertir los valores (`/Decode [1 0 ...]`).
fn inverted(pdf: &Pdf, d: &Dict) -> bool {
    match pdf.array(d, "Decode") {
        Some(a) if a.len() >= 2 => a[0].as_f64() == Some(1.0) && a[1].as_f64() == Some(0.0),
        _ => false,
    }
}

/// Espacio de color. Sin `shared` no se lee la tabla de una paleta (para clasificar basta saber
/// que es indexado): `Space::Indexed` queda vacio.
fn space(pdf: &Pdf, obj: &Obj, depth: usize, shared: Option<&mut Shared>) -> Space {
    resolved(pdf, obj, |o| match o {
        Obj::Name(n) => match n.as_str() {
            "DeviceGray" | "G" | "CalGray" => Space::Gray,
            "DeviceRGB" | "RGB" | "CalRGB" => Space::Rgb,
            "DeviceCMYK" | "CMYK" => Space::Cmyk,
            other => Space::Other(other.to_string()),
        },
        Obj::Array(a) if depth < 4 => {
            let head = a.first().map(|x| pdf.resolve(x)).and_then(|x| x.as_name().map(String::from)).unwrap_or_default();
            match head.as_str() {
                "CalGray" => Space::Gray,
                "CalRGB" => Space::Rgb,
                "ICCBased" => {
                    let stream = a.get(1).map(|x| pdf.resolve(x));
                    let n = stream.as_deref().and_then(|s| match s {
                        Obj::Stream { dict, .. } => pdf.get(dict, "N").as_i64(),
                        _ => None,
                    });
                    match n {
                        Some(1) => Space::Gray,
                        Some(3) => Space::Rgb,
                        Some(4) => Space::Cmyk,
                        _ => Space::Other("ICCBased".into()),
                    }
                }
                "Indexed" | "I" => {
                    let base = a.get(1).map(|b| space(pdf, b, depth + 1, None)).unwrap_or(Space::Other("Indexed".into()));
                    if !matches!(base, Space::Rgb | Space::Cmyk | Space::Tint | Space::Gray) {
                        return Space::Other("Indexed".into());
                    }
                    let shared = match shared {
                        Some(s) => s,
                        None => return Space::Indexed(Vec::new()),
                    };
                    let hival = a.get(2).and_then(|v| resolved(pdf, v, |x| x.as_i64())).unwrap_or(0).clamp(0, 255) as usize;
                    let lookup = a.get(3).map(|l| shared.lookup(pdf, l)).unwrap_or_default();
                    let n = base.components();
                    let mut palette = Vec::with_capacity((hival + 1) * 3);
                    for i in 0..=hival {
                        let at = |k: usize| lookup.get(i * n + k).copied().unwrap_or(0);
                        let rgb = match base {
                            Space::Rgb => [at(0), at(1), at(2)],
                            Space::Cmyk => cmyk([at(0), at(1), at(2), at(3)]),
                            Space::Tint => [255 - at(0); 3],
                            _ => [at(0); 3],
                        };
                        palette.extend_from_slice(&rgb);
                    }
                    Space::Indexed(palette)
                }
                "Separation" => Space::Tint,
                "DeviceN" if a.get(1).and_then(|x| resolved(pdf, x, |v| v.as_array().map(|v| v.len()))) == Some(1) => Space::Tint,
                other => Space::Other(other.to_string()),
            }
        }
        _ => Space::Other("desconocido".into()),
    })
}

fn cmyk(c: [u8; 4]) -> [u8; 3] {
    let k = 255 - c[3] as u32;
    [((255 - c[0] as u32) * k / 255) as u8, ((255 - c[1] as u32) * k / 255) as u8, ((255 - c[2] as u32) * k / 255) as u8]
}

fn filter_names(pdf: &Pdf, d: &Dict) -> Vec<String> {
    match &*pdf.get(d, "Filter") {
        Obj::Name(n) => vec![n.clone()],
        Obj::Array(a) => a.iter().filter_map(|o| pdf.resolve(o).as_name().map(String::from)).collect(),
        _ => Vec::new(),
    }
}

fn image_space(pdf: &Pdf, d: &Dict, shared: Option<&mut Shared>) -> Space {
    if flag(pdf, d, "ImageMask") || flag(pdf, d, "IM") {
        return Space::Gray;
    }
    match d.get("ColorSpace").or_else(|| d.get("CS")) {
        Some(cs) => space(pdf, cs, 0, shared),
        None => Space::Other("sin espacio de color".into()),
    }
}

/// Formato con que se entregaria la imagen y, si no se puede entregar, por que. No decodifica
/// nada, tampoco la tabla de una paleta.
pub fn classify(pdf: &Pdf, d: &Dict) -> (&'static str, Option<String>) {
    if let Some(f) = filter_names(pdf, d).iter().find_map(|n| image_filter(n)) {
        return (f, (f == "jbig2").then(|| NOTE_JBIG2.to_string()));
    }
    match image_space(pdf, d, None) {
        Space::Other(name) => ("desconocido", Some(format!("Espacio de color {} no soportado.", name))),
        _ => ("png", None),
    }
}

/// Ancho y alto en pixeles declarados por el diccionario.
pub fn size(pdf: &Pdf, d: &Dict) -> Option<(u32, u32)> {
    let w = int(pdf, d, "Width").or_else(|| int(pdf, d, "W"))?;
    let h = int(pdf, d, "Height").or_else(|| int(pdf, d, "H"))?;
    (w > 0 && h > 0 && w <= u32::MAX as i64 && h <= u32::MAX as i64).then_some((w as u32, h as u32))
}

/// Rectangulo que ocupa en la pagina el cuadrado unidad transformado por `m` (asi se dibuja
/// una imagen), en la orientacion de lectura y con el origen arriba a la izquierda.
pub fn place(m: [f64; 6], bbox: [f64; 4], rotate: i64) -> (f64, f64, f64, f64) {
    let (w, h) = (bbox[2] - bbox[0], bbox[3] - bbox[1]);
    let ph = if rotate == 90 || rotate == 270 { w } else { h };
    let point = |x: f64, y: f64| {
        let (x, y) = (m[0] * x + m[2] * y + m[4] - bbox[0], m[1] * x + m[3] * y + m[5] - bbox[1]);
        match rotate {
            90 => (y, w - x),
            180 => (w - x, h - y),
            270 => (h - y, x),
            _ => (x, y),
        }
    };
    let corners = [point(0.0, 0.0), point(1.0, 0.0), point(0.0, 1.0), point(1.0, 1.0)];
    let (x0, x1) = corners.iter().fold((f64::MAX, f64::MIN), |(a, b), c| (a.min(c.0), b.max(c.0)));
    let (y0, y1) = corners.iter().fold((f64::MAX, f64::MIN), |(a, b), c| (a.min(c.1), b.max(c.1)));
    let r = |v: f64| (v * 100.0).round() / 100.0;
    (r(x0), r(ph - y1), r(x1 - x0), r(y1 - y0))
}

/// Bytes que costara extraer la imagen segun lo que declara, sin leer sus datos: sus pixeles
/// decodificados (y los de su mascara suave, si la trae) o, si se entrega tal cual (JPEG, JPEG
/// 2000, fax o JBIG2), el largo del flujo. 0 si `extract` la rechaza sin leer nada.
pub fn work(pdf: &Pdf, obj: &Obj) -> usize {
    let (d, len) = match obj {
        Obj::Stream { dict, len, .. } => (dict, *len),
        _ => return 0,
    };
    let (w, h) = match size(pdf, d) {
        Some((w, h)) if w as u64 * h as u64 <= MAX_PIXELS => (w, h),
        _ => return 0,
    };
    if filter_names(pdf, d).iter().any(|n| image_filter(n).is_some()) {
        return len;
    }
    let space = image_space(pdf, d, None);
    if let Space::Other(_) = space {
        return 0;
    }
    let pixels = image_bytes(w, h, space.components(), bits(pdf, d)).unwrap_or(usize::MAX);
    let mask = if d.get("SMask").is_some() { w as usize * h as usize } else { 0 };
    pixels.saturating_add(mask)
}

/// Extrae la imagen: formato y bytes del archivo, o el motivo por el que no se entrega.
pub fn extract(pdf: &Pdf, shared: &mut Shared, obj: &Obj) -> Result<(&'static str, Vec<u8>), String> {
    let d = match obj {
        Obj::Stream { dict, .. } => dict,
        _ => return Err("El objeto no es una imagen.".into()),
    };
    let (w, h) = size(pdf, d).ok_or("La imagen no declara su tamano.")?;
    if w as u64 * h as u64 > MAX_PIXELS {
        return Err(format!("Imagen demasiado grande para extraerla ({} x {} pixeles).", w, h));
    }
    // Un espacio de color que no se convierte se rechaza antes de decodificar (`work` la cuenta en 0).
    if !filter_names(pdf, d).iter().any(|n| image_filter(n).is_some()) {
        if let Space::Other(name) = image_space(pdf, d, None) {
            return Err(format!("Espacio de color {} no soportado.", name));
        }
    }
    let (mut data, chain) = pdf.stream_parts(obj).ok_or("No se pudieron leer los datos de la imagen.")?;
    let last = chain.len().saturating_sub(1);
    for (i, (name, parms, raw)) in chain.iter().enumerate() {
        if let Some(kind) = image_filter(name) {
            return match kind {
                "jbig2" => Err(NOTE_JBIG2.into()),
                "tiff" => Ok(("tiff", ccitt_tiff(pdf, &data, w, h, raw.as_ref(), inverted(pdf, d)))),
                k => Ok((k, data)),
            };
        }
        if i == last && matches!(name.as_str(), "FlateDecode" | "Fl") {
            if let Some(png) = passthrough(pdf, shared, d, &data, w, h, parms) {
                return Ok(("png", png));
            }
        }
        data = filters::apply(name, &data, parms).ok_or_else(|| format!("Filtro {} no soportado.", name))?;
    }
    pixels_to_png(pdf, shared, d, data, w, h).map(|p| ("png", p))
}

fn bits(pdf: &Pdf, d: &Dict) -> u8 {
    if flag(pdf, d, "ImageMask") || flag(pdf, d, "IM") {
        return 1;
    }
    match int(pdf, d, "BitsPerComponent").or_else(|| int(pdf, d, "BPC")) {
        Some(b @ (1 | 2 | 4 | 8 | 16)) => b as u8,
        _ => 8,
    }
}

/// Los datos Flate con predictor PNG ya son el contenido de IDAT de un PNG. Se copian solo si un
/// lector de PNG estricto los aceptaria: zlib que pasa `inflate::zlib_strict` (sin diccionario,
/// bloques bien formados, sin bytes entre el final y la suma Adler-32, y esa suma correcta),
/// filas completas y tipos de filtro 0 a 4. Si no, `None` y la imagen se decodifica y se vuelve
/// a comprimir.
fn passthrough(pdf: &Pdf, shared: &mut Shared, d: &Dict, data: &[u8], w: u32, h: u32, p: &Parms) -> Option<Vec<u8>> {
    if d.get("SMask").is_some() || p.predictor < 10 || inverted(pdf, d) {
        return None;
    }
    let bpc = bits(pdf, d);
    let (color, palette) = match image_space(pdf, d, Some(shared)) {
        Space::Gray => (Color::Gray, Vec::new()),
        Space::Rgb if bpc >= 8 => (Color::Rgb, Vec::new()),
        Space::Indexed(pal) if bpc <= 8 => (Color::Palette, pal),
        _ => return None,
    };
    if p.colors != color.channels() as i64 || p.bits != bpc as i64 || p.columns != w as i64 {
        return None;
    }
    let spec = Spec { width: w, height: h, depth: bpc, color, palette: fit_palette(palette, bpc) };
    let row = png::row_bytes(&spec)?.checked_add(1)?;
    // Se descomprime a lo sumo lo que deben medir las filas (y un byte mas, para ver si sobra): un
    // flujo que infla mucho mas no se descomprime entero dos veces, aqui y al decodificarlo.
    let expected = row.checked_mul(h as usize)?;
    let rows = inflate::zlib_strict(data, expected.saturating_add(1))?;
    let valid = rows.len() == expected && rows.chunks(row).all(|r| r[0] <= 4);
    valid.then(|| png::from_idat(&spec, data))
}

/// Paleta para el PNG: exactamente una entrada por cada indice posible con `bpc` bits. Un PNG no
/// admite mas, y un indice sin entrada lo deja invalido: las que el PDF no trae quedan en negro.
fn fit_palette(mut pal: Vec<u8>, bpc: u8) -> Vec<u8> {
    pal.resize((1usize << bpc.min(8)) * 3, 0);
    pal
}

/// Bytes de una imagen de `w` x `h` con `comps` componentes de `bpc` bits, filas completas al
/// byte. `None` si no cabe en la memoria direccionable (en wasm32, 4 GB).
fn image_bytes(w: u32, h: u32, comps: usize, bpc: u8) -> Option<usize> {
    let row = (w as u64).checked_mul(comps as u64)?.checked_mul(bpc as u64)?.checked_add(7)? / 8;
    usize::try_from(row.checked_mul(h as u64)?).ok()
}

/// Los datos decodificados alcanzan para la imagen declarada. Faltando algo (un escaneo
/// cortado), se completa con ceros; faltando mas de la mitad no se extrae: el tamano declarado
/// no debe decidir cuanta memoria se pide si los datos no lo respaldan.
fn enough(len: usize, need: Option<usize>) -> Result<usize, String> {
    match need {
        Some(n) if len >= n - n / 2 => Ok(n),
        Some(n) => Err(format!("Los datos de la imagen estan incompletos ({} de {} bytes).", len, n)),
        None => Err("Imagen demasiado grande para extraerla.".into()),
    }
}

/// Muestra `i` de una fila (sin escalar).
fn sample(row: &[u8], i: usize, bpc: u8) -> u16 {
    match bpc {
        8 => row.get(i).copied().unwrap_or(0) as u16,
        16 => u16::from_be_bytes([row.get(2 * i).copied().unwrap_or(0), row.get(2 * i + 1).copied().unwrap_or(0)]),
        b => {
            let bit = i * b as usize;
            let byte = row.get(bit / 8).copied().unwrap_or(0);
            ((byte >> (8 - b as usize - bit % 8)) & ((1u8 << b) - 1)) as u16
        }
    }
}

/// Lleva una muestra a 0..255.
fn scale(v: u16, bpc: u8) -> u8 {
    match bpc {
        8 => v as u8,
        16 => (v >> 8) as u8,
        b => (v as u32 * 255 / ((1u32 << b) - 1)) as u8,
    }
}

/// Pixeles en 8 bits por canal, gris o RGB.
fn to_8bit(space: &Space, data: &[u8], w: u32, h: u32, bpc: u8, invert: bool) -> Result<(Vec<u8>, Color), String> {
    let (w, h) = (w as usize, h as usize);
    let comps = space.components();
    let row = (w * comps * bpc as usize + 7) / 8;
    let gray = matches!(space, Space::Gray | Space::Tint);
    let mut out = Vec::with_capacity(w * h * if gray { 1 } else { 3 });
    let empty: Vec<u8> = Vec::new();
    for y in 0..h {
        let line = data.get(y * row..((y + 1) * row).min(data.len())).unwrap_or(&empty);
        for x in 0..w {
            let s = |k: usize| {
                let v = scale(sample(line, x * comps + k, bpc), bpc);
                if invert {
                    255 - v
                } else {
                    v
                }
            };
            match space {
                Space::Gray => out.push(s(0)),
                Space::Tint => out.push(255 - s(0)),
                Space::Rgb => out.extend_from_slice(&[s(0), s(1), s(2)]),
                Space::Cmyk => out.extend_from_slice(&cmyk([s(0), s(1), s(2), s(3)])),
                Space::Indexed(pal) => {
                    let i = sample(line, x, bpc) as usize * 3;
                    out.extend_from_slice(pal.get(i..i + 3).unwrap_or(&[0, 0, 0]));
                }
                Space::Other(name) => return Err(format!("Espacio de color {} no soportado.", name)),
            }
        }
    }
    Ok((out, if gray { Color::Gray } else { Color::Rgb }))
}

/// Mascara suave (transparencia) de la imagen, si tiene su mismo tamano. Una mascara que
/// comparten varias imagenes se decodifica con la primera y se guarda hasta la ultima.
fn soft_mask(pdf: &Pdf, shared: &mut Shared, d: &Dict, w: u32, h: u32) -> Option<Rc<Mask>> {
    // Una mascara es un flujo, y los flujos son siempre objetos indirectos.
    let entry = d.get("SMask")?;
    let n = match entry {
        Obj::Ref(n, _) => *n,
        _ => return None,
    };
    let left = shared.mask_uses.get_mut(&n).map_or(0, |c| {
        *c = c.saturating_sub(1);
        *c
    });
    let obj = pdf.resolve(entry);
    let fits = match &*obj {
        Obj::Stream { dict, .. } => size(pdf, dict) == Some((w, h)),
        _ => false,
    };
    // De otro tamano no se usa ni se decodifica.
    let mask = if !fits {
        None
    } else if let Some(m) = shared.masks.get(&n) {
        m.clone()
    } else {
        decode_mask(pdf, &obj, w, h).map(Rc::new)
    };
    if left > 0 && fits {
        shared.masks.insert(n, mask.clone());
    } else if left == 0 {
        shared.masks.remove(&n);
    }
    mask
}

/// Decodifica una mascara suave de `w` x `h` en 8 bits, con su color `/Matte` (0-255) si los
/// colores de la imagen vienen ya mezclados con el.
fn decode_mask(pdf: &Pdf, obj: &Obj, w: u32, h: u32) -> Option<Mask> {
    let md = obj.as_dict()?;
    let data = pdf.stream_data(obj)?;
    // Una mascara sin datos que la respalden se ignora: la imagen sale sin transparencia.
    enough(data.len(), image_bytes(w, h, 1, bits(pdf, md))).ok()?;
    let matte = pdf.array(md, "Matte").map(|m| m.iter().map(|v| v.as_f64().unwrap_or(0.0).clamp(0.0, 1.0) * 255.0).collect());
    to_8bit(&Space::Gray, &data, w, h, bits(pdf, md), inverted(pdf, md)).ok().map(|(alpha, _)| Mask { alpha, matte })
}

fn pixels_to_png(pdf: &Pdf, shared: &mut Shared, d: &Dict, mut data: Vec<u8>, w: u32, h: u32) -> Result<Vec<u8>, String> {
    let bpc = bits(pdf, d);
    let space = image_space(pdf, d, Some(shared));
    let invert = inverted(pdf, d);
    if let Space::Other(name) = &space {
        return Err(format!("Espacio de color {} no soportado.", name));
    }
    enough(data.len(), image_bytes(w, h, space.components(), bpc))?;
    if let Some(mask) = soft_mask(pdf, shared, d, w, h) {
        let (pixels, color) = to_8bit(&space, &data, w, h, bpc, invert)?;
        let ch = color.channels();
        // Con /Matte el color guardado es m + a * (c - m): se recupera c = m + (guardado - m) / a.
        let matte = mask.matte.as_ref().filter(|m| m.len() == ch && matches!(space, Space::Gray | Space::Rgb));
        let mut out = Vec::with_capacity(pixels.len() + mask.alpha.len());
        for (px, &a) in pixels.chunks(ch).zip(&mask.alpha) {
            match matte {
                Some(m) if a > 0 => {
                    let f = 255.0 / a as f64;
                    out.extend(px.iter().zip(m).map(|(&c, &mv)| (mv + (c as f64 - mv) * f).round().clamp(0.0, 255.0) as u8));
                }
                _ => out.extend_from_slice(px),
            }
            out.push(a);
        }
        let color = if color == Color::Gray { Color::GrayAlpha } else { Color::Rgba };
        return encode(&Spec { width: w, height: h, depth: 8, color, palette: Vec::new() }, &out);
    }
    // Sin transparencia se conserva la profundidad original cuando PNG la admite.
    let native = match &space {
        Space::Gray | Space::Tint => Some((Color::Gray, Vec::new())),
        Space::Rgb if bpc >= 8 => Some((Color::Rgb, Vec::new())),
        Space::Indexed(pal) if bpc <= 8 => Some((Color::Palette, fit_palette(pal.clone(), bpc))),
        _ => None,
    };
    match native {
        Some((color, palette)) => {
            let spec = Spec { width: w, height: h, depth: bpc, color, palette };
            // Ya se comprobo que los datos cubren al menos la mitad: el relleno queda acotado.
            let need = png::row_bytes(&spec).and_then(|r| r.checked_mul(h as usize)).ok_or("Imagen demasiado grande para extraerla.")?;
            data.resize(need, 0);
            // Invertir todos los bits invierte cada muestra: v -> maximo - v.
            let flip = (space == Space::Tint) != invert && color == Color::Gray;
            if flip {
                data.iter_mut().for_each(|b| *b = !*b);
            }
            encode(&spec, &data)
        }
        None => {
            let (pixels, color) = to_8bit(&space, &data, w, h, bpc, invert)?;
            encode(&Spec { width: w, height: h, depth: 8, color, palette: Vec::new() }, &pixels)
        }
    }
}

fn encode(spec: &Spec, pixels: &[u8]) -> Result<Vec<u8>, String> {
    png::encode(spec, pixels).ok_or_else(|| "Los datos de la imagen estan incompletos.".to_string())
}

impl PartialEq for Space {
    fn eq(&self, other: &Space) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// TIFF de una tira con los datos de fax tal cual (Grupo 3 o 4 segun `K`).
fn ccitt_tiff(pdf: &Pdf, data: &[u8], w: u32, h: u32, parms: Option<&Dict>, invert: bool) -> Vec<u8> {
    let get = |k: &str| parms.and_then(|p| pdf.get(p, k).as_i64());
    let on = |k: &str| parms.map_or(false, |p| flag(pdf, p, k));
    let k = get("K").unwrap_or(0);
    let compression: u32 = if k < 0 { 4 } else { 3 };
    // En PDF, sin BlackIs1 los bits 0 son negros: en TIFF eso es "BlackIsZero" (1).
    let black_is_zero = !on("BlackIs1") != invert;
    let mut entries: Vec<(u16, u16, u32)> = vec![
        (256, 4, w),
        (257, 4, h),
        (258, 3, 1),
        (259, 3, compression),
        (262, 3, black_is_zero as u32),
        (273, 4, 0), // desplazamiento de la tira: se completa abajo
        (277, 3, 1),
        (278, 4, h),
        (279, 4, data.len() as u32),
    ];
    if compression == 3 {
        entries.push((292, 4, (k > 0) as u32 | if on("EncodedByteAlign") { 4 } else { 0 }));
    } else {
        entries.push((293, 4, 0));
    }
    let offset = 8 + 2 + 12 * entries.len() as u32 + 4;
    let mut out = Vec::with_capacity(offset as usize + data.len());
    out.extend_from_slice(b"II*\0");
    out.extend_from_slice(&8u32.to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    for (tag, kind, value) in entries {
        let value = if tag == 273 { offset } else { value };
        out.extend_from_slice(&tag.to_le_bytes());
        out.extend_from_slice(&kind.to_le_bytes());
        out.extend_from_slice(&1u32.to_le_bytes());
        out.extend_from_slice(&value.to_le_bytes());
    }
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(data);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn muestras() {
        assert_eq!(sample(&[0b1011_0010], 0, 1), 1);
        assert_eq!(sample(&[0b1011_0010], 1, 1), 0);
        assert_eq!(sample(&[0b1011_0010], 1, 2), 0b11);
        assert_eq!(sample(&[0xAB, 0xCD], 1, 4), 0xB);
        assert_eq!(sample(&[0x12, 0x34], 0, 16), 0x1234);
        assert_eq!(scale(3, 2), 255);
        assert_eq!(scale(1, 1), 255);
        assert_eq!(cmyk([0, 0, 0, 0]), [255, 255, 255]);
        assert_eq!(cmyk([255, 0, 0, 0]), [0, 255, 255]);
    }

    #[test]
    fn ubicacion() {
        // Imagen de 100 x 50 puntos dibujada en (72, 600) en una pagina carta vertical.
        let m = [100.0, 0.0, 0.0, 50.0, 72.0, 600.0];
        assert_eq!(place(m, [0.0, 0.0, 612.0, 792.0], 0), (72.0, 142.0, 100.0, 50.0));
        // La misma pagina girada 90 grados: se lee apaisada.
        let (x, y, w, h) = place(m, [0.0, 0.0, 612.0, 792.0], 90);
        assert_eq!((w, h), (50.0, 100.0));
        assert_eq!((x, y), (600.0, 72.0));
    }
}
