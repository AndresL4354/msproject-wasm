//! Markdown y fragmentos: el documento listo para un modelo de lenguaje o un buscador.
//!
//! - `to_markdown` arma Markdown (con tablas GFM) desde los bloques: titulos con `#`, listas con
//!   su numeracion o vineta, tablas con sus celdas combinadas, notas al pie como `[^1]`, y en Word
//!   negrita, cursiva y enlaces. En PDF marca el inicio de cada pagina con `<!-- pagina N -->`.
//! - `chunks` corta ese Markdown en fragmentos de un tamano maximo para indexar o enviar a un
//!   modelo: cada fragmento empieza en un titulo cuando se puede, nunca corta un bloque (salvo que
//!   el bloque solo ya sea mas grande que el fragmento) y trae la ruta de titulos de su seccion y
//!   las paginas que cubre. Las tablas largas se parten por filas repitiendo el encabezado.
//!
//! El texto del documento no se vuelve HTML, enlaces, titulos ni listas (`*x*`, `_x_` y `~~x~~` no
//! se escapan y pueden salir como enfasis; con GFM, una direccion `https://...` o `www....` escrita
//! en el texto sale como enlace automatico): se escapan `<`, `[`, `]`, `` ` ``, las entidades
//! (`&lt;`), las barras invertidas y lo que al inicio de una linea (tambien tras un `\r` solo)
//! abriria un titulo, una cita, una lista o una linea horizontal. Los enlaces de Word solo salen si
//! son `http`, `https` o `mailto`. Esto evita que el texto se lea como marca, pero no reemplaza a un
//! sanitizador: para mostrar el Markdown como HTML hay que limpiar igual el HTML resultante.

use crate::doc::{Block, Cell, Document, Kind, MAX_COLSPAN};
use crate::json::{self, Obj};
use std::collections::HashSet;

/// Tamano por defecto de un fragmento, en caracteres (unos 500 tokens en castellano).
pub const DEFAULT_CHUNK: usize = 2000;
/// Una seccion mas corta que esto no cierra su fragmento: se junta con la siguiente.
const MIN_CHUNK: usize = 200;
/// Largo maximo de cada titulo en la ruta (`path`) de un fragmento; el texto lo trae completo.
const MAX_PATH_CHARS: usize = 200;
/// Columnas de una tabla en Markdown. Las celdas que pasan de ahi se juntan en la ultima columna:
/// asi una fila con miles de celdas no obliga a rellenar todas las demas filas hasta ese ancho.
const MAX_COLS: usize = 64;
/// Largo maximo, en caracteres, de una etiqueta de nota que sale como `[^etiqueta]`.
const MAX_LABEL: usize = 16;

/// Escapa lo que Markdown leeria como marca o HTML dentro de una linea de texto: `<` (etiquetas y
/// enlaces automaticos), `[` y `]` (enlaces, imagenes y definiciones), `` ` `` (codigo), `&` cuando
/// forma una entidad, y las barras invertidas que escaparian el caracter siguiente.
fn escape_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    let mut it = text.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        match c {
            // Al final tambien: el texto puede quedar pegado a una marca ("<br>" en una celda).
            '\\' if it.peek().map_or(true, |(_, n)| n.is_ascii_punctuation()) => out.push_str("\\\\"),
            '<' | '[' | ']' | '`' => {
                out.push('\\');
                out.push(c);
            }
            '&' if is_entity(&text[i + 1..]) => out.push_str("\\&"),
            _ => out.push(c),
        }
    }
    out
}

/// `rest` empieza como el resto de una entidad HTML (`lt;`, `#60;`, `#x3C;`) tras el `&`.
fn is_entity(rest: &str) -> bool {
    let body = rest.strip_prefix('#').unwrap_or(rest);
    let n = body.bytes().take(32).take_while(u8::is_ascii_alphanumeric).count();
    n > 0 && body.as_bytes().get(n) == Some(&b';')
}

/// Escapa lo que Markdown leeria como marca al inicio de un texto (titulo, cita, lista, linea
/// horizontal, subrayado de titulo, bloque de codigo). Quita la sangria: cuatro espacios serian
/// codigo. Mira solo la primera linea: en un trozo de varias (un fragmento cortado) las demas ya
/// vienen escapadas, y mirarlo entero dejaria pasar "1." o "---" solos en la primera.
fn escape_start(text: &str) -> String {
    let t = text.trim_start();
    let end = t.find(['\r', '\n']).unwrap_or(t.len());
    let mut out = escape_line(&t[..end]);
    out.push_str(&t[end..]);
    out
}

/// `escape_start` de una sola linea.
fn escape_line(t: &str) -> String {
    let first = t.chars().next();
    let only = |c: char, min: usize| t.chars().all(|x| x == c || x == ' ' || x == '\t') && t.chars().filter(|x| *x == c).count() >= min;
    let needs = match first {
        Some('#') | Some('>') => true,
        Some('=') => only('=', 1),
        // "--" bajo una linea de parrafo la vuelve titulo (subrayado), como "-" o "---".
        Some('-') if only('-', 1) => true,
        Some('~') => t.starts_with("~~~"),
        Some('-') | Some('+') | Some('*') | Some('_') if only(first.unwrap_or(' '), 3) => true,
        Some('-') | Some('+') | Some('*') => t.chars().nth(1).map_or(true, |c| c == ' ' || c == '\t'),
        Some(c) if c.is_ascii_digit() => {
            // "1. " y "1) " (tambien con tabulador) abren una lista numerada. En CommonMark "\1" no
            // es un escape (la barra se veria): se escapa el punto o el parentesis, "1\. Objetivo".
            let digits = t.bytes().take_while(u8::is_ascii_digit).count();
            let rest = &t[digits..];
            let opens = |p: char| rest.starts_with(p) && rest[1..].starts_with([' ', '\t']);
            let list = (opens('.') || opens(')') || rest == "." || rest == ")") && digits <= 9;
            return if list { format!("{}\\{}", &t[..digits], rest) } else { t.to_string() };
        }
        _ => false,
    };
    if needs {
        format!("\\{}", t)
    } else {
        t.to_string()
    }
}

/// Escapa el inicio de cada linea de un texto de varias lineas. Un "\r" solo tambien termina una
/// linea en CommonMark: todos los finales de linea quedan como "\n".
fn escape_lines(text: &str) -> String {
    text.replace("\r\n", "\n").split(['\r', '\n']).map(escape_start).collect::<Vec<_>>().join("\n")
}

/// Un enlace que se puede publicar: solo `http`, `https` y `mailto`, con los caracteres que
/// cortarian el destino de Markdown codificados. Cualquier otro esquema (`javascript:`, `data:`,
/// `file:`...) o un destino interno del documento queda fuera.
fn safe_url(link: &str) -> Option<String> {
    let link = link.trim();
    let (scheme, _) = link.split_once(':')?;
    if !["http", "https", "mailto"].iter().any(|s| scheme.eq_ignore_ascii_case(s)) {
        return None;
    }
    Some(encode_url(link))
}

fn encode_url(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            ' ' => out.push_str("%20"),
            '(' => out.push_str("%28"),
            ')' => out.push_str("%29"),
            '<' => out.push_str("%3C"),
            '>' => out.push_str("%3E"),
            '"' => out.push_str("%22"),
            '\\' => out.push_str("%5C"),
            '`' => out.push_str("%60"),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out
}

