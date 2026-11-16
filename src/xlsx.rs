//! Escritor XLSX propio (SpreadsheetML minimo): genera el libro de Excel del proyecto.
//!
//! Hojas: "Gantt" (tabla resumida + barras en escala de tiempo), "Tareas" (transcripcion
//! completa de la tabla del Gantt del .mpp), "Recursos", "Asignaciones" y "Proyecto".
//! Las barras del Gantt son formato condicional sobre las fechas, asi que se mueven solas
//! si alguien edita Start/Finish en Excel.

use crate::model::*;
use crate::mpp::fields::task as tf;
use crate::schema::{self, Cell, Col, Index, Key};
use crate::zip::Zip;

const NS_MAIN: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
const NS_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const XML_HEAD: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n";

// ---------------------------------------------------------------------------------------------
// Estilos
// ---------------------------------------------------------------------------------------------

/// (negrita, cursiva, tachado, color ARGB, tamano)
const FONTS: [(bool, bool, bool, &str, f32); 8] = [
    (false, false, false, "FF000000", 10.0), // 0 normal
    (true, false, false, "FF000000", 10.0),  // 1 negrita (tareas resumen)
    (true, false, false, "FFFFFFFF", 10.0),  // 2 encabezado
    (false, false, true, "FF8C8C8C", 10.0),  // 3 tarea inactiva
    (true, false, true, "FF8C8C8C", 10.0),   // 4 tarea resumen inactiva
    (true, false, false, "FF1F3864", 14.0),  // 5 titulo
    (false, true, false, "FF595959", 9.0),   // 6 nota
    (false, false, false, "FFFFFFFF", 8.0),  // 7 etiqueta de hora en la escala de tiempo
];
const F_NORMAL: u8 = 0;
const F_BOLD: u8 = 1;
const F_HEADER: u8 = 2;
const F_INACTIVE: u8 = 3;
const F_INACTIVE_BOLD: u8 = 4;
const F_TITLE: u8 = 5;
const F_NOTE: u8 = 6;
const F_TIME: u8 = 7;

/// Rellenos solidos (los dos primeros los exige el formato).
const FILLS: [&str; 6] = ["", "", "FF1F3864", "FF44546A", "FF2E75B6", "FFDDEBF7"];
const FILL_HEADER: u8 = 2;
const FILL_HEADER_EXTRA: u8 = 3;
const FILL_HEADER_CUSTOM: u8 = 4;
const FILL_LABEL: u8 = 5;

/// Colores de las barras del Gantt, en el orden de las reglas de formato condicional.
const BARS: [(&str, &str, &str); 5] = [
    ("Milestone", "FF000000", "Hito"),
    ("Inactive", "FFBFBFBF", "Tarea inactiva"),
    ("Summary", "FF404040", "Tarea resumen"),
    ("Critical", "FFC00000", "Tarea crítica"),
    ("Normal", "FF2E75B6", "Tarea normal"),
];

#[derive(Clone, Copy, PartialEq, Default)]
struct Xf {
    font: u8,
    fill: u8,
    fmt: u16,
    /// 0 general, 1 izquierda, 2 centro, 3 derecha
    halign: u8,
    indent: u8,
    wrap: bool,
    rotate: bool,
    /// Alineacion vertical superior (deja libre la esquina del boton de filtro).
    top: bool,
}

struct Styles {
    xfs: Vec<Xf>,
    fmts: Vec<String>,
}

impl Styles {
    fn new() -> Styles {
        Styles { xfs: vec![Xf::default()], fmts: Vec::new() }
    }

    /// Registra un formato numerico propio y devuelve su id.
    fn fmt(&mut self, code: &str) -> u16 {
        let pos = match self.fmts.iter().position(|f| f == code) {
            Some(p) => p,
            None => {
                self.fmts.push(code.to_string());
                self.fmts.len() - 1
            }
        };
        164 + pos as u16
    }

    fn xf(&mut self, x: Xf) -> usize {
        match self.xfs.iter().position(|e| *e == x) {
            Some(p) => p,
            None => {
                self.xfs.push(x);
                self.xfs.len() - 1
            }
        }
    }

