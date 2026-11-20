//! Lector de presentaciones PowerPoint (.pptx, formato PresentationML).
//!
//! Un .pptx es un ZIP con una parte XML por diapositiva. De cada una se toma, en orden de
//! lectura (el titulo primero y luego de arriba hacia abajo y de izquierda a derecha):
//! - el titulo, como `heading` de nivel 1;
//! - el texto de los marcadores de contenido como items de lista (con su nivel y su vineta o
//!   numeracion) y el de los cuadros de texto como parrafos;
//! - las tablas, con sus celdas combinadas, y las imagenes con su texto alternativo.
//!
//! Cada bloque lleva en `page` el numero de su diapositiva. Las notas del orador van aparte, en
//! `notes`, con el numero de la diapositiva en `label`. Las diapositivas ocultas se omiten, y una
//! diapositiva referida varias veces se lee una vez. El texto de toda la presentacion tiene un
//! tope (`MAX_TEXT`), igual que en Excel.
//! No se lee el texto de graficos ni de SmartArt.

use crate::doc::*;
use crate::docx::{letters, part_rels, properties, relationships, resolve, roman, MAX_TARGET};
use crate::symbols::{classify, translate, translate_str, SymbolFont};
use crate::xml::{Ev, Tag, Xml};
use crate::zipread::{part_key, ZipArchive};
use std::collections::{HashMap, HashSet};

/// Tope de diapositivas distintas que se leen.
const MAX_SLIDES: usize = 10_000;

/// Texto de un tramo segun sus fuentes: con `a:latin` en una fuente de simbolos se dibuja todo
/// con ella; con `a:sym` solo los caracteres de simbolo (area privada U+F020..U+F0FF).
fn run_text(latin: SymbolFont, sym: SymbolFont, s: &str) -> String {
    if latin != SymbolFont::None {
        translate_str(latin, s)
    } else if sym != SymbolFont::None {
        s.chars().map(|c| if (0xF020..=0xF0FF).contains(&(c as u32)) { translate(sym, c) } else { c }).collect()
    } else {
        translate_str(SymbolFont::None, s)
    }
}

/// Un elemento de la diapositiva con su posicion, para ordenarlos como se leen.
struct Item {
    title: bool,
    y: i64,
    x: i64,
    blocks: Vec<Block>,
}

/// Rol del texto segun el marcador de posicion de la forma.
#[derive(Clone, Copy, PartialEq)]
enum Role {
    Title,
    Subtitle,
    /// Marcador de contenido: sus parrafos son items de lista salvo que digan lo contrario.
    Body,
    /// Cuadro de texto libre: sus parrafos son parrafos salvo que traigan vineta propia.
    Text,
    /// Numero de diapositiva, fecha, pie: no es contenido.
    Skip,
}

fn role_of(ph: Option<(String, bool)>) -> Role {
    match ph {
        None => Role::Text,
        Some((t, _)) => match t.as_str() {
            "title" | "ctrTitle" => Role::Title,
            "subTitle" => Role::Subtitle,
            "sldNum" | "dt" | "ftr" | "hdr" | "sldImg" => Role::Skip,
            // Sin tipo (solo idx), "body" u "obj": contenido.
            _ => Role::Body,
        },
    }
}

/// Numeracion automatica de PowerPoint (`a:buAutoNum type`) para el numero `n`.
fn auto_number(kind: &str, n: i64) -> String {
    let (core, wrap) = if let Some(base) = kind.strip_suffix("ParenBoth") {
        (base, "()")
    } else if let Some(base) = kind.strip_suffix("ParenR") {
        (base, ")")
    } else if let Some(base) = kind.strip_suffix("Period") {
        (base, ".")
    } else if let Some(base) = kind.strip_suffix("Plain") {
        (base, "")
    } else {
        (kind, ".")
    };
    let num = match core {
        "alphaLc" => letters(n, false),
        "alphaUc" => letters(n, true),
        "romanLc" => roman(n, false),
        "romanUc" => roman(n, true),
        _ => n.to_string(),
    };
    match wrap {
        "()" => format!("({})", num),
        w => format!("{}{}", num, w),
    }
}