/// Texto con las citas "[n]" de notas que existen como "[^n]" (nota al pie de Markdown) y todo lo
/// demas escapado. Un "(" o un ":" pegado a una cita va escapado: para un lector sin notas al pie,
/// "[^1](javascript:...)" es un enlace y "[^1]: ..." al inicio de una linea, una definicion.
fn text_md(text: &str, labels: &HashSet<String>) -> String {
    if labels.is_empty() || !text.contains('[') {
        return escape_text(text);
    }
    let mut out = String::with_capacity(text.len() + 8);
    let mut rest = text;
    while let Some(i) = rest.find('[') {
        out.push_str(&escape_text(&rest[..i]));
        let after = &rest[i + 1..];
        // El "]" se busca solo en el largo que puede tener una etiqueta: buscarlo en todo el resto
        // haria cuadratico un texto con muchos "[" sin cerrar.
        let close = after.char_indices().take(MAX_LABEL + 1).find(|(_, c)| *c == ']').map(|(j, _)| j);
        match close {
            Some(j) if labels.contains(&after[..j]) => {
                out.push_str("[^");
                out.push_str(&after[..j]);
                out.push(']');
                rest = &after[j + 1..];
                if rest.starts_with(['(', ':']) {
                    out.push('\\');
                    out.push_str(&rest[..1]);
                    rest = &rest[1..];
                }
            }
            _ => {
                out.push_str("\\[");
                rest = after;
            }
        }
    }
    out.push_str(&escape_text(rest));
    out
}

/// Texto de un bloque con su formato de Word (negrita, cursiva, enlaces) si lo trae.
fn inline(b: &Block, labels: &HashSet<String>) -> String {
    let joined: String = b.runs.iter().map(|r| r.text.as_str()).collect();
    // El texto del bloque viene recortado y los tramos no: se comparan sin los espacios de los bordes.
    if b.runs.is_empty() || joined.trim() != b.text.trim() || !b.runs.iter().any(|r| r.bold || r.italic || r.link.is_some()) {
        return text_md(&b.text, labels);
    }
    let mut out = String::new();
    for r in &b.runs {
        // Los espacios de los bordes van fuera de la marca: "** hola**" no es negrita en Markdown.
        let core = r.text.trim();
        if core.is_empty() {
            out.push_str(&r.text);
            continue;
        }
        let lead = &r.text[..r.text.len() - r.text.trim_start().len()];
        let tail = &r.text[r.text.trim_end().len()..];
        let mut s = text_md(core, labels);
        // Una cita al final del tramo anterior y un "(" o ":" al inicio de este (los tramos se
        // separan tambien por el subrayado, que no deja marca): se escapa igual que en un tramo.
        if lead.is_empty() && out.ends_with(']') && s.starts_with(['(', ':']) {
            s.insert(0, '\\');
        }
        if r.italic {
            s = format!("*{}*", s);
        }
        if r.bold {
            s = format!("**{}**", s);
        }
        if let Some(url) = r.link.as_deref().and_then(safe_url) {
            s = format!("[{}]({})", s, url);
        }
        out.push_str(lead);
        out.push_str(&s);
        out.push_str(tail);
    }
    out.trim().to_string()
}