    fn xml(&self) -> String {
        let mut s = String::from(XML_HEAD);
        s.push_str(&format!("<styleSheet xmlns=\"{}\">", NS_MAIN));
        if !self.fmts.is_empty() {
            s.push_str(&format!("<numFmts count=\"{}\">", self.fmts.len()));
            for (i, f) in self.fmts.iter().enumerate() {
                s.push_str(&format!("<numFmt numFmtId=\"{}\" formatCode=\"{}\"/>", 164 + i, esc(f)));
            }
            s.push_str("</numFmts>");
        }
        s.push_str(&format!("<fonts count=\"{}\">", FONTS.len()));
        for (bold, italic, strike, color, size) in FONTS {
            s.push_str("<font>");
            if bold {
                s.push_str("<b/>");
            }
            if italic {
                s.push_str("<i/>");
            }
            if strike {
                s.push_str("<strike/>");
            }
            s.push_str(&format!("<sz val=\"{}\"/><color rgb=\"{}\"/><name val=\"Arial\"/><family val=\"2\"/></font>", size, color));
        }
        s.push_str("</fonts>");
        s.push_str(&format!("<fills count=\"{}\"><fill><patternFill patternType=\"none\"/></fill><fill><patternFill patternType=\"gray125\"/></fill>", FILLS.len()));
        for color in &FILLS[2..] {
            s.push_str(&format!("<fill><patternFill patternType=\"solid\"><fgColor rgb=\"{}\"/><bgColor indexed=\"64\"/></patternFill></fill>", color));
        }
        s.push_str("</fills>");
        s.push_str("<borders count=\"1\"><border><left/><right/><top/><bottom/><diagonal/></border></borders>");
        s.push_str("<cellStyleXfs count=\"1\"><xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\"/></cellStyleXfs>");
        s.push_str(&format!("<cellXfs count=\"{}\">", self.xfs.len()));
        for x in &self.xfs {
            s.push_str(&format!(
                "<xf numFmtId=\"{}\" fontId=\"{}\" fillId=\"{}\" borderId=\"0\" xfId=\"0\" applyNumberFormat=\"1\" applyFont=\"1\" applyFill=\"1\" applyAlignment=\"1\">",
                x.fmt, x.font, x.fill
            ));
            s.push_str("<alignment");
            match x.halign {
                1 => s.push_str(" horizontal=\"left\""),
                2 => s.push_str(" horizontal=\"center\""),
                3 => s.push_str(" horizontal=\"right\""),
                _ => {}
            }
            s.push_str(if x.rotate {
                " vertical=\"bottom\" textRotation=\"90\""
            } else if x.top {
                " vertical=\"top\""
            } else {
                " vertical=\"center\""
            });
            if x.indent > 0 {
                s.push_str(&format!(" indent=\"{}\"", x.indent));
            }
            if x.wrap {
                s.push_str(" wrapText=\"1\"");
            }
            s.push_str("/></xf>");
        }
        s.push_str("</cellXfs>");
        s.push_str("<cellStyles count=\"1\"><cellStyle name=\"Normal\" xfId=\"0\" builtinId=\"0\"/></cellStyles>");
        s.push_str(&format!("<dxfs count=\"{}\">", BARS.len()));
        for (_, color, _) in BARS {
            s.push_str(&format!(
                "<dxf><fill><patternFill patternType=\"solid\"><fgColor rgb=\"{c}\"/><bgColor rgb=\"{c}\"/></patternFill></fill><border><left/><right/><top style=\"thin\"><color rgb=\"FFFFFFFF\"/></top><bottom style=\"thin\"><color rgb=\"FFFFFFFF\"/></bottom></border></dxf>",
                c = color
            ));
        }
        s.push_str("</dxfs></styleSheet>");
        s
    }
}

// ---------------------------------------------------------------------------------------------
// Utilidades de XML y celdas
// ---------------------------------------------------------------------------------------------

fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\t' | '\n' => o.push(c),
            '\r' => {}
            c if (c as u32) < 0x20 || c == '\u{FFFE}' || c == '\u{FFFF}' => {}
            c => o.push(c),
        }
    }
    o
}