struct Para {
    text: String,
    runs: Vec<Run>,
    level: u8,
    /// Vineta explicita: None = la del marcador; Some("") = sin vineta.
    bullet: Option<String>,
    auto: Option<(String, i64)>,
}

/// Una copia de `s` si cabe entera en el limite de texto de la lectura (`budget::take_text`), que
/// la descuenta; si no, nada. El destino de un enlace o el nombre de una imagen se copia en cada
/// tramo o imagen que lo cita: sin descontarlo, un destino citado por un millon de tramos de una
/// letra ocuparia mil veces mas que el texto.
fn charged(s: &str) -> Option<String> {
    let kept = crate::budget::take_text(s);
    (kept.len() == s.len()).then(|| s.to_string())
}

/// Lee un `a:p` (ya abierto) hasta su cierre.
fn paragraph(x: &mut Xml, rels: &HashMap<String, String>) -> Para {
    let mut p = Para { text: String::new(), runs: Vec::new(), level: 0, bullet: None, auto: None };
    let mut bu_font = SymbolFont::None;
    let mut cur = Run::default();
    let mut in_t = false;
    // Fuente del tramo (`a:latin`) y de sus caracteres de simbolo (`a:sym`).
    let (mut latin, mut sym) = (SymbolFont::None, SymbolFont::None);
    while let Some(ev) = x.next() {
        match ev {
            Ev::Start(t) => match t.name {
                "a:pPr" => {
                    p.level = t.attr("lvl").and_then(|v| v.parse::<u8>().ok()).unwrap_or(0).min(8);
                }
                "a:buNone" => p.bullet = Some(String::new()),
                "a:buFont" => bu_font = classify(&t.attr("typeface").unwrap_or_default()),
                "a:buChar" => p.bullet = Some(translate_str(bu_font, &t.attr("char").unwrap_or_default())),
                "a:buAutoNum" => {
                    // PowerPoint admite inicios de 1 a 32767.
                    let start = t.attr("startAt").and_then(|v| v.parse::<i64>().ok()).unwrap_or(1).clamp(1, 32_767);
                    p.auto = Some((t.attr("type").unwrap_or_else(|| "arabicPeriod".into()), start));
                }
                "a:r" | "a:fld" => {
                    cur = Run::default();
                    latin = SymbolFont::None;
                    sym = SymbolFont::None;
                }
                "a:rPr" => {
                    cur.bold = t.on_flag("b");
                    cur.italic = t.on_flag("i");
                    cur.underline = matches!(t.attr("u").as_deref(), Some(v) if v != "none");
                    cur.strike = matches!(t.attr("strike").as_deref(), Some(v) if v != "noStrike");
                }
                "a:latin" => latin = classify(&t.attr("typeface").unwrap_or_default()),
                "a:sym" => sym = classify(&t.attr("typeface").unwrap_or_default()),
                "a:hlinkClick" => cur.link = t.attr("r:id").and_then(|id| rels.get(&id)).and_then(|l| charged(l)),
                "a:t" if !t.empty => in_t = true,
                "a:br" => {
                    p.text.push('\n');
                    p.runs.push(Run { text: "\n".into(), ..Run::default() });
                }
                "a:endParaRPr" | "a:ea" | "a:cs" | "a:solidFill" => {}
                _ => {}
            },
            Ev::Text(s) if in_t => {
                let s = run_text(latin, sym, &s);
                p.text.push_str(&s);
                cur.text.push_str(&s);
            }
            Ev::End("a:t") => in_t = false,
            Ev::End("a:r") | Ev::End("a:fld") => {
                if !cur.text.is_empty() {
                    p.runs.push(std::mem::take(&mut cur));
                }
            }
            Ev::End("a:p") => break,
            _ => {}
        }
    }
    p
}