fn cell_text(c: &Cell) -> String {
    if c.merged {
        return String::new();
    }
    // CommonMark tambien corta la linea en un "\r" solo; las lineas vacias se descartan.
    c.text
        .trim()
        .split(['\r', '\n'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| escape_text(s).replace('|', "\\|"))
        .collect::<Vec<_>>()
        .join("<br>")
}

fn span(c: &Cell) -> usize {
    c.colspan.clamp(1, MAX_COLSPAN) as usize
}

/// Celdas de cada fila de una tabla, ya escapadas y todas con el mismo numero de columnas (como
/// mucho `MAX_COLS`: lo que sobra se junta en la ultima).
fn table_cells(rows: &[Vec<Cell>]) -> Vec<Vec<String>> {
    let width = rows.iter().map(|r| r.iter().map(span).sum::<usize>()).max().unwrap_or(0).clamp(1, MAX_COLS);
    rows.iter()
        .enumerate()
        // Las filas sin ningun texto (formularios por llenar) no aportan; la primera se deja como encabezado.
        .filter(|(i, r)| *i == 0 || r.iter().any(|c| !c.merged && !c.text.trim().is_empty()))
        .map(|(_, r)| {
            let mut cells: Vec<String> = Vec::with_capacity(width);
            for c in r {
                let text = cell_text(c);
                if cells.len() >= width {
                    if let Some(last) = cells.last_mut().filter(|_| !text.is_empty()) {
                        if !last.is_empty() {
                            last.push(' ');
                        }
                        last.push_str(&text);
                    }
                    continue;
                }
                cells.push(text);
                // Una celda que ocupa varias columnas deja las siguientes vacias.
                let pad = (span(c) - 1).min(width - cells.len());
                cells.resize(cells.len() + pad, String::new());
            }
            cells.resize(width, String::new());
            cells
        })
        .collect::<Vec<_>>()
        .pipe_drop_empty_columns()
}

trait DropEmptyColumns {
    fn pipe_drop_empty_columns(self) -> Self;
}

impl DropEmptyColumns for Vec<Vec<String>> {
    /// Quita las columnas sin texto en ninguna fila (rejillas de formularios, celdas de relleno).
    fn pipe_drop_empty_columns(self) -> Self {
        let width = self.first().map_or(0, Vec::len);
        let keep: Vec<bool> = (0..width).map(|j| self.iter().any(|r| r.get(j).map_or(false, |c| !c.is_empty()))).collect();
        if keep.iter().all(|k| *k) || !keep.iter().any(|k| *k) {
            return self;
        }
        self.into_iter().map(|r| r.into_iter().zip(&keep).filter(|(_, k)| **k).map(|(c, _)| c).collect()).collect()
    }
}

fn row_line(cells: &[String]) -> String {
    format!("| {} |", cells.join(" | "))
}

/// Una fila demasiado larga para un fragmento se reparte en filas de continuacion con las mismas
/// columnas: cada celda larga se corta (en una oracion o un espacio) y sus partes van una por fila.
fn split_row(cells: &[String], room: usize) -> Vec<String> {
    let line = row_line(cells);
    if line.chars().count() <= room {
        return vec![line];
    }
    let filled = cells.iter().filter(|c| !c.is_empty()).count().max(1);
    // Lo que queda para texto despues de los separadores "| ", repartido entre las celdas con texto.
    let budget = (room.saturating_sub(4 * cells.len() + 2) / filled).max(40);
    let pieces: Vec<Vec<String>> = cells.iter().map(|c| if c.is_empty() { Vec::new() } else { split_text(c, budget) }).collect();
    let n = pieces.iter().map(Vec::len).max().unwrap_or(1).max(1);
    (0..n).map(|i| row_line(&pieces.iter().map(|p| p.get(i).cloned().unwrap_or_default()).collect::<Vec<_>>())).collect()
}

fn separator(width: usize) -> String {
    format!("|{}", " --- |".repeat(width.max(1)))
}

fn table_md(rows: &[Vec<Cell>]) -> String {
    let cells = table_cells(rows);
    let first = match cells.first() {
        Some(f) => f,
        None => return String::new(),
    };
    let mut out = vec![row_line(first), separator(first.len())];
    out.extend(cells[1..].iter().map(|c| row_line(c)));
    out.join("\n")
}

/// Titulo de seccion de un bloque: (nivel, texto sin escapar). Ademas de los titulos, muchas
/// plantillas escriben los de seccion ("1. OBJETIVO") como una tabla de una sola celda con texto
/// corto; esas tablas cuentan como titulo, con el nivel que de su numeracion ("3.2" = 2), solo si
/// el texto parece titulo: numerado ("3.2 Responsables") o en mayusculas ("OBJETIVO"). Un recuadro
/// con una advertencia ("Use casco en todo momento") sigue siendo tabla.
fn section(b: &Block) -> Option<(u8, String)> {
    match b.kind {
        Kind::Title => Some((0, b.text.trim().to_string())),
        Kind::Heading => Some((b.level.max(1), b.text.trim().to_string())),
        Kind::Table => {
            let cells: Vec<&Cell> = b.rows.iter().flatten().filter(|c| !c.merged && !c.text.trim().is_empty()).collect();
            if b.rows.len() != 1 || cells.len() != 1 {
                return None;
            }
            let t = cells[0].text.trim();
            if t.contains(['\n', '\r']) || t.chars().count() > 80 || t.chars().filter(|c| c.is_alphabetic()).count() < 3 {
                return None;
            }
            let raw = t.split_whitespace().next().unwrap_or("");
            let token = raw.trim_end_matches('.');
            let numbered = raw.contains('.') && raw.len() < t.len() && !token.is_empty() && token.split('.').all(|p| !p.is_empty() && p.len() <= 3 && p.bytes().all(|c| c.is_ascii_digit()));
            let upper = t.chars().filter(|c| c.is_alphabetic()).all(char::is_uppercase);
            if !numbered && !upper {
                return None;
            }
            let depth = if numbered { token.split('.').count() } else { 1 };
            Some((depth.min(6) as u8, t.to_string()))
        }
        _ => None,
    }
}

struct Ctx {
    /// Suma a los niveles de titulo: 1 si el documento tiene un titulo principal (va con `#`).
    offset: u8,
    labels: HashSet<String>,
}

/// Una etiqueta de nota que se puede escribir como `[^etiqueta]` sin romper la sintaxis.
fn safe_label(l: &str) -> bool {
    !l.is_empty() && l.chars().count() <= MAX_LABEL && l.chars().all(|c| c.is_alphanumeric() || "*†‡§¶#-_.".contains(c))
}

/// Markdown de un bloque (sin la marca de pagina). Vacio si el bloque no aporta texto.
fn block_md(b: &Block, cx: &Ctx) -> String {
    let text = || inline(b, &cx.labels);
    // Un titulo es una sola linea: un salto dentro de el lo cortaria.
    let one_line = |s: String| s.split_whitespace().collect::<Vec<_>>().join(" ");
    match b.kind {
        Kind::Title => format!("# {}", one_line(text())),
        Kind::Heading => format!("{} {}", "#".repeat((b.level.max(1) + cx.offset).min(6) as usize), one_line(text())),
        Kind::Paragraph => escape_lines(&text()),
        Kind::ListItem => {
            let indent = "  ".repeat(b.level as usize);
            let marker = b.marker.as_deref().unwrap_or("").trim();
            let numbered = !marker.is_empty() && marker.trim_end_matches(['.', ')']).chars().all(|c| c.is_ascii_digit()) && marker.len() <= 10 && (marker.ends_with('.') || marker.ends_with(')'));
            let body = escape_lines(text().trim());
            if numbered {
                format!("{}{} {}", indent, marker, body)
            } else if b.ordered && !marker.is_empty() {
                // "a)", "iv.", "3.2.1": Markdown no los numera; se dejan escritos. El texto del
                // numero lo define el documento (`w:lvlText`): en una linea y escapado al inicio,
                // como el texto ("#" o ">" lo volverian titulo o cita dentro del item).
                format!("{}- {} {}", indent, escape_start(&one_line(escape_text(marker))), body)
            } else {
                format!("{}- {}", indent, body)
            }
        }
        Kind::Table => match section(b) {
            Some((level, t)) => format!("{} {}", "#".repeat((level.max(1) + cx.offset).min(6) as usize), escape_text(&t)),
            None => table_md(&b.rows),
        },
        Kind::Image => {
            let alt = one_line(escape_text(&b.text.replace(['[', ']'], "")));
            match &b.name {
                Some(n) => {
                    // Un nombre con ":" antes de la primera "/" se leeria como esquema ("javascript:").
                    // Un "&" tambien: CommonMark decodifica las entidades de un destino ("&#58;" = ":").
                    let n = encode_url(n.trim()).replace('&', "%26");
                    let local = if n.split('/').next().map_or(false, |p| p.contains(':')) { format!("./{}", n) } else { n };
                    format!("![{}]({})", alt, local)
                }
                None if !alt.is_empty() => format!("*\\[Imagen: {}\\]*", alt),
                None => String::new(),
            }
        }
        Kind::PageBreak => String::new(),
    }
}

fn context(d: &Document) -> Ctx {
    let labels: HashSet<String> = d
        .extras
        .iter()
        .filter(|(k, _)| *k == "footnotes" || *k == "endnotes")
        // Una nota sin texto no sale (`notes_md`): su cita queda como texto, no como `[^n]`.
        .flat_map(|(_, v)| v.iter().filter(|b| !b.text.trim().is_empty()).filter_map(|b| b.label.clone()))
        .filter(|l| safe_label(l))
        .collect();
    Ctx { offset: if d.content.iter().any(|b| b.kind == Kind::Title) { 1 } else { 0 }, labels }
}

/// Texto de una nota despues de "[^n]: ". Para un lector sin notas al pie (CommonMark puro) esa
/// linea es una definicion de referencia si el texto es un destino ("javascript:...", "<...>") o un
/// destino y un titulo ('x "t"'), y cada cita "[^n]" del cuerpo seria un enlace a ese destino.
/// Escapar no alcanza (CommonMark deshace escapes y entidades en un destino): la linea nunca tiene
/// esa forma. Despues de la primera palabra, que se leeria como destino, siempre va otra (un texto
/// de una sola palabra va despues de una raya) y esa no abre un titulo: `"`, `'` y `(` van escapados.
fn note_text(text: &str) -> String {
    not_destination(escape_text(&text.split_whitespace().collect::<Vec<_>>().join(" ")))
}

/// La regla de `note_text` sobre un texto ya escapado. Suma como mucho 3 caracteres y no cambia
/// un texto que ya la cumple.
fn not_destination(mut t: String) -> String {
    if !t.contains(' ') {
        t = format!("— {}", t);
    }
    let i = t.find(' ').map_or(0, |i| i + 1);
    if t[i..].starts_with(['"', '\'', '(']) {
        t.insert(i, '\\');
    }
    t
}

/// Una linea de notas en un fragmento: si es el inicio de una nota ("[^n]: ..."), cumple la regla
/// de `note_text` aunque el corte la haya dejado con una sola palabra.
fn note_line(l: &str) -> String {
    match l.strip_prefix("[^").and_then(|r| r.split_once("]: ")) {
        Some((label, rest)) if safe_label(label) && !rest.trim().is_empty() => format!("[^{}]: {}", label, not_destination(rest.trim().to_string())),
        _ => l.to_string(),
    }
}

/// Notas al pie y finales como notas de Markdown (`[^1]: texto`). Una nota sin texto no sale.
fn notes_md(d: &Document) -> String {
    let mut out: Vec<String> = Vec::new();
    for (k, blocks) in &d.extras {
        if *k != "footnotes" && *k != "endnotes" {
            continue;
        }
        for b in blocks {
            if let Some(l) = b.label.as_deref().filter(|l| safe_label(l) && !b.text.trim().is_empty()) {
                out.push(format!("[^{}]: {}", l, note_text(&b.text)));
            }
        }
    }
    out.join("\n")
}

/// Separador entre dos bloques: los items de una misma lista van seguidos.
fn joiner(prev: Option<Kind>, next: Kind) -> &'static str {
    match (prev, next) {
        (None, _) => "",
        (Some(Kind::ListItem), Kind::ListItem) => "\n",
        _ => "\n\n",
    }
}