/// Letra(s) de columna a partir del indice (0 = A).
fn col_name(mut n: usize) -> String {
    let mut s = Vec::new();
    loop {
        s.push(b'A' + (n % 26) as u8);
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    s.reverse();
    String::from_utf8(s).unwrap_or_default()
}

fn num_str(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

/// Acumula las filas de una hoja.
struct Rows {
    xml: String,
    row: usize,
    open: bool,
    max_col: usize,
}

impl Rows {
    fn new() -> Rows {
        Rows { xml: String::new(), row: 0, open: false, max_col: 0 }
    }

    fn start(&mut self, attrs: &str) {
        self.close();
        self.row += 1;
        self.xml.push_str(&format!("<row r=\"{}\"{}>", self.row, attrs));
        self.open = true;
    }

    fn close(&mut self) {
        if self.open {
            self.xml.push_str("</row>");
            self.open = false;
        }
    }

    fn skip(&mut self, n: usize) {
        self.close();
        self.row += n;
    }

    fn text(&mut self, col: usize, style: usize, s: &str) {
        self.max_col = self.max_col.max(col);
        let space = if s.starts_with(' ') || s.ends_with(' ') || s.contains('\n') { " xml:space=\"preserve\"" } else { "" };
        self.xml.push_str(&format!("<c r=\"{}{}\" s=\"{}\" t=\"inlineStr\"><is><t{}>{}</t></is></c>", col_name(col), self.row, style, space, esc(s)));
    }

    fn num(&mut self, col: usize, style: usize, v: f64) {
        self.max_col = self.max_col.max(col);
        self.xml.push_str(&format!("<c r=\"{}{}\" s=\"{}\"><v>{}</v></c>", col_name(col), self.row, style, num_str(v)));
    }

    fn blank(&mut self, col: usize, style: usize) {
        self.max_col = self.max_col.max(col);
        self.xml.push_str(&format!("<c r=\"{}{}\" s=\"{}\"/>", col_name(col), self.row, style));
    }

    fn finish(mut self) -> (String, usize, usize) {
        self.close();
        (self.xml, self.row, self.max_col)
    }
}

/// Etiqueta de unidades al estilo de Project en ingles ("24 hrs").
fn unit_label(u: Unit) -> &'static str {
    match u {
        Unit::Minutes => " mins",
        Unit::Hours => " hrs",
        Unit::Days => " days",
        Unit::Weeks => " wks",
        Unit::Months => " mons",
        Unit::Percent => "%",
        Unit::ElapsedMinutes => " emins",
        Unit::ElapsedHours => " ehrs",
        Unit::ElapsedDays => " edays",
        Unit::ElapsedWeeks => " ewks",
        Unit::ElapsedMonths => " emons",
        Unit::ElapsedPercent => "e%",
    }
}

const FMT_DATETIME: &str = "dd/mm/yy hh:mm";

/// Escribe una celda de datos eligiendo formato segun el tipo del valor.
fn data_cell(rows: &mut Rows, st: &mut Styles, col: usize, cell: &Cell, font: u8, indent: u8, wrap: bool) {
    let base = Xf { font, ..Xf::default() };
    match cell {
        Cell::Empty => {}
        Cell::Text(t) => {
            let s = st.xf(Xf { indent, wrap, halign: if indent > 0 { 1 } else { 0 }, ..base });
            rows.text(col, s, t);
        }
        Cell::Int(v) => {
            let s = st.xf(base);
            rows.num(col, s, *v as f64);
        }
        Cell::Num(v) => {
            let s = st.xf(base);
            rows.num(col, s, *v);
        }
        Cell::Date(d) => {
            let fmt = st.fmt(FMT_DATETIME);
            let s = st.xf(Xf { fmt, halign: 2, ..base });
            rows.num(col, s, excel_serial(*d));
        }
        Cell::Dur(d) => {
            let fmt = st.fmt(&format!("General\"{}\"", unit_label(d.unit)));
            let s = st.xf(Xf { fmt, halign: 3, ..base });
            rows.num(col, s, (d.value * 100.0).round() / 100.0);
        }
        Cell::Bool(b) => {
            let s = st.xf(Xf { halign: 2, ..base });
            rows.text(col, s, if *b { "Yes" } else { "No" });
        }
        Cell::Pct(v) => {
            let s = st.xf(Xf { fmt: 9, halign: 3, ..base });
            rows.num(col, s, v / 100.0);
        }
    }
}

fn task_font(t: &Task) -> u8 {
    match (t.active, t.summary) {
        (true, true) => F_BOLD,
        (true, false) => F_NORMAL,
        (false, true) => F_INACTIVE_BOLD,
        (false, false) => F_INACTIVE,
    }
}

struct SheetOpts {
    /// Columnas y filas inmovilizadas.
    freeze: (usize, usize),
    widths: Vec<f64>,
    /// Rango del autofiltro, p. ej. "A1:K200".
    filter: Option<String>,
    merges: Vec<String>,
    /// XML de formato condicional ya armado.
    conditional: String,
    max_outline: usize,
    landscape: bool,
}

fn sheet_xml(rows: Rows, o: &SheetOpts) -> String {
    let (data, last_row, max_col) = rows.finish();
    let mut s = String::from(XML_HEAD);
    s.push_str(&format!("<worksheet xmlns=\"{}\">", NS_MAIN));
    s.push_str("<sheetPr><outlinePr summaryBelow=\"0\" summaryRight=\"0\"/><pageSetUpPr fitToPage=\"1\"/></sheetPr>");
    s.push_str(&format!("<dimension ref=\"A1:{}{}\"/>", col_name(max_col), last_row.max(1)));
    s.push_str("<sheetViews><sheetView workbookViewId=\"0\">");
    let (fc, fr) = o.freeze;
    if fc > 0 || fr > 0 {
        let cell = format!("{}{}", col_name(fc), fr + 1);
        let pane = match (fc > 0, fr > 0) {
            (true, true) => "bottomRight",
            (true, false) => "topRight",
            _ => "bottomLeft",
        };
        s.push_str("<pane");
        if fc > 0 {
            s.push_str(&format!(" xSplit=\"{}\"", fc));
        }
        if fr > 0 {
            s.push_str(&format!(" ySplit=\"{}\"", fr));
        }
        s.push_str(&format!(" topLeftCell=\"{c}\" activePane=\"{p}\" state=\"frozen\"/><selection pane=\"{p}\" activeCell=\"{c}\" sqref=\"{c}\"/>", c = cell, p = pane));
    }
    s.push_str("</sheetView></sheetViews>");
    s.push_str(&format!("<sheetFormatPr defaultRowHeight=\"13.5\" outlineLevelRow=\"{}\"/>", o.max_outline.min(7)));
    if !o.widths.is_empty() {
        s.push_str("<cols>");
        for (i, w) in o.widths.iter().enumerate() {
            s.push_str(&format!("<col min=\"{n}\" max=\"{n}\" width=\"{w}\" customWidth=\"1\"/>", n = i + 1, w = w));
        }
        s.push_str("</cols>");
    }
    s.push_str("<sheetData>");
    s.push_str(&data);
    s.push_str("</sheetData>");
    if let Some(f) = &o.filter {
        s.push_str(&format!("<autoFilter ref=\"{}\"/>", f));
    }
    if !o.merges.is_empty() {
        s.push_str(&format!("<mergeCells count=\"{}\">", o.merges.len()));
        for m in &o.merges {
            s.push_str(&format!("<mergeCell ref=\"{}\"/>", m));
        }
        s.push_str("</mergeCells>");
    }
    s.push_str(&o.conditional);
    s.push_str("<pageMargins left=\"0.4\" right=\"0.4\" top=\"0.5\" bottom=\"0.5\" header=\"0.3\" footer=\"0.3\"/>");
    s.push_str(&format!("<pageSetup orientation=\"{}\" fitToWidth=\"1\" fitToHeight=\"0\"/>", if o.landscape { "landscape" } else { "portrait" }));
    // Codigos como "0010" o predecesoras como "8" son texto a proposito: sin triangulos de aviso.
    s.push_str(&format!("<ignoredErrors><ignoredError sqref=\"A1:{}{}\" numberStoredAsText=\"1\"/></ignoredErrors>", col_name(max_col), last_row.max(1)));
    s.push_str("</worksheet>");
    s
}

// ---------------------------------------------------------------------------------------------
// Hoja "Gantt"
// ---------------------------------------------------------------------------------------------

/// Escala de tiempo: inicio, tamano del tramo y numero de tramos (todo en segundos).
struct Timeline {
    start: i64,
    slot: i64,
    count: usize,
}

const MAX_SLOTS: i64 = 240;

fn timeline(p: &Project) -> Option<Timeline> {
    // Solo tareas activas: las inactivas suelen conservar fechas viejas que estirarian la escala.
    let span = |only_active: bool| -> Vec<(i64, i64)> { p.tasks.iter().filter(|t| t.active || !only_active).filter_map(|t| Some((t.start?, t.finish?))).collect() };
    let mut dated = span(true);
    if dated.is_empty() {
        dated = span(false);
    }
    let min = dated.iter().map(|d| d.0).min()?;
    let max = dated.iter().map(|d| d.1).max()?.max(min);
    // El tramo mas grande (hasta un dia) en el que caben exactas todas las fechas...
    const SLOTS: [i64; 9] = [900, 1800, 3600, 7200, 14400, 21600, 43200, 86400, 604800];
    let mut pick = 0usize;
    for (i, s) in SLOTS.iter().enumerate().take(8) {
        if dated.iter().all(|d| d.0 % s == 0 && d.1 % s == 0) {
            pick = i;
        }
    }
    // ...y si aun asi salen demasiadas columnas, se agranda.
    while pick + 1 < SLOTS.len() && (max - min) / SLOTS[pick] + 2 > MAX_SLOTS {
        pick += 1;
    }
    let slot = SLOTS[pick];
    // Las semanas se alinean a lunes (la epoca de Project cae en sabado).
    let shift = if slot == 604800 { 2 * 86400 } else { 0 };
    let start = (min - shift).div_euclid(slot) * slot + shift;
    // Un tramo extra al final para los hitos que caen justo en el termino.
    let count = (((max - start + slot - 1) / slot) + 1).clamp(1, MAX_SLOTS) as usize;
    Some(Timeline { start, slot, count })
}

fn bar_type(t: &Task) -> &'static str {
    let zero = t.duration.map_or(false, |d| d.value == 0.0);
    if !t.active {
        "Inactive"
    } else if t.milestone || zero {
        "Milestone"
    } else if t.summary {
        "Summary"
    } else if t.critical {
        "Critical"
    } else {
        "Normal"
    }
}