/// Texto de un `p:txBody` o `a:txBody` (ya abierto) como bloques segun el rol de la forma.
fn text_body(x: &mut Xml, end: &str, role: Role, rels: &HashMap<String, String>, slide: u32) -> Vec<Block> {
    let mut out = Vec::new();
    // Contadores de numeracion automatica por nivel (se reinician al cambiar de tipo).
    let mut counters: [Option<(String, i64)>; 9] = Default::default();
    while let Some(ev) = x.next() {
        match ev {
            Ev::Start(t) if t.name == "a:p" && !t.empty => {
                let p = paragraph(x, rels);
                let text = p.text.replace('\n', " ").split_whitespace().collect::<Vec<_>>().join(" ");
                if text.is_empty() || role == Role::Skip {
                    continue;
                }
                let runs = if p.runs.iter().any(|r| r.bold || r.italic || r.link.is_some()) { p.runs } else { Vec::new() };
                let mut b = Block { text, runs, page: Some(slide), ..Block::default() };
                match role {
                    Role::Title => {
                        b.kind = Kind::Heading;
                        b.level = 1;
                    }
                    Role::Subtitle => b.kind = Kind::Paragraph,
                    _ => {
                        let explicit = p.bullet.as_deref().map(|s| !s.is_empty()).unwrap_or(false) || p.auto.is_some();
                        let suppressed = p.bullet.as_deref() == Some("");
                        if suppressed || (role == Role::Text && !explicit) {
                            b.kind = Kind::Paragraph;
                        } else {
                            b.kind = Kind::ListItem;
                            b.level = p.level;
                            if let Some((kind, start)) = &p.auto {
                                let c = &mut counters[p.level as usize];
                                if !matches!(c, Some(c) if c.0 == *kind) {
                                    *c = Some((kind.clone(), *start - 1));
                                }
                                if let Some(c) = c {
                                    c.1 = c.1.saturating_add(1);
                                    b.marker = Some(auto_number(kind, c.1));
                                }
                                b.ordered = true;
                            } else {
                                b.marker = Some(p.bullet.clone().filter(|s| !s.is_empty()).unwrap_or_else(|| "\u{2022}".into()));
                            }
                            // Un nivel menos profundo reinicia la numeracion de los mas profundos.
                            for deeper in counters.iter_mut().skip(p.level as usize + 1) {
                                *deeper = None;
                            }
                        }
                    }
                }
                out.push(b);
            }
            Ev::End(n) if n == end => break,
            _ => {}
        }
    }
    out
}

/// Tabla (`a:tbl`, ya abierta).
fn table(x: &mut Xml, rels: &HashMap<String, String>, slide: u32) -> Block {
    let mut rows: Vec<Vec<Cell>> = Vec::new();
    while let Some(ev) = x.next() {
        match ev {
            Ev::Start(t) if t.name == "a:tr" && !t.empty => rows.push(Vec::new()),
            Ev::Start(t) if t.name == "a:tc" => {
                let colspan = t.attr("gridSpan").and_then(|v| v.parse().ok()).unwrap_or(1u32).max(1);
                let rowspan = t.attr("rowSpan").and_then(|v| v.parse().ok()).unwrap_or(1u32).max(1);
                let h_merge = t.on_flag("hMerge");
                let v_merge = t.on_flag("vMerge");
                let mut text = String::new();
                if !t.empty {
                    let mut parts: Vec<String> = Vec::new();
                    while let Some(e2) = x.next() {
                        match e2 {
                            Ev::Start(p) if p.name == "a:p" && !p.empty => {
                                let para = paragraph(x, rels);
                                let s = para.text.trim().to_string();
                                if !s.is_empty() {
                                    parts.push(s);
                                }
                            }
                            Ev::End("a:tc") => break,
                            _ => {}
                        }
                    }
                    text = parts.join("\n");
                }
                // Celda cubierta por una combinacion horizontal: ya la representa la de la izquierda.
                if h_merge {
                    continue;
                }
                if let Some(r) = rows.last_mut() {
                    r.push(Cell { text: if v_merge { String::new() } else { text }, colspan, rowspan, merged: v_merge });
                }
            }
            Ev::End("a:tbl") => break,
            _ => {}
        }
    }
    Block { kind: Kind::Table, rows, page: Some(slide), ..Block::default() }
}