/// El documento completo en Markdown.
pub fn to_markdown(d: &Document) -> String {
    let cx = context(d);
    let mut out = String::new();
    let mut prev: Option<Kind> = None;
    let mut page: Option<u32> = None;
    let pdf = d.kind == "pdf";
    for b in &d.content {
        let md = block_md(b, &cx);
        if md.trim().is_empty() {
            continue;
        }
        if pdf && b.page.is_some() && b.page != page {
            page = b.page;
            out.push_str(if out.is_empty() { "" } else { "\n\n" });
            out.push_str(&format!("<!-- pagina {} -->", page.unwrap_or(0)));
            prev = Some(Kind::PageBreak);
        }
        out.push_str(joiner(prev, b.kind));
        out.push_str(&md);
        prev = Some(b.kind);
    }
    let notes = notes_md(d);
    if !notes.is_empty() {
        out.push_str("\n\n");
        out.push_str(&notes);
    }
    out.push('\n');
    out
}

/// Fragmento del documento.
#[derive(Debug, Clone, Default)]
pub struct Chunk {
    /// Titulos de las secciones que lo contienen, del mas general al mas especifico.
    pub path: Vec<String>,
    pub text: String,
    /// Paginas que cubre (solo PDF).
    pub pages: Option<(u32, u32)>,
}

/// Titulos con que termina el fragmento en curso: pasan al siguiente si su contenido no cabe.
#[derive(Clone, Copy)]
struct Tail {
    /// Posicion en bytes de `cur.text` donde empiezan (antes del separador).
    at: usize,
    /// Paginas del fragmento antes de esos titulos.
    pages_before: Option<(u32, u32)>,
    /// Paginas de los titulos.
    pages: Option<(u32, u32)>,
}

struct Builder {
    max: usize,
    out: Vec<Chunk>,
    cur: Chunk,
    /// Caracteres de `cur.text`: se lleva la cuenta para no recontar el fragmento en cada bloque.
    chars: usize,
    prev: Option<Kind>,
    /// El fragmento en curso solo tiene titulos: acompanan al bloque que sigue aunque sea grande.
    only_headings: bool,
    tail: Option<Tail>,
}

fn widen(range: Option<(u32, u32)>, p: Option<u32>) -> Option<(u32, u32)> {
    match (range, p) {
        (Some((a, b)), Some(p)) => Some((a.min(p), b.max(p))),
        (None, Some(p)) => Some((p, p)),
        (r, None) => r,
    }
}

impl Builder {
    fn emit(&mut self) {
        if !self.cur.text.trim().is_empty() {
            let mut c = std::mem::take(&mut self.cur);
            c.text = c.text.trim().to_string();
            self.out.push(c);
        }
        self.cur = Chunk::default();
        self.chars = 0;
        self.prev = None;
        self.only_headings = false;
        self.tail = None;
    }

    /// Cierra el fragmento en curso porque el bloque que sigue no cabe. Si el fragmento termina en
    /// titulos, esos titulos pasan al fragmento nuevo junto a su contenido (con la ruta `path`).
    fn close_before(&mut self, path: &[String]) {
        match self.tail {
            Some(t) if t.at > 0 && !self.only_headings => {
                let carried = self.cur.text[t.at..].trim_start().to_string();
                self.cur.text.truncate(t.at);
                self.cur.pages = t.pages_before;
                self.emit();
                self.chars = carried.chars().count();
                self.cur = Chunk { path: path.to_vec(), text: carried, pages: t.pages };
                self.prev = Some(Kind::Heading);
                self.only_headings = true;
                self.tail = Some(Tail { at: 0, pages_before: None, pages: t.pages });
            }
            _ => self.emit(),
        }
    }

    fn push(&mut self, md: &str, kind: Kind, page: Option<u32>, path: &[String]) {
        if self.cur.text.is_empty() {
            self.cur.path = path.to_vec();
            self.only_headings = kind == Kind::Heading;
        } else if kind != Kind::Heading {
            self.only_headings = false;
        }
        if kind == Kind::Heading {
            let t = self.tail.get_or_insert(Tail { at: self.cur.text.len(), pages_before: self.cur.pages, pages: None });
            t.pages = widen(t.pages, page);
        } else {
            self.tail = None;
        }
        let j = joiner(self.prev, kind);
        self.cur.text.push_str(j);
        self.cur.text.push_str(md);
        self.chars += j.len() + md.chars().count();
        self.prev = Some(kind);
        self.cur.pages = widen(self.cur.pages, page);
    }
}

/// Parte un texto largo en trozos de hasta `max` caracteres, cortando de preferencia al final de
/// una oracion y si no en un espacio. Cada vuelta mira solo la ventana de `max` caracteres que
/// sigue, asi el costo es lineal en el largo del texto.
fn split_text(text: &str, max: usize) -> Vec<String> {
    let max = max.max(1);
    let mut parts = Vec::new();
    let mut rest = text.trim();
    while let Some((limit, _)) = rest.char_indices().nth(max) {
        let window = &rest[..limit];
        let mut cut = window.rfind(". ").map(|i| i + 1).filter(|i| *i > limit / 3).or_else(|| window.rfind(' ').filter(|i| *i > limit / 3)).unwrap_or(limit);
        cut = safe_cut(rest, cut);
        parts.push(rest[..cut].trim().to_string());
        rest = rest[cut..].trim_start();
    }
    if !rest.is_empty() {
        parts.push(rest.to_string());
    }
    parts
}

/// Corrige un corte que separaria un escape de su caracter ("\" | "<img") o partiria un "<br>":
/// el corte retrocede hasta antes de la barra o del "<". Si asi quedaria vacio, avanza.
fn safe_cut(text: &str, cut: usize) -> usize {
    let before = &text[..cut];
    let mut at = cut;
    if let Some(i) = before.rfind('<').filter(|i| cut - i < 4 && text[*i..].starts_with("<br>")) {
        at = i;
    }
    let slashes = text[..at].bytes().rev().take_while(|b| *b == b'\\').count();
    if slashes % 2 == 1 {
        at -= 1;
    }
    if text[..at].trim().is_empty() {
        // Nada antes del escape: el corte pasa despues del caracter escapado o del "<br>".
        let next = &text[at..];
        let skip = if next.starts_with("<br>") { 4 } else { next.chars().take(2).map(char::len_utf8).sum::<usize>().min(next.len()) };
        return (at + skip).max(cut).min(text.len());
    }
    at
}