fn gantt_sheet(ix: &Index, st: &mut Styles) -> String {
    let p = ix.p;
    let table: [(&str, Key, f64); 9] = [
        ("ID", Key::Id, 6.0),
        ("Task Name", Key::Name, 54.0),
        ("Duration", Key::Duration, 10.0),
        ("Start", Key::Start, 15.0),
        ("Finish", Key::Finish, 15.0),
        ("% Complete", Key::Field(tf::PERCENT_COMPLETE), 10.0),
        ("Predecessors", Key::Preds, 13.0),
        ("Resource Names", Key::ResNames, 34.0),
        ("Bar", Key::Mode, 10.0),
    ];
    let (c_start, c_finish, c_type) = (3usize, 4usize, 8usize);
    let first_tl = table.len();
    let tl = timeline(p);
    let hourly = tl.as_ref().map_or(true, |t| t.slot < 86400);

    let head = st.xf(Xf { font: F_HEADER, fill: FILL_HEADER, halign: 2, wrap: true, ..Xf::default() });
    let head_left = st.xf(Xf { font: F_HEADER, fill: FILL_HEADER, halign: 1, ..Xf::default() });
    let fmt_group = st.fmt(if hourly { "ddd dd/mm/yyyy" } else { "mmm yyyy" });
    let fmt_slot = st.fmt(if hourly { "hh:mm" } else { "dd" });
    let head_group = st.xf(Xf { font: F_HEADER, fill: FILL_HEADER_EXTRA, fmt: fmt_group, halign: 1, ..Xf::default() });
    let head_slot = st.xf(Xf { font: F_TIME, fill: FILL_HEADER_CUSTOM, fmt: fmt_slot, halign: 2, rotate: hourly, ..Xf::default() });
    let hidden_fmt = st.fmt(";;;");
    let sentinel = st.xf(Xf { fmt: hidden_fmt, ..Xf::default() });

    let mut rows = Rows::new();
    let mut merges = Vec::new();

    // Fila 1: titulo del proyecto y agrupacion de la escala (dia o mes).
    rows.start(" ht=\"18\" customHeight=\"1\"");
    rows.text(0, head_left, p.props.title.as_deref().unwrap_or("Gantt"));
    for c in 1..first_tl {
        rows.blank(c, head_left);
    }
    merges.push(format!("A1:{}1", col_name(first_tl - 1)));
    if let Some(t) = &tl {
        let group_of = |secs: i64| {
            let (y, m, d, ..) = civil(secs);
            if hourly {
                (y, m, d)
            } else {
                (y, m, 0)
            }
        };
        let mut run_start = 0usize;
        for i in 0..=t.count {
            let cur = t.start + i as i64 * t.slot;
            let boundary = i == t.count || (i > 0 && group_of(cur) != group_of(cur - t.slot));
            if boundary {
                if i - run_start > 1 {
                    merges.push(format!("{}1:{}1", col_name(first_tl + run_start), col_name(first_tl + i - 1)));
                }
                run_start = i;
            }
            if i < t.count {
                if i == run_start {
                    rows.num(first_tl + i, head_group, excel_serial(cur));
                } else {
                    rows.blank(first_tl + i, head_group);
                }
            }
        }
    }

    // Fila 2: encabezados de la tabla y de cada tramo de tiempo.
    rows.start(if hourly { " ht=\"34\" customHeight=\"1\"" } else { " ht=\"18\" customHeight=\"1\"" });
    for (i, (title, _, _)) in table.iter().enumerate() {
        rows.text(i, head, title);
    }
    if let Some(t) = &tl {
        for i in 0..t.count {
            rows.num(first_tl + i, head_slot, excel_serial(t.start + i as i64 * t.slot));
        }
        // Celda centinela: fin del ultimo tramo (la usan las formulas; no se muestra).
        rows.num(first_tl + t.count, sentinel, excel_serial(t.start + t.count as i64 * t.slot));
    }

    let mut max_outline = 0usize;
    for t in &p.tasks {
        let level = (t.outline_level.max(0) as usize).min(7);
        max_outline = max_outline.max(level);
        rows.start(&if level > 0 { format!(" outlineLevel=\"{}\"", level) } else { String::new() });
        let font = task_font(t);
        for (i, (_, key, _)) in table.iter().enumerate() {
            if i == c_type {
                let s = st.xf(Xf { font, halign: 2, ..Xf::default() });
                rows.text(i, s, bar_type(t));
                continue;
            }
            let cell = ix.task_cell(t, key);
            let indent = if *key == Key::Name { (t.outline_level.max(0) as u8).min(15) } else { 0 };
            data_cell(&mut rows, st, i, &cell, font, indent, false);
        }
    }

    let last_row = rows.row;
    let mut conditional = String::new();
    let mut widths: Vec<f64> = table.iter().map(|c| c.2).collect();
    if let Some(t) = &tl {
        widths.extend(std::iter::repeat(if hourly { 2.7 } else { 3.2 }).take(t.count));
        widths.push(1.0);
        if last_row >= 3 {
            let (a, b) = (col_name(first_tl), col_name(first_tl + 1));
            let (s, f, ty) = (col_name(c_start), col_name(c_finish), col_name(c_type));
            conditional.push_str(&format!("<conditionalFormatting sqref=\"{}3:{}{}\">", a, col_name(first_tl + t.count - 1), last_row));
            for (i, (name, _, _)) in BARS.iter().enumerate() {
                // Un hito ocupa solo el tramo donde cae; el resto, todos los tramos que tocan.
                let formula = match *name {
                    "Milestone" => format!("AND(${ty}3=\"Milestone\",${s}3>={a}$2,${s}3<{b}$2)", ty = ty, s = s, a = a, b = b),
                    "Normal" => format!("AND(${s}3<{b}$2,${f}3>{a}$2)", s = s, f = f, a = a, b = b),
                    other => format!("AND(${ty}3=\"{o}\",${s}3<{b}$2,${f}3>{a}$2)", ty = ty, o = other, s = s, f = f, a = a, b = b),
                };
                conditional.push_str(&format!("<cfRule type=\"expression\" dxfId=\"{}\" priority=\"{}\" stopIfTrue=\"1\"><formula>{}</formula></cfRule>", i, i + 1, esc(&formula)));
            }
            conditional.push_str("</conditionalFormatting>");
        }
    }

    sheet_xml(
        rows,
        &SheetOpts { freeze: (2, 2), widths, filter: None, merges, conditional, max_outline, landscape: true },
    )
}