trait Flag {
    fn on_flag(&self, key: &str) -> bool;
}

impl Flag for Tag<'_> {
    fn on_flag(&self, key: &str) -> bool {
        matches!(self.attr(key).as_deref(), Some("1") | Some("true"))
    }
}

/// Lee las formas de un `p:spTree` o `p:grpSp` (ya abierto) hasta `end`.
fn shapes(x: &mut Xml, end: &str, rels: &HashMap<String, String>, slide: u32, depth: usize) -> Vec<Item> {
    let mut items: Vec<Item> = Vec::new();
    // Datos de la forma en curso.
    let mut ph: Option<(String, bool)> = None;
    let (mut ox, mut oy) = (i64::MAX, i64::MAX);
    let mut alt = String::new();
    let mut pending: Vec<Block> = Vec::new();
    let mut in_shape = 0usize;
    while let Some(ev) = x.next() {
        match ev {
            Ev::Start(t) => match t.name {
                "p:sp" | "p:pic" | "p:graphicFrame" | "p:cxnSp" if !t.empty => {
                    in_shape += 1;
                    ph = None;
                    ox = i64::MAX;
                    oy = i64::MAX;
                    alt.clear();
                    pending.clear();
                }
                "p:grpSp" if !t.empty && depth < 16 => {
                    // Grupo: sus formas se ordenan entre si y el grupo cuenta como un elemento.
                    let mut inner = shapes(x, "p:grpSp", rels, slide, depth + 1);
                    if !inner.is_empty() {
                        sort(&mut inner);
                        let (gy, gx) = (inner.iter().map(|i| i.y).min().unwrap_or(0), inner.iter().map(|i| i.x).min().unwrap_or(0));
                        let title = inner.iter().any(|i| i.title);
                        items.push(Item { title, y: gy, x: gx, blocks: inner.into_iter().flat_map(|i| i.blocks).collect() });
                    }
                }
                // Contenido alternativo: se lee la opcion preferida (mc:Choice), no la de respaldo.
                "mc:Fallback" if !t.empty => x.skip("mc:Fallback"),
                "p:cNvPr" => alt =t.attr("descr").or_else(|| t.attr("title")).unwrap_or_default(),
                "p:ph" => ph = Some((t.attr("type").unwrap_or_default(), t.attr("idx").is_some())),
                "a:off" if ox == i64::MAX => {
                    ox = t.attr("x").and_then(|v| v.parse().ok()).unwrap_or(0);
                    oy = t.attr("y").and_then(|v| v.parse().ok()).unwrap_or(0);
                }
                "p:txBody" if !t.empty && in_shape > 0 => {
                    let role = role_of(ph.clone());
                    pending.extend(text_body(x, "p:txBody", role, rels, slide));
                }
                "a:tbl" if !t.empty => pending.push(table(x, rels, slide)),
                "a:blip" => {
                    let name = t.attr("r:embed").and_then(|id| rels.get(&id)).and_then(|p| charged(p.rsplit('/').next().unwrap_or(p)));
                    pending.push(Block { kind: Kind::Image, name, text: alt.trim().to_string(), page: Some(slide), ..Block::default() });
                }
                _ => {}
            },
            Ev::End("p:sp" | "p:pic" | "p:graphicFrame" | "p:cxnSp") => {
                in_shape = in_shape.saturating_sub(1);
                if !pending.is_empty() {
                    let title = role_of(ph.clone()) == Role::Title;
                    items.push(Item { title, y: if oy == i64::MAX { 0 } else { oy }, x: if ox == i64::MAX { 0 } else { ox }, blocks: std::mem::take(&mut pending) });
                }
            }
            Ev::End(n) if n == end => break,
            _ => {}
        }
    }
    items
}

/// Orden de lectura: el titulo primero; luego por filas (de arriba hacia abajo, con una tolerancia
/// para formas casi alineadas) y dentro de una fila de izquierda a derecha.
fn sort(items: &mut [Item]) {
    // 1/40 de una diapositiva estandar (6858000 EMU de alto) separa una fila de otra.
    const ROW: i64 = 171_450;
    items.sort_by(|a, b| b.title.cmp(&a.title).then((a.y / ROW).cmp(&(b.y / ROW))).then(a.x.cmp(&b.x)));
}