/// Titulo para la ruta de un fragmento: en una linea y como mucho `MAX_PATH_CHARS` caracteres.
fn path_title(t: &str) -> String {
    let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.chars().count() <= MAX_PATH_CHARS {
        return t;
    }
    let mut s: String = t.chars().take(MAX_PATH_CHARS - 1).collect();
    s.truncate(s.trim_end().len());
    s.push('…');
    s
}

/// Corta el documento en fragmentos de hasta `max_chars` caracteres (minimo 200).
pub fn chunks(d: &Document, max_chars: usize) -> Vec<Chunk> {
    let max = max_chars.max(MIN_CHUNK);
    let cx = context(d);
    let mut b = Builder { max, out: Vec::new(), cur: Chunk::default(), chars: 0, prev: None, only_headings: false, tail: None };
    let mut path: Vec<(u8, String)> = Vec::new();
    let names = |p: &[(u8, String)]| p.iter().map(|(_, t)| t.clone()).collect::<Vec<_>>();
    for blk in &d.content {
        let md = block_md(blk, &cx);
        if md.trim().is_empty() {
            continue;
        }
        let len = md.chars().count();
        if let Some((level, title)) = section(blk) {
            // Un titulo abre fragmento nuevo, salvo que el actual sea muy corto y quepan juntos.
            if b.chars >= MIN_CHUNK || b.chars + len + 2 > b.max {
                b.emit();
            }
            while path.last().map_or(false, |(l, _)| *l >= level) {
                path.pop();
            }
            path.push((level, path_title(&title)));
            let here = names(&path);
            if len > b.max {
                // Un titulo mas largo que el fragmento se parte como un parrafo: la primera parte
                // conserva la marca de titulo y las demas se escapan al inicio de la linea. La
                // ultima queda abierta para el contenido de la seccion.
                for (i, piece) in split_text(&md, b.max - 1).iter().enumerate() {
                    if i > 0 {
                        b.emit();
                    }
                    let piece = if i == 0 { piece.clone() } else { escape_start(piece) };
                    b.push(&piece, Kind::Paragraph, blk.page, &here);
                }
                continue;
            }
            b.push(&md, Kind::Heading, blk.page, &here);
            continue;
        }
        let here = names(&path);
        if b.chars > 0 && b.chars + len + 2 > b.max && !b.only_headings {
            // No cabe: el fragmento se cierra y los titulos con que termina pasan al siguiente.
            b.close_before(&here);
        }
        let splittable = matches!(blk.kind, Kind::Paragraph | Kind::ListItem) || (blk.kind == Kind::Table && blk.rows.len() > 1);
        if len <= b.max && (b.chars == 0 || b.chars + len + 2 <= b.max) {
            b.push(&md, blk.kind, blk.page, &here);
            continue;
        }
        if len <= b.max && !splittable {
            // Una imagen o una tabla de una fila que no cabe junto a sus titulos: los titulos van solos.
            b.emit();
            b.push(&md, blk.kind, blk.page, &here);
            continue;
        }
        // Bloque que no cabe entero: va en partes. Los titulos que lo preceden (si el fragmento en
        // curso solo tiene titulos) quedan con la primera parte.
        if !b.only_headings || b.max.saturating_sub(b.chars + 2) < MIN_CHUNK / 2 {
            b.emit();
        }
        // Espacio que dejan los titulos que ya estan en el fragmento (solo para la primera parte).
        let room = |b: &Builder| b.max.saturating_sub(b.chars + 2).max(MIN_CHUNK / 2);
        if blk.kind == Kind::Table && blk.rows.len() > 1 {
            let cells = table_cells(&blk.rows);
            let width = cells[0].len();
            // Una primera fila muy larga no es un encabezado sino contenido: se usa un encabezado
            // vacio (valido en GFM) y esa fila se reparte como las demas.
            let real_head = row_line(&cells[0]).chars().count() <= b.max / 3;
            let head_cells = if real_head { cells[0].clone() } else { vec![String::new(); width] };
            let head = format!("{}\n{}", row_line(&head_cells), separator(width));
            let head_len = head.chars().count() + 1;
            // Un encabezado que ocupa mas de la mitad del fragmento (tablas muy anchas o fragmentos
            // chicos) no se repite: la tabla sale como texto, fila por fila, sin perder el encabezado.
            let plain = head_len * 2 > b.max;
            let body = if real_head && !plain { &cells[1..] } else { &cells[..] };
            // Filas que caben en un fragmento junto al encabezado (las mas largas, repartidas).
            let lines: Vec<String> = body.iter().flat_map(|r| split_row(r, b.max.saturating_sub(head_len))).collect();
            let mut part = head.clone();
            let mut part_len = head_len - 1;
            let mut limit = room(&b);
            for row in &lines {
                let row_len = row.chars().count();
                if plain || head_len + row_len > b.max {
                    // Ni sola con el encabezado cabe (muchas columnas con texto): esa fila sale como
                    // texto partido al tamano del fragmento, despues de las filas anteriores.
                    if part_len != head_len - 1 {
                        b.push(&part, Kind::Table, blk.page, &here);
                    }
                    b.emit();
                    for piece in split_text(row, b.max - 1) {
                        b.push(&escape_start(&piece), Kind::Paragraph, blk.page, &here);
                        b.emit();
                    }
                    part = head.clone();
                    part_len = head_len - 1;
                    limit = b.max;
                    continue;
                }
                let fits = part_len + row_len < limit;
                if !fits && part_len == head_len - 1 && limit < b.max {
                    // Ni la primera fila cabe junto a los titulos: los titulos van solos.
                    b.emit();
                    limit = b.max;
                }
                if part_len + row_len + 1 > limit && part_len != head_len - 1 {
                    b.push(&part, Kind::Table, blk.page, &here);
                    b.emit();
                    part = head.clone();
                    part_len = head_len - 1;
                    limit = b.max;
                }
                part.push('\n');
                part.push_str(row);
                part_len += row_len + 1;
            }
            if part_len != head_len - 1 {
                b.push(&part, Kind::Table, blk.page, &here);
            }
            b.emit();
        } else {
            // Cada parte se escapa al inicio como una linea nueva (un corte puede dejar "# ..." o
            // "1. ..." al comienzo), por eso se corta con un caracter de margen.
            let pieces = split_text(&md, room(&b) - 1);
            if let Some((head, rest)) = pieces.split_first() {
                b.push(head, Kind::Paragraph, blk.page, &here);
                // El resto se vuelve a partir con el tamano completo. La ultima parte queda abierta:
                // los bloques que siguen se pueden juntar con ella.
                for piece in split_text(&rest.join(" "), b.max - 1) {
                    b.emit();
                    b.push(&escape_start(&piece), Kind::Paragraph, blk.page, &here);
                }
            }
        }
    }
    // Las notas al pie van en un fragmento propio al final. Un corte puede dejar una nota con solo
    // su primera palabra ("[^1]: palabra"), que seria una definicion de referencia: esa linea se
    // arregla como en `note_text` (por eso el corte deja 3 caracteres mas de margen).
    let notes = notes_md(d);
    b.emit();
    if !notes.is_empty() {
        for piece in split_text(&notes, b.max - 4) {
            let piece = escape_start(&piece).lines().map(note_line).collect::<Vec<_>>().join("\n");
            b.push(&piece, Kind::Paragraph, None, &["Notas".to_string()]);
            b.emit();
        }
    }
    b.out
}