// ---------------------------------------------------------------------------------------------
// Hojas de datos
// ---------------------------------------------------------------------------------------------

/// Ancho de columna que deja leer el titulo en dos lineas junto al boton de filtro.
fn fit_width(width: f64, title: &str) -> f64 {
    let longest = title.split_whitespace().map(|w| w.chars().count()).max().unwrap_or(0) as f64;
    let half = (title.chars().count() as f64 / 2.0).ceil();
    width.max(longest + 6.5).max(half + 4.0)
}

fn header_row(rows: &mut Rows, st: &mut Styles, cols: &[(String, u8)]) {
    rows.start(" ht=\"32\" customHeight=\"1\"");
    for (i, (title, fill)) in cols.iter().enumerate() {
        let s = st.xf(Xf { font: F_HEADER, fill: *fill, halign: 2, wrap: true, top: true, ..Xf::default() });
        rows.text(i, s, title);
    }
}

fn tasks_sheet(ix: &Index, st: &mut Styles, cols: &[Col], view_cols: usize) -> (String, String) {
    let p = ix.p;
    let mut rows = Rows::new();
    let heads: Vec<(String, u8)> = cols
        .iter()
        .enumerate()
        .map(|(i, c)| (c.title.clone(), if i < view_cols { FILL_HEADER } else if c.custom { FILL_HEADER_CUSTOM } else { FILL_HEADER_EXTRA }))
        .collect();
    header_row(&mut rows, st, &heads);
    let mut max_outline = 0usize;
    for t in &p.tasks {
        let level = (t.outline_level.max(0) as usize).min(7);
        max_outline = max_outline.max(level);
        rows.start(&if level > 0 { format!(" outlineLevel=\"{}\"", level) } else { String::new() });
        let font = task_font(t);
        for (i, c) in cols.iter().enumerate() {
            let cell = ix.task_cell(t, &c.key);
            let indent = if c.key == Key::Name { (t.outline_level.max(0) as u8).min(15) } else { 0 };
            data_cell(&mut rows, st, i, &cell, font, indent, false);
        }
    }
    let range = format!("A1:{}{}", col_name(cols.len().saturating_sub(1)), rows.row.max(1));
    let name_col = cols.iter().position(|c| c.key == Key::Name).unwrap_or(0);
    let freeze_cols = if name_col < 8 { name_col + 1 } else { 1 };
    let xml = sheet_xml(
        rows,
        &SheetOpts {
            freeze: (freeze_cols, 1),
            widths: cols.iter().map(|c| fit_width(c.width, &c.title)).collect(),
            filter: Some(range.clone()),
            merges: Vec::new(),
            conditional: String::new(),
            max_outline,
            landscape: true,
        },
    );
    (xml, range)
}