fn slide_blocks(zip: &ZipArchive, path: &str, number: u32) -> Option<(Vec<Block>, bool)> {
    let src = zip.read_text(path)?;
    let rels = part_rels(zip, path, MAX_TARGET);
    let mut x = Xml::new(&src);
    let mut hidden = false;
    let mut items = Vec::new();
    while let Some(ev) = x.next() {
        if let Ev::Start(t) = ev {
            match t.name {
                "p:sld" => hidden = matches!(t.attr("show").as_deref(), Some("0") | Some("false")),
                "p:spTree" if !t.empty => items = shapes(&mut x, "p:spTree", &rels, number, 0),
                _ => {}
            }
        }
    }
    sort(&mut items);
    Some((items.into_iter().flat_map(|i| i.blocks).collect(), hidden))
}

/// Notas del orador de una diapositiva: el texto de su marcador de cuerpo. Unas notas ya leidas
/// (por su clave o por la posicion de sus datos en el ZIP, como las diapositivas) no se leen de
/// nuevo: muchas diapositivas que citan unas mismas notas grandes no las descomprimen cada vez.
fn notes(zip: &ZipArchive, slide_path: &str, number: u32, seen: &mut (HashSet<String>, HashSet<usize>)) -> Option<Block> {
    let dir = slide_path.rfind('/').map_or("", |i| &slide_path[..i]);
    let file = slide_path.rsplit('/').next().unwrap_or(slide_path);
    let rels_path = format!("{}/_rels/{}.rels", dir, file);
    let target = relationships(zip, &rels_path, MAX_TARGET).into_iter().find(|(_, kind, _, _)| kind == "notesSlide").map(|(_, _, t, _)| resolve(dir, &t))?;
    let (key, at) = (part_key(&target), zip.data_at(&target));
    if !seen.0.insert(key) || at.map_or(false, |a| !seen.1.insert(a)) {
        return None;
    }
    let src = zip.read_text(&target)?;
    let rels = part_rels(zip, &target, MAX_TARGET);
    let mut x = Xml::new(&src);
    let items = loop {
        match x.next()? {
            Ev::Start(t) if t.name == "p:spTree" && !t.empty => break shapes(&mut x, "p:spTree", &rels, number, 0),
            _ => {}
        }
    };
    // El marcador de cuerpo de las notas sale como lista (es un "body"): se junta como texto.
    let text: Vec<String> = items.into_iter().flat_map(|i| i.blocks).filter(|b| b.kind != Kind::Image).map(|b| b.text).collect();
    let text = text.join("\n");
    if text.trim().is_empty() {
        None
    } else {
        Some(Block { kind: Kind::Paragraph, text, label: Some(number.to_string()), page: Some(number), ..Block::default() })
    }
}