/// Fragmentos en JSON: `{ type, metadata, page_count, chunks: [{ index, path, pages, chars, text }] }`.
pub fn chunks_json(d: &Document, list: &[Chunk]) -> String {
    let mut meta = Obj::new();
    for (k, v) in &d.meta {
        match v {
            crate::doc::Meta::Text(s) => meta.str(k, s),
            crate::doc::Meta::Int(i) => meta.int(k, *i),
            crate::doc::Meta::Bool(x) => meta.bool(k, *x),
        };
    }
    let items: Vec<String> = list
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let mut o = Obj::new();
            o.int("index", i as i64);
            let path: Vec<String> = c.path.iter().map(|p| json::string(p)).collect();
            o.raw("path", &json::array(&path));
            if let Some((a, z)) = c.pages {
                o.raw("pages", &json::array(&[a.to_string(), z.to_string()]));
            }
            o.int("chars", c.text.chars().count() as i64);
            o.str("text", &c.text);
            o.end()
        })
        .collect();
    let mut root = Obj::new();
    root.str("type", d.kind).raw("metadata", &meta.end());
    match d.page_count {
        Some(n) => root.int("page_count", n as i64),
        None => root.raw("page_count", "null"),
    };
    root.raw("chunks", &json::array(&items));
    root.end()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(text: &str) -> Block {
        Block { kind: Kind::Paragraph, text: text.into(), ..Block::default() }
    }

    fn h(level: u8, text: &str) -> Block {
        Block { kind: Kind::Heading, level, text: text.into(), ..Block::default() }
    }

    fn celda(t: &str) -> Cell {
        Cell { text: t.into(), colspan: 1, rowspan: 1, merged: false, images: Vec::new() }
    }

    fn tabla(rows: Vec<Vec<Cell>>) -> Block {
        Block { kind: Kind::Table, rows, ..Block::default() }
    }

    fn doc(content: Vec<Block>) -> Document {
        Document { kind: "docx", content, ..Document::default() }
    }

    #[test]
    fn escapa_lo_que_markdown_leeria_como_marca() {
        assert_eq!(escape_start("# no es titulo"), "\\# no es titulo");
        assert_eq!(escape_start("1. no es lista"), "1\\. no es lista");
        assert_eq!(escape_start("12) tampoco"), "12\\) tampoco");
        assert_eq!(escape_start("3.2 no es lista"), "3.2 no es lista");
        assert_eq!(escape_start("- tampoco"), "\\- tampoco");
        assert_eq!(escape_start("2024 fue un buen anio"), "2024 fue un buen anio");
        assert_eq!(escape_start("-5 grados"), "-5 grados");
        // En un trozo de varias lineas (un fragmento cortado) cuenta solo la primera: el resto no
        // impide ver un "1." o un "---" solos en ella.
        assert_eq!(escape_start("1.\nSiguiente linea"), "1\\.\nSiguiente linea");
        assert_eq!(escape_start("---\notra"), "\\---\notra");
        assert_eq!(escape_start("-\notra"), "\\-\notra");
        assert_eq!(escape_start("\n  # titulo\n# otro"), "\\# titulo\n# otro");
    }

    #[test]
    fn el_numero_de_lista_de_word_no_abre_marcas() {
        // El texto del numero (`w:lvlText`) lo define el documento: no debe volver titulo, cita,
        // lista ni bloque de codigo el contenido del item, ni cortar la lista con un salto.
        let item = |marker: &str| Block { kind: Kind::ListItem, ordered: true, marker: Some(marker.into()), text: "Detener".into(), ..Block::default() };
        for (marker, esperado) in [
            ("#", "- \\# Detener"),
            (">", "- \\> Detener"),
            ("~~~", "- \\~~~ Detener"),
            ("*", "- \\* Detener"),
            ("-", "- \\- Detener"),
            ("a)\n# Titulo", "- a) # Titulo Detener"),
            ("iv.", "- iv. Detener"),
        ] {
            assert_eq!(to_markdown(&doc(vec![item(marker)])).trim_end(), esperado, "{:?}", marker);
        }
    }

    #[test]
    fn la_cita_de_una_nota_vacia_queda_como_texto() {
        // La nota vacia no sale, asi que su cita no puede ser "[^6]" (se veria literal y sin nota).
        let mut d = doc(vec![p("Cita [6] y [7].")]);
        let nota = |l: &str, t: &str| Block { label: Some(l.into()), text: t.into(), ..Block::default() };
        d.extras.push(("footnotes", vec![nota("6", " "), nota("7", "Nota siete.")]));
        let md = to_markdown(&d);
        assert!(md.contains("Cita \\[6\\] y [^7]."), "{}", md);
        assert!(!md.contains("[^6]"), "{}", md);
    }

    #[test]
    fn tablas_gfm_con_combinadas_y_sin_filas_vacias() {
        let mut ancha = celda("Riesgos");
        ancha.colspan = 3;
        let t = tabla(vec![vec![ancha], vec![celda("A|B"), celda("x\ny"), celda("z")], vec![celda(""), celda(" "), celda("")], vec![Cell { merged: true, colspan: 1, rowspan: 1, ..Cell::default() }, celda("c"), celda("d")]]);
        let md = to_markdown(&doc(vec![t]));
        assert_eq!(md, "| Riesgos |  |  |\n| --- | --- | --- |\n| A\\|B | x<br>y | z |\n|  | c | d |\n");
    }

    #[test]
    fn titulos_listas_y_notas() {
        let mut d = doc(vec![
            Block { kind: Kind::Title, text: "Manual".into(), ..Block::default() },
            h(1, "Pasos"),
            p("Antes de partir.[1]"),
            Block { kind: Kind::ListItem, ordered: true, marker: Some("1.".into()), text: "Detener".into(), ..Block::default() },
            Block { kind: Kind::ListItem, level: 1, marker: Some("•".into()), text: "Avisar".into(), ..Block::default() },
            Block { kind: Kind::ListItem, ordered: true, marker: Some("a)".into()), text: "Bloquear".into(), ..Block::default() },
            p("[2] no es una nota"),
        ]);
        d.extras.push(("footnotes", vec![Block { label: Some("1".into()), text: "Solo electricos.".into(), ..Block::default() }]));
        assert_eq!(to_markdown(&d), "# Manual\n\n## Pasos\n\nAntes de partir.[^1]\n\n1. Detener\n  - Avisar\n- a) Bloquear\n\n\\[2\\] no es una nota\n\n[^1]: Solo electricos.\n");
    }

    #[test]
    fn tabla_de_una_celda_es_titulo_de_seccion() {
        let d = doc(vec![tabla(vec![vec![celda("3.2 RESPONSABLES")]]), p("Texto")]);
        assert_eq!(to_markdown(&d), "## 3.2 RESPONSABLES\n\nTexto\n");
        let c = chunks(&d, 500);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].path, ["3.2 RESPONSABLES"]);
    }

    #[test]
    fn fragmentos_por_seccion_sin_pasar_el_maximo() {
        let largo = "Oracion de prueba con varias palabras. ".repeat(30); // ~1170 caracteres
        let mut content = vec![h(1, "Objetivo"), p(&largo), h(1, "Desarrollo"), h(2, "Antes"), p(&largo), p(&largo)];
        // Tabla de 40 filas: mas grande que un fragmento.
        let mut rows = vec![vec![celda("Paso"), celda("Detalle")]];
        for i in 0..40 {
            rows.push(vec![celda(&i.to_string()), celda("Revisar el equipo y registrar la medicion")]);
        }
        content.push(h(2, "Tabla"));
        content.push(tabla(rows));
        let c = chunks(&doc(content), 1000);
        assert!(c.iter().all(|x| x.text.chars().count() <= 1000), "{:?}", c.iter().map(|x| x.text.chars().count()).collect::<Vec<_>>());
        assert_eq!(c[0].path, ["Objetivo"]);
        assert!(c[0].text.starts_with("## Objetivo") || c[0].text.starts_with("# Objetivo"));
        assert!(c.iter().any(|x| x.path == ["Desarrollo", "Antes"]));
        // La tabla se parte por filas y cada parte repite el encabezado; el titulo va con la primera.
        let partes: Vec<&Chunk> = c.iter().filter(|x| x.path == ["Desarrollo", "Tabla"]).collect();
        assert!(partes.len() >= 2);
        assert!(partes[0].text.starts_with("## Tabla\n\n| Paso | Detalle |"));
        assert!(partes.iter().all(|x| x.text.contains("| Paso | Detalle |\n| --- | --- |")));
        // No se pierde ninguna fila.
        let filas: usize = partes.iter().map(|x| x.text.matches("Revisar el equipo").count()).sum();
        assert_eq!(filas, 40);
    }

    #[test]
    fn fragmentos_en_json() {
        let d = doc(vec![h(1, "Alcance"), p("Aplica a \"todos\".")]);
        let j = chunks_json(&d, &chunks(&d, 2000));
        assert!(j.contains("\"chunks\":[{\"index\":0,\"path\":[\"Alcance\"],\"chars\":"));
        assert!(j.contains("Aplica a \\\"todos\\\"."));
    }
}