fn resources_sheet(ix: &Index, st: &mut Styles) -> (String, String) {
    let cols = schema::resource_columns(ix);
    let mut rows = Rows::new();
    let heads: Vec<(String, u8)> = cols.iter().map(|c| (c.title.clone(), if c.custom { FILL_HEADER_CUSTOM } else { FILL_HEADER })).collect();
    header_row(&mut rows, st, &heads);
    for r in ix.p.resources.iter().filter(|r| r.name.is_some()) {
        rows.start("");
        for (i, c) in cols.iter().enumerate() {
            data_cell(&mut rows, st, i, &ix.resource_cell(r, &c.key), F_NORMAL, 0, false);
        }
    }
    let range = format!("A1:{}{}", col_name(cols.len().saturating_sub(1)), rows.row.max(1));
    let xml = sheet_xml(
        rows,
        &SheetOpts {
            freeze: (0, 1),
            widths: cols.iter().map(|c| fit_width(c.width, &c.title)).collect(),
            filter: Some(range.clone()),
            merges: Vec::new(),
            conditional: String::new(),
            max_outline: 0,
            landscape: true,
        },
    );
    (xml, range)
}

fn assignments_sheet(ix: &Index, st: &mut Styles) -> (String, String) {
    let p = ix.p;
    let cols: [(&str, f64); 9] = [
        ("Task ID", 8.0),
        ("Task Name", 54.0),
        ("Resource ID", 9.0),
        ("Resource Name", 32.0),
        ("Resource Group", 28.0),
        ("Units", 8.0),
        ("Work", 11.0),
        ("Start", 15.0),
        ("Finish", 15.0),
    ];
    let mut rows = Rows::new();
    let heads: Vec<(String, u8)> = cols.iter().map(|c| (c.0.to_string(), FILL_HEADER)).collect();
    header_row(&mut rows, st, &heads);
    for t in &p.tasks {
        for (a, r) in ix.assignments(t.uid) {
            rows.start("");
            let font = if t.active { F_NORMAL } else { F_INACTIVE };
            let opt = |c: Option<Cell>| c.unwrap_or(Cell::Empty);
            let cells = [
                Cell::Int(t.id as i64),
                opt(t.name.clone().map(Cell::Text)),
                Cell::Int(r.id as i64),
                opt(r.name.clone().map(Cell::Text)),
                ix.resource_cell(r, &Key::Field(crate::mpp::fields::resource::GROUP)),
                opt(a.units.map(Cell::Num)),
                opt(a.work.map(Cell::Dur)),
                opt(a.start.map(Cell::Date)),
                opt(a.finish.map(Cell::Date)),
            ];
            for (i, c) in cells.iter().enumerate() {
                data_cell(&mut rows, st, i, c, font, 0, false);
            }
        }
    }
    let range = format!("A1:{}{}", col_name(cols.len() - 1), rows.row.max(1));
    let xml = sheet_xml(
        rows,
        &SheetOpts {
            freeze: (0, 1),
            widths: cols.iter().map(|c| fit_width(c.1, c.0)).collect(),
            filter: Some(range.clone()),
            merges: Vec::new(),
            conditional: String::new(),
            max_outline: 0,
            landscape: true,
        },
    );
    (xml, range)
}