pub fn read(data: &[u8]) -> Result<Document, String> {
    let zip = ZipArchive::open(data).map_err(|_| "El archivo .pptx esta danado o incompleto: no se pudo abrir como ZIP.".to_string())?;
    let main = relationships(&zip, "_rels/.rels", MAX_TARGET)
        .into_iter()
        .find(|(_, kind, _, _)| kind == "officeDocument")
        .map(|(_, _, target, _)| resolve("", &target))
        .filter(|p| zip.find(p).is_some())
        .or_else(|| zip.find("ppt/presentation.xml").map(|e| e.name.clone()))
        .ok_or("El archivo no es una presentacion de PowerPoint (.pptx): no contiene ppt/presentation.xml.")?;
    let dir = main.rfind('/').map_or(String::new(), |i| main[..i].to_string());
    let rels = part_rels(&zip, &main, MAX_TARGET);
    let src = zip.read_text(&main).ok_or("No se pudo leer la presentacion.")?;

    // Diapositivas en el orden de la presentacion. Cada relacion se toma una vez (la primera
    // referencia): copiar su ruta por cada `sldId` que la cita multiplicaria la memoria.
    let mut order: Vec<String> = Vec::new();
    let mut refs = 0usize;
    let mut ids = HashSet::new();
    let mut x = Xml::new(&src);
    while let Some(ev) = x.next() {
        if let Ev::Start(t) = ev {
            if t.name == "p:sldId" {
                if let Some((id, p)) = t.attr("r:id").and_then(|id| rels.get(&id).map(|p| (id, p))) {
                    refs += 1;
                    if ids.insert(id) {
                        order.push(p.clone());
                    }
                }
            }
        }
    }
    if order.is_empty() {
        // Sin lista de diapositivas: las que haya, por numero (puede haber huecos). `find` usa un
        // indice, asi que probar cada numero hasta el de entradas del ZIP es lineal.
        let base = if dir.is_empty() { "ppt" } else { &dir };
        order = (1..=zip.entries.len()).map(|n| format!("{}/slides/slide{}.xml", base, n)).filter(|p| zip.find(p).is_some()).collect();
    }

    let mut doc = Document { kind: "pptx", ..Document::default() };
    properties(&zip, &mut doc);
    // Una diapositiva referida varias veces se lee una vez, en su primera posicion. La clave es
    // la de `find`: "slides/Slide1.xml" y "//ppt/slides/slide1.xml" son la misma parte. Tambien
    // lo son dos nombres distintos cuyas entradas del ZIP apuntan a los mismos datos.
    let total = refs.max(order.len());
    let mut seen = HashSet::new();
    let mut seen_at = HashSet::new();
    order.retain(|p| {
        let (key, at) = (part_key(p), zip.data_at(p));
        if seen.contains(&key) || at.map_or(false, |a| seen_at.contains(&a)) {
            return false;
        }
        seen.insert(key);
        seen_at.extend(at);
        true
    });
    if order.len() < total {
        doc.warnings.push(format!("Se omitieron {} referencias repetidas a diapositivas ya leidas.", total - order.len()));
    }
    if order.len() > MAX_SLIDES {
        order.truncate(MAX_SLIDES);
        doc.warnings.push(format!("La presentacion tiene mas de {} diapositivas; se leyeron solo las primeras.", MAX_SLIDES));
    }
    doc.page_count = Some(order.len());
    let mut notes_list = Vec::new();
    let mut hidden = 0usize;
    let mut text_left = MAX_TEXT;
    let mut seen_notes = (HashSet::new(), HashSet::new());
    for (i, path) in order.iter().enumerate() {
        let n = i as u32 + 1;
        match slide_blocks(&zip, path, n) {
            Some((_, true)) => hidden += 1,
            Some((blocks, false)) => {
                let note = notes(&zip, path, n, &mut seen_notes);
                // Una misma parte puede figurar en el ZIP con muchos nombres, y muchas
                // diapositivas pueden compartir las mismas notas: el texto total tiene tope.
                let size: usize = blocks.iter().chain(note.as_ref()).map(text_size).sum();
                if size > text_left {
                    doc.warnings.push(format!("La presentacion supera el tope de {} MB de texto; se omitieron las diapositivas desde la {}.", MAX_TEXT >> 20, n));
                    break;
                }
                text_left -= size;
                doc.content.extend(blocks);
                notes_list.extend(note);
            }
            None => doc.warnings.push(format!("No se pudo leer la diapositiva {}.", n)),
        }
    }
    if !notes_list.is_empty() {
        doc.extras.push(("notes", notes_list));
    }
    if hidden > 0 {
        doc.warnings.push(format!("Se omitieron {} diapositivas ocultas.", hidden));
    }
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeracion_automatica() {
        assert_eq!(auto_number("arabicPeriod", 3), "3.");
        assert_eq!(auto_number("arabicParenR", 2), "2)");
        assert_eq!(auto_number("alphaLcParenBoth", 1), "(a)");
        assert_eq!(auto_number("romanUcPeriod", 4), "IV.");
        assert_eq!(auto_number("alphaUcPeriod", 27), "AA.");
        assert_eq!(auto_number("alphaLcPeriod", 32_767), "32767.");
    }
}