#[cfg(test)]
mod filas_largas {
    use super::*;

    #[test]
    fn una_fila_mas_grande_que_el_fragmento_se_reparte() {
        let larga = "Verificar el bloqueo antes de intervenir el equipo. ".repeat(60); // ~3100 caracteres
        let fila = vec![Cell { text: "1".into(), colspan: 1, rowspan: 1, merged: false, images: Vec::new() }, Cell { text: larga.clone(), colspan: 1, rowspan: 1, merged: false, images: Vec::new() }];
        let cab = vec![Cell { text: "Paso".into(), colspan: 1, rowspan: 1, merged: false, images: Vec::new() }, Cell { text: "Riesgos y controles".into(), colspan: 1, rowspan: 1, merged: false, images: Vec::new() }];
        let d = Document { kind: "docx", content: vec![Block { kind: Kind::Heading, level: 1, text: "Ejecucion".into(), ..Block::default() }, Block { kind: Kind::Table, rows: vec![cab, fila], ..Block::default() }], ..Document::default() };
        let c = chunks(&d, 1000);
        assert!(c.iter().all(|x| x.text.chars().count() <= 1000), "{:?}", c.iter().map(|x| x.text.chars().count()).collect::<Vec<_>>());
        assert!(c.len() >= 4);
        assert!(c.iter().all(|x| x.text.contains("| Paso | Riesgos y controles |\n| --- | --- |")));
        // No se pierde texto: todas las repeticiones de la oracion siguen estando.
        let total: usize = c.iter().map(|x| x.text.matches("Verificar el bloqueo").count()).sum();
        assert_eq!(total, 60);
    }
}

#[cfg(test)]
mod revision {
    use super::*;
    use std::time::Instant;

    fn p(text: &str) -> Block {
        Block { kind: Kind::Paragraph, text: text.into(), ..Block::default() }
    }

    fn h(level: u8, text: &str) -> Block {
        Block { kind: Kind::Heading, level, text: text.into(), ..Block::default() }
    }

    fn una_celda(t: &str) -> Block {
        Block { kind: Kind::Table, rows: vec![vec![Cell { text: t.into(), colspan: 1, rowspan: 1, merged: false, images: Vec::new() }]], ..Block::default() }
    }

    fn doc(content: Vec<Block>) -> Document {
        Document { kind: "docx", content, ..Document::default() }
    }

    #[test]
    fn partir_un_texto_enorme_es_lineal() {
        // 8 millones de caracteres en trozos de 200: cuadratico serian ~10^11 pasos.
        let texto = "palabra ".repeat(1_000_000);
        let inicio = Instant::now();
        let partes = split_text(&texto, 200);
        assert!(inicio.elapsed().as_secs() < 20, "{:?}", inicio.elapsed());
        assert!(partes.iter().all(|p| p.chars().count() <= 200));
        assert_eq!(partes.iter().map(|p| p.matches("palabra").count()).sum::<usize>(), 1_000_000);
        // Tambien sin espacios y con caracteres de varios bytes.
        let partes = split_text(&"ñ🦺".repeat(300_000), 150);
        assert!(partes.iter().all(|p| p.chars().count() <= 150));
        assert_eq!(partes.concat().chars().count(), 600_000);
    }