fn project_sheet(ix: &Index, st: &mut Styles) -> String {
    let p = ix.p;
    let title = st.xf(Xf { font: F_TITLE, ..Xf::default() });
    let label = st.xf(Xf { font: F_BOLD, fill: FILL_LABEL, halign: 1, ..Xf::default() });
    let note = st.xf(Xf { font: F_NOTE, halign: 1, wrap: true, ..Xf::default() });
    let section = st.xf(Xf { font: F_HEADER, fill: FILL_HEADER, halign: 1, ..Xf::default() });
    let mut rows = Rows::new();

    rows.start(" ht=\"24\" customHeight=\"1\"");
    rows.text(0, title, p.props.title.as_deref().unwrap_or("Proyecto"));
    rows.skip(1);

    let text = |v: &Option<String>| v.clone().map_or(Cell::Empty, Cell::Text);
    let date = |v: Option<i64>| v.map_or(Cell::Empty, Cell::Date);
    let work: f64 = p.tasks.iter().filter(|t| !t.summary && t.active).filter_map(|t| match t.fields.get(&tf::WORK) {
        Some(Value::Dur(d)) => crate::mpp::util::to_hours(&ix.sc, *d),
        _ => None,
    }).sum();
    let items: Vec<(&str, Cell)> = vec![
        ("Título", text(&p.props.title)),
        ("Autor", text(&p.props.author)),
        ("Última modificación por", text(&p.props.last_author)),
        ("Empresa", text(&p.props.company)),
        ("Responsable", text(&p.props.manager)),
        ("Inicio del proyecto", date(p.props.start)),
        ("Fin del proyecto", date(p.props.finish)),
        ("Fecha de estado", date(p.props.status_date)),
        ("Calendario del proyecto", text(&p.props.default_calendar)),
        ("Horas por día", Cell::Num(p.props.minutes_per_day as f64 / 60.0)),
        ("Horas por semana", Cell::Num(p.props.minutes_per_week as f64 / 60.0)),
        ("Tareas", Cell::Int(p.tasks.len() as i64)),
        ("Tareas resumen", Cell::Int(p.tasks.iter().filter(|t| t.summary).count() as i64)),
        ("Hitos", Cell::Int(p.tasks.iter().filter(|t| t.milestone).count() as i64)),
        ("Tareas críticas", Cell::Int(p.tasks.iter().filter(|t| t.critical && !t.summary).count() as i64)),
        ("Tareas inactivas", Cell::Int(p.tasks.iter().filter(|t| !t.active).count() as i64)),
        ("Recursos", Cell::Int(p.resources.iter().filter(|r| r.name.is_some()).count() as i64)),
        ("Asignaciones", Cell::Int(p.tasks.iter().map(|t| ix.assignments(t.uid).len()).sum::<usize>() as i64)),
        ("Trabajo total (tareas activas)", Cell::Dur(Dur { value: (work * 100.0).round() / 100.0, unit: Unit::Hours })),
        ("Archivo creado con", Cell::Text(p.props.app_name.clone())),
    ];
    for (name, cell) in items {
        if cell.is_empty() {
            continue;
        }
        rows.start("");
        rows.text(0, label, name);
        // Los valores van alineados a la izquierda para leerse como ficha.
        match cell {
            Cell::Text(s) => {
                let x = st.xf(Xf { halign: 1, ..Xf::default() });
                rows.text(1, x, &s);
            }
            Cell::Date(d) => {
                let fmt = st.fmt(FMT_DATETIME);
                let x = st.xf(Xf { fmt, halign: 1, ..Xf::default() });
                rows.num(1, x, excel_serial(d));
            }
            Cell::Dur(d) => {
                let fmt = st.fmt(&format!("#,##0.##\"{}\"", unit_label(d.unit)));
                let x = st.xf(Xf { fmt, halign: 1, ..Xf::default() });
                rows.num(1, x, d.value);
            }
            Cell::Int(v) => {
                let x = st.xf(Xf { halign: 1, ..Xf::default() });
                rows.num(1, x, v as f64);
            }
            Cell::Num(v) => {
                let x = st.xf(Xf { halign: 1, ..Xf::default() });
                rows.num(1, x, v);
            }
            _ => {}
        }
    }

    rows.skip(1);
    rows.start("");
    rows.text(0, section, "Colores del Gantt");
    rows.blank(1, section);
    // La muestra de color se pinta con las mismas reglas que las barras.
    let swatch = st.xf(Xf { font: F_HEADER, halign: 2, ..Xf::default() });
    for (name, _, desc) in BARS {
        rows.start("");
        rows.text(0, label, desc);
        rows.text(1, swatch, name);
    }
    let legend_last = rows.row;
    let legend_first = legend_last + 1 - BARS.len();
    let mut conditional = format!("<conditionalFormatting sqref=\"B{}:B{}\">", legend_first, legend_last);
    for (i, (name, _, _)) in BARS.iter().enumerate() {
        conditional.push_str(&format!(
            "<cfRule type=\"expression\" dxfId=\"{}\" priority=\"{}\" stopIfTrue=\"1\"><formula>$B{}=\"{}\"</formula></cfRule>",
            i,
            i + 1,
            legend_first,
            name
        ));
    }
    conditional.push_str("</conditionalFormatting>");

    rows.skip(1);
    for line in [
        "Hoja Gantt: las barras se calculan con formato condicional a partir de Start y Finish; si edita esas fechas, la barra se mueve.",
        "Hoja Tareas: mismas columnas y títulos que la tabla del Gantt en Project; a la derecha, datos de estructura y demás campos personalizados con contenido.",
        "Las filas se pueden contraer y expandir por nivel de esquema con los botones 1-2-3 del margen izquierdo.",
        "Las tareas inactivas aparecen en gris y tachadas, igual que en Project.",
    ] {
        rows.start(" ht=\"26\" customHeight=\"1\"");
        rows.text(0, note, line);
        rows.blank(1, note);
    }
    let note_last = rows.row;
    let merges: Vec<String> = (note_last - 3..=note_last).map(|r| format!("A{r}:B{r}", r = r)).collect();

    sheet_xml(
        rows,
        &SheetOpts { freeze: (0, 0), widths: vec![34.0, 70.0], filter: None, merges, conditional, max_outline: 0, landscape: false },
    )
}

// ---------------------------------------------------------------------------------------------
// Libro
// ---------------------------------------------------------------------------------------------

/// Genera el .xlsx completo del proyecto.
pub fn workbook(p: &Project) -> Vec<u8> {
    let ix = Index::new(p);
    let mut st = Styles::new();
    let cols = schema::task_columns(&ix);
    // Las primeras columnas son las de la vista de Project; se distinguen por color de encabezado.
    let view_cols = cols.iter().position(|c| c.key == Key::OutlineLevel).unwrap_or(cols.len());

    let gantt = gantt_sheet(&ix, &mut st);
    let (tasks, tasks_range) = tasks_sheet(&ix, &mut st, &cols, view_cols);
    let (resources, res_range) = resources_sheet(&ix, &mut st);
    let (assignments, asg_range) = assignments_sheet(&ix, &mut st);
    let project = project_sheet(&ix, &mut st);

    let sheets: [(&str, &String); 5] = [("Gantt", &gantt), ("Tareas", &tasks), ("Recursos", &resources), ("Asignaciones", &assignments), ("Proyecto", &project)];

    let abs = |range: &str| {
        // "A1:K20" -> "$A$1:$K$20"
        range
            .split(':')
            .map(|cell| {
                let letters: String = cell.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
                let digits: String = cell.chars().skip(letters.len()).collect();
                format!("${}${}", letters, digits)
            })
            .collect::<Vec<_>>()
            .join(":")
    };

    let mut wb = String::from(XML_HEAD);
    wb.push_str(&format!("<workbook xmlns=\"{}\" xmlns:r=\"{}\"><bookViews><workbookView xWindow=\"0\" yWindow=\"0\" windowWidth=\"28800\" windowHeight=\"15000\"/></bookViews><sheets>", NS_MAIN, NS_REL));
    for (i, (name, _)) in sheets.iter().enumerate() {
        wb.push_str(&format!("<sheet name=\"{}\" sheetId=\"{}\" r:id=\"rId{}\"/>", name, i + 1, i + 1));
    }
    wb.push_str("</sheets><definedNames>");
    wb.push_str(&format!("<definedName name=\"_xlnm._FilterDatabase\" localSheetId=\"1\" hidden=\"1\">Tareas!{}</definedName>", abs(&tasks_range)));
    wb.push_str(&format!("<definedName name=\"_xlnm._FilterDatabase\" localSheetId=\"2\" hidden=\"1\">Recursos!{}</definedName>", abs(&res_range)));
    wb.push_str(&format!("<definedName name=\"_xlnm._FilterDatabase\" localSheetId=\"3\" hidden=\"1\">Asignaciones!{}</definedName>", abs(&asg_range)));
    wb.push_str("<definedName name=\"_xlnm.Print_Titles\" localSheetId=\"0\">Gantt!$1:$2</definedName>");
    wb.push_str("<definedName name=\"_xlnm.Print_Titles\" localSheetId=\"1\">Tareas!$1:$1</definedName>");
    wb.push_str("</definedNames></workbook>");

    let mut rels = String::from(XML_HEAD);
    rels.push_str("<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">");
    for i in 0..sheets.len() {
        rels.push_str(&format!("<Relationship Id=\"rId{n}\" Type=\"{}/worksheet\" Target=\"worksheets/sheet{n}.xml\"/>", NS_REL, n = i + 1));
    }
    rels.push_str(&format!("<Relationship Id=\"rId{}\" Type=\"{}/styles\" Target=\"styles.xml\"/></Relationships>", sheets.len() + 1, NS_REL));

    let mut types = String::from(XML_HEAD);
    types.push_str("<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/>");
    for i in 0..sheets.len() {
        types.push_str(&format!("<Override PartName=\"/xl/worksheets/sheet{}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>", i + 1));
    }
    types.push_str("<Override PartName=\"/xl/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml\"/></Types>");

    let root_rels = format!(
        "{}<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"{}/officeDocument\" Target=\"xl/workbook.xml\"/></Relationships>",
        XML_HEAD, NS_REL
    );

    let mut zip = Zip::new();
    zip.add("[Content_Types].xml", types.as_bytes());
    zip.add("_rels/.rels", root_rels.as_bytes());
    zip.add("xl/workbook.xml", wb.as_bytes());
    zip.add("xl/_rels/workbook.xml.rels", rels.as_bytes());
    zip.add("xl/styles.xml", st.xml().as_bytes());
    for (i, (_, xml)) in sheets.iter().enumerate() {
        zip.add(&format!("xl/worksheets/sheet{}.xml", i + 1), xml.as_bytes());
    }
    zip.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nombres_de_columna() {
        let casos = [(0, "A"), (25, "Z"), (26, "AA"), (51, "AZ"), (52, "BA"), (701, "ZZ"), (702, "AAA")];
        for (n, esperado) in casos {
            assert_eq!(col_name(n), esperado);
        }
    }

    #[test]
    fn escape_xml() {
        assert_eq!(esc("a<b & \"c\"\u{1}"), "a&lt;b &amp; &quot;c&quot;");
        assert_eq!(num_str(24.0), "24");
        assert_eq!(num_str(0.5), "0.5");
    }
}