    #[test]
    fn muchos_bloques_chicos_con_fragmentos_grandes_es_lineal() {
        let content: Vec<Block> = (0..200_000).map(|i| p(&format!("Linea {}", i))).collect();
        let inicio = Instant::now();
        let c = chunks(&doc(content), 16_000_000);
        assert!(inicio.elapsed().as_secs() < 20, "{:?}", inicio.elapsed());
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn la_ruta_acorta_los_titulos_largos() {
        let largo = "Titulo muy largo ".repeat(500);
        let c = chunks(&doc(vec![h(1, &largo), p("Texto de la seccion.")]), 20_000);
        assert!(c[0].path[0].chars().count() <= MAX_PATH_CHARS, "{}", c[0].path[0].chars().count());
        assert!(c[0].path[0].ends_with('…'));
        // El titulo completo sigue en el texto del fragmento.
        assert!(c[0].text.contains(largo.trim()));
    }

    #[test]
    fn un_titulo_no_queda_al_final_de_un_fragmento() {
        let corto = "Texto breve de la primera seccion.";
        let largo = "Oracion de relleno para la seccion siguiente. ".repeat(21); // 966 caracteres
        let d = doc(vec![h(1, "Primera"), p(corto), h(1, "Segunda"), h(2, "Detalle"), p(&largo)]);
        let c = chunks(&d, 1000);
        assert_eq!(c.len(), 2, "{:#?}", c);
        assert!(!c[0].text.contains("Segunda"), "{:#?}", c);
        assert!(c[1].text.starts_with("# Segunda\n\n## Detalle\n\nOracion"), "{:#?}", c);
        assert_eq!(c[1].path, ["Segunda", "Detalle"]);
        for x in &c {
            let ultimo = x.text.rsplit("\n\n").next().unwrap();
            assert!(!ultimo.starts_with('#'), "{:?}", x.text);
        }
    }

    #[test]
    fn un_recuadro_de_una_celda_no_es_titulo() {
        for t in ["Use casco en todo momento", "Nota: revisar antes de partir", "Ver anexo"] {
            let md = to_markdown(&doc(vec![una_celda(t), p("Texto")]));
            assert_eq!(md, format!("| {} |\n| --- |\n\nTexto\n", t));
        }
        for (t, esperado) in [("3.2 RESPONSABLES", "## 3.2 RESPONSABLES"), ("1. Objetivo", "# 1. Objetivo"), ("OBJETIVO", "# OBJETIVO"), ("DEFINICIONES Y TÉRMINOS", "# DEFINICIONES Y TÉRMINOS")] {
            let md = to_markdown(&doc(vec![una_celda(t), p("Texto")]));
            assert_eq!(md, format!("{}\n\nTexto\n", esperado));
        }
    }

    /// La linea es una definicion de referencia para CommonMark: "[etiqueta]:", un destino
    /// ("<...>" o sin espacios y con los parentesis equilibrados) y, si hay, un titulo separado por
    /// espacios ("...", '...' o (...)), sin nada mas hasta el final de la linea.
    fn es_definicion(linea: &str) -> bool {
        let resto = match linea.strip_prefix('[').and_then(|r| r.split_once("]:")) {
            Some((_, r)) => r,
            None => return false,
        };
        let s: Vec<char> = resto.trim_start_matches([' ', '\t']).chars().collect();
        let escape = |s: &[char], i: usize| s[i] == '\\' && s.get(i + 1).map_or(false, char::is_ascii_punctuation);
        let mut i = 0;
        if s.first() == Some(&'<') {
            i = 1;
            loop {
                match s.get(i) {
                    None | Some('<') => return false,
                    Some('>') => {
                        i += 1;
                        break;
                    }
                    _ => i += if escape(&s, i) { 2 } else { 1 },
                }
            }
        } else {
            let mut abiertos = 0;
            while i < s.len() && s[i] != ' ' && !s[i].is_ascii_control() {
                if escape(&s, i) {
                    i += 2;
                    continue;
                }
                if s[i] == '(' {
                    abiertos += 1;
                } else if s[i] == ')' {
                    if abiertos == 0 {
                        break;
                    }
                    abiertos -= 1;
                }
                i += 1;
            }
            if i == 0 || abiertos != 0 {
                return false;
            }
        }
        let espacios = s[i..].iter().take_while(|c| **c == ' ' || **c == '\t').count();
        let t = &s[i + espacios..];
        if t.is_empty() {
            return true;
        }
        let cierre = match t[0] {
            '"' => '"',
            '\'' => '\'',
            '(' => ')',
            _ => return false,
        };
        let mut j = 1;
        while j < t.len() && espacios > 0 {
            if escape(t, j) {
                j += 2;
                continue;
            }
            if t[j] == cierre {
                return t[j + 1..].iter().all(|c| *c == ' ' || *c == '\t');
            }
            if t[0] == '(' && t[j] == '(' {
                return false;
            }
            j += 1;
        }
        false
    }

    /// La primera definicion de referencia de un texto, tambien con el destino o el titulo en la
    /// linea siguiente (CommonMark admite un salto de linea entre la etiqueta, el destino y el titulo).
    fn hay_definicion(texto: &str) -> Option<String> {
        let lineas: Vec<&str> = texto.lines().collect();
        (0..lineas.len()).find_map(|i| {
            let dos = lineas.get(i + 1).map(|s| format!("{} {}", lineas[i], s.trim_start()));
            let es = es_definicion(lineas[i]) || dos.as_deref().map_or(false, es_definicion);
            es.then(|| dos.unwrap_or_else(|| lineas[i].to_string()))
        })
    }

    #[test]
    fn una_nota_no_es_una_definicion_de_referencia() {
        // Para un lector sin notas al pie, "[^1]: destino" y "[^1]: destino \"titulo\"" definen la
        // referencia "^1" y cada cita "[^1]" del cuerpo seria un enlace a ese destino.
        for l in ["[^1]: javascript:alert(1)", "[^1]: \\<javascript:alert(1)>", "[^1]: <javascript:alert(1)>", "[^1]: x \"t\"", "[^1]: x 't'", "[^1]: x (t)"] {
            assert!(es_definicion(l), "{}", l);
        }
        for l in ["[^1]: x y", "[^1]: (x", "[^1]: x \\\"t\"", "[^1]: x \"t\" y", "[^1]:"] {
            assert!(!es_definicion(l), "{}", l);
        }
        // Con el destino o el titulo en la linea siguiente tambien es una definicion.
        for t in ["[^1]:\njavascript:alert(1)", "[^1]:\n  x \"t\"", "Texto\n[^1]: x\n\"t\""] {
            assert!(hay_definicion(t).is_some(), "{}", t);
        }
        assert_eq!(hay_definicion("[^1]: — x\n\"t\""), None);
        let notas = ["javascript:alert(1)", "<javascript:alert(1)>", "javascript:alert(1) \"titulo\"", "javascript:alert(1) 'titulo'", "javascript:alert(1) (titulo)", " ", "\"javascript:alert(1)\"", "Solo equipos electricos."];
        let citas: String = (1..=notas.len()).map(|i| format!("Cita [{}]. ", i)).collect();
        let mut d = doc(vec![p(&citas)]);
        d.extras.push(("footnotes", notas.iter().enumerate().map(|(i, t)| Block { label: Some((i + 1).to_string()), text: t.to_string(), ..Block::default() }).collect()));
        let md = to_markdown(&d);
        let lineas: Vec<&str> = md.lines().filter(|l| l.starts_with("[^")).collect();
        assert_eq!(
            lineas,
            [
                "[^1]: — javascript:alert(1)",
                "[^2]: — \\<javascript:alert(1)>",
                "[^3]: javascript:alert(1) \\\"titulo\"",
                "[^4]: javascript:alert(1) \\'titulo'",
                "[^5]: javascript:alert(1) \\(titulo)",
                "[^7]: — \\\"javascript:alert(1)\"",
                "[^8]: Solo equipos electricos.",
            ],
            "{}",
            md
        );
        assert_eq!(hay_definicion(&md), None, "{}", md);
        // En los fragmentos, un corte no deja una nota con solo su primera palabra: aqui la segunda
        // palabra es enorme frente al fragmento y el corte cae justo despues de la primera.
        let larga = format!("javascript:alert(1)//{} {}", "a".repeat(80), "x".repeat(500));
        d.extras.push(("endnotes", vec![Block { label: Some("i".into()), text: larga, ..Block::default() }]));
        for max in [200, 300] {
            for c in chunks(&d, max) {
                assert!(c.text.chars().count() <= max, "{}", c.text);
                assert_eq!(hay_definicion(&c.text), None, "{}", c.text);
            }
        }
    }

    #[test]
    fn una_imagen_con_entidades_no_sale_remota() {
        // CommonMark decodifica las entidades de un destino: "&#58;" seria ":" y la imagen, remota.
        let img = |n: &str| Block { kind: Kind::Image, name: Some(n.into()), text: "foto".into(), ..Block::default() };
        let md = to_markdown(&doc(vec![img("https&#58;&#47;&#47;host&#47;p.png"), img("javascript&#58;alert(1)")]));
        assert_eq!(md, "![foto](https%26#58;%26#47;%26#47;host%26#47;p.png)\n\n![foto](javascript%26#58;alert%281%29)\n");
    }
}
