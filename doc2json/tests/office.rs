//! Lectores de PowerPoint y Excel con paquetes armados dentro de la prueba: cada caso prueba
//! una regla de lectura (orden, vinetas, combinaciones, fechas, ocultas) sin depender de
//! archivos de terceros.

use doc2json::doc::{Block, Document, Kind};
use doc2json::{detect, read, Format};

/// ZIP sin compresion (metodo 0) con varias partes.
fn zip(partes: &[(&str, &str)]) -> Vec<u8> {
    let partes: Vec<(&str, &[u8])> = partes.iter().map(|(n, c)| (*n, c.as_bytes())).collect();
    zip_con_alias(&partes, &[])
}

/// Como `zip`, con el contenido de cada parte en un `String`.
fn zip_de(partes: &[(&str, String)]) -> Vec<u8> {
    let partes: Vec<(&str, &[u8])> = partes.iter().map(|(n, c)| (*n, c.as_bytes())).collect();
    zip_con_alias(&partes, &[])
}

/// ZIP sin compresion (metodo 0). Cada alias (nombre, parte) es un nombre mas en el directorio
/// central que apunta a los datos de una parte ya escrita: el mismo contenido con otro nombre,
/// sin repetirlo en el archivo.
fn zip_con_alias(partes: &[(&str, &[u8])], alias: &[(String, &str)]) -> Vec<u8> {
    let le16 = |v: usize| (v as u16).to_le_bytes();
    let le32 = |v: usize| (v as u32).to_le_bytes();
    let entrada = |central: &mut Vec<u8>, n: &[u8], largo: usize, desde: usize| {
        central.extend_from_slice(&[0x50, 0x4B, 0x01, 0x02, 20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        central.extend_from_slice(&le32(largo));
        central.extend_from_slice(&le32(largo));
        central.extend_from_slice(&le16(n.len()));
        central.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        central.extend_from_slice(&le32(desde));
        central.extend_from_slice(n);
    };
    let mut z = Vec::new();
    let mut central = Vec::new();
    let mut lugares = std::collections::HashMap::new();
    for (nombre, c) in partes {
        let n = nombre.as_bytes();
        let desde = z.len();
        z.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        z.extend_from_slice(&le32(c.len()));
        z.extend_from_slice(&le32(c.len()));
        z.extend_from_slice(&le16(n.len()));
        z.extend_from_slice(&[0, 0]);
        z.extend_from_slice(n);
        z.extend_from_slice(c);
        entrada(&mut central, n, c.len(), desde);
        lugares.insert(*nombre, (c.len(), desde));
    }
    for (nombre, parte) in alias {
        let (largo, desde) = lugares[parte];
        entrada(&mut central, nombre.as_bytes(), largo, desde);
    }
    let total = partes.len() + alias.len();
    assert!(total <= 65_535, "un ZIP sin ZIP64 admite 65.535 entradas");
    let inicio = z.len();
    z.extend_from_slice(&central);
    z.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06, 0, 0, 0, 0]);
    z.extend_from_slice(&le16(total));
    z.extend_from_slice(&le16(total));
    z.extend_from_slice(&le32(central.len()));
    z.extend_from_slice(&le32(inicio));
    z.extend_from_slice(&[0, 0]);
    z
}

const NS: &str = r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main""#;
const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

fn rels(lista: &[(&str, &str, &str)]) -> String {
    let mut s = String::from(r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#);
    for (id, tipo, destino) in lista {
        let externo = if destino.starts_with("http") { r#" TargetMode="External""# } else { "" };
        s.push_str(&format!(r#"<Relationship Id="{}" Type="{}/{}" Target="{}"{}/>"#, id, REL, tipo, destino, externo));
    }
    s + "</Relationships>"
}

/// Forma de texto en la posicion (x, y), con marcador de posicion opcional.
fn forma(ph: &str, x: i64, y: i64, parrafos: &str) -> String {
    format!(
        r#"<p:sp><p:nvSpPr><p:cNvPr id="2" name="f"/><p:cNvSpPr/><p:nvPr>{}</p:nvPr></p:nvSpPr><p:spPr><a:xfrm><a:off x="{}" y="{}"/><a:ext cx="100" cy="100"/></a:xfrm></p:spPr><p:txBody><a:bodyPr/>{}</p:txBody></p:sp>"#,
        ph, x, y, parrafos
    )
}

fn diapositiva(formas: &str, oculta: bool) -> String {
    format!(r#"<?xml version="1.0"?><p:sld {}{}><p:cSld><p:spTree><p:nvGrpSpPr/><p:grpSpPr/>{}</p:spTree></p:cSld></p:sld>"#, NS, if oculta { r#" show="0""# } else { "" }, formas)
}

fn partes_de_presentacion() -> Vec<(&'static str, String)> {
    let tabla = r#"<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="5" name="t"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr><p:xfrm><a:off x="0" y="3000000"/></p:xfrm><a:graphic><a:graphicData><a:tbl><a:tblGrid><a:gridCol/><a:gridCol/><a:gridCol/></a:tblGrid>
        <a:tr><a:tc gridSpan="2"><a:txBody><a:p><a:r><a:t>Equipo</a:t></a:r></a:p></a:txBody></a:tc><a:tc hMerge="1"><a:txBody><a:p/></a:txBody></a:tc><a:tc rowSpan="2"><a:txBody><a:p><a:r><a:t>Riesgo</a:t></a:r></a:p></a:txBody></a:tc></a:tr>
        <a:tr><a:tc><a:txBody><a:p><a:r><a:t>Grua</a:t></a:r></a:p></a:txBody></a:tc><a:tc><a:txBody><a:p><a:r><a:t>50 t</a:t></a:r></a:p></a:txBody></a:tc><a:tc vMerge="1"><a:txBody><a:p/></a:txBody></a:tc></a:tr>
        </a:tbl></a:graphicData></a:graphic></p:graphicFrame>"#;
    let imagen = r#"<p:pic><p:nvPicPr><p:cNvPr id="6" name="Imagen" descr="Diagrama del circuito"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill><a:blip r:embed="rId2"/></p:blipFill><p:spPr><a:xfrm><a:off x="0" y="5000000"/></a:xfrm></p:spPr></p:pic>"#;
    let grupo = format!(
        r#"<p:grpSp><p:nvGrpSpPr><p:cNvPr id="7" name="g"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x="0" y="4000000"/></a:xfrm></p:grpSpPr>{}{}</p:grpSp>"#,
        forma("", 5000000, 4000000, "<a:p><a:r><a:t>Grupo derecha</a:t></a:r></a:p>"),
        forma("", 0, 4000000, "<a:p><a:r><a:t>Grupo izquierda</a:t></a:r></a:p>"),
    );
    let s1 = diapositiva(
        &[
            // Un cuadro de texto mas arriba que el titulo: el titulo igual sale primero.
            forma("", 0, 100000, r#"<a:p><a:r><a:rPr b="1"/><a:t>Nota previa</a:t></a:r><a:r><a:rPr><a:hlinkClick r:id="rId4"/></a:rPr><a:t> enlace</a:t></a:r></a:p>"#),
            forma(r#"<p:ph type="title"/>"#, 0, 500000, "<a:p><a:r><a:t>Izaje de cargas</a:t></a:r></a:p>"),
            forma(
                r#"<p:ph idx="1"/>"#,
                0,
                1500000,
                r#"<a:p><a:r><a:t>Antes de izar</a:t></a:r></a:p><a:p><a:pPr lvl="1"/><a:r><a:t>Revisar eslingas</a:t></a:r></a:p><a:p><a:pPr><a:buNone/></a:pPr><a:r><a:t>Sin vineta</a:t></a:r></a:p>"#,
            ),
            forma(
                "",
                0,
                2500000,
                r#"<a:p><a:pPr><a:buAutoNum type="alphaLcParenR"/></a:pPr><a:r><a:t>Primero</a:t></a:r></a:p><a:p><a:pPr><a:buAutoNum type="alphaLcParenR"/></a:pPr><a:r><a:t>Segundo</a:t></a:r></a:p><a:p><a:pPr><a:buFont typeface="Wingdings"/><a:buChar char="&#xF0FC;"/></a:pPr><a:r><a:t>Listo</a:t></a:r></a:p>"#,
            ),
            tabla.to_string(),
            grupo,
            imagen.to_string(),
            forma(r#"<p:ph type="sldNum" idx="12"/>"#, 0, 6000000, "<a:p><a:r><a:t>1</a:t></a:r></a:p>"),
        ]
        .concat(),
        false,
    );
    let s2 = diapositiva(&forma(r#"<p:ph type="ctrTitle"/>"#, 0, 0, "<a:p><a:r><a:t>Portada</a:t></a:r></a:p>"), false);
    let s3 = diapositiva(&forma(r#"<p:ph type="title"/>"#, 0, 0, "<a:p><a:r><a:t>Respaldo oculto</a:t></a:r></a:p>"), true);
    let notas = format!(
        r#"<?xml version="1.0"?><p:notes {}><p:cSld><p:spTree>{}{}</p:spTree></p:cSld></p:notes>"#,
        NS,
        forma(r#"<p:ph type="sldImg"/>"#, 0, 0, ""),
        forma(r#"<p:ph type="body" idx="1"/>"#, 0, 1000, "<a:p><a:r><a:t>Recordar la charla de 5 minutos.</a:t></a:r></a:p>"),
    );
    let presentacion = format!(
        r#"<?xml version="1.0"?><p:presentation {}><p:sldIdLst><p:sldId id="256" r:id="rId3"/><p:sldId id="257" r:id="rId1"/><p:sldId id="258" r:id="rId2"/></p:sldIdLst></p:presentation>"#,
        NS
    );
    let partes: &[(&'static str, &str)] = &[
        ("_rels/.rels", &rels(&[("rId1", "officeDocument", "ppt/presentation.xml"), ("rId2", "metadata/core-properties", "docProps/core.xml")])),
        ("docProps/core.xml", r#"<?xml version="1.0"?><cp:coreProperties xmlns:cp="x" xmlns:dc="y"><dc:title>Charla de izaje</dc:title></cp:coreProperties>"#),
        ("ppt/presentation.xml", &presentacion),
        ("ppt/_rels/presentation.xml.rels", &rels(&[("rId1", "slide", "slides/slide1.xml"), ("rId2", "slide", "slides/slide3.xml"), ("rId3", "slide", "slides/slide2.xml")])),
        ("ppt/slides/slide1.xml", &s1),
        ("ppt/slides/_rels/slide1.xml.rels", &rels(&[("rId2", "image", "../media/image1.png"), ("rId3", "notesSlide", "../notesSlides/notesSlide1.xml"), ("rId4", "hyperlink", "https://example.com/izaje")])),
        ("ppt/slides/slide2.xml", &s2),
        ("ppt/slides/slide3.xml", &s3),
        ("ppt/notesSlides/notesSlide1.xml", &notas),
    ];
    partes.iter().map(|(n, c)| (*n, c.to_string())).collect()
}

fn presentacion() -> Vec<u8> {
    zip_de(&partes_de_presentacion())
}

fn textos(d: &Document) -> Vec<String> {
    d.content.iter().map(|b| b.text.clone()).collect()
}

fn bloque<'a>(d: &'a Document, texto: &str) -> &'a Block {
    d.content.iter().find(|b| b.text == texto).unwrap_or_else(|| panic!("falta el bloque «{}»", texto))
}

#[test]
fn powerpoint_orden_y_tipos() {
    let bytes = presentacion();
    assert_eq!(detect(&bytes).unwrap(), Format::Pptx);
    let d = read(&bytes).unwrap();
    assert_eq!(d.kind, "pptx");
    assert_eq!(d.page_count, Some(3));
    let t = textos(&d);
    // Orden de la presentacion (no el de los archivos): la portada es la primera.
    assert_eq!(t[0], "Portada");
    assert_eq!(bloque(&d, "Portada").page, Some(1));
    // Dentro de la diapositiva: titulo primero, luego de arriba hacia abajo.
    assert_eq!(&t[1..6], &["Izaje de cargas", "Nota previa enlace", "Antes de izar", "Revisar eslingas", "Sin vineta"]);
    let titulo = bloque(&d, "Izaje de cargas");
    assert_eq!((titulo.kind, titulo.level, titulo.page), (Kind::Heading, 1, Some(2)));
    let nota = bloque(&d, "Nota previa enlace");
    assert_eq!(nota.kind, Kind::Paragraph);
    assert!(nota.runs[0].bold);
    assert_eq!(nota.runs[1].link.as_deref(), Some("https://example.com/izaje"));
    let item = bloque(&d, "Antes de izar");
    assert_eq!((item.kind, item.level, item.marker.as_deref()), (Kind::ListItem, 0, Some("\u{2022}")));
    assert_eq!(bloque(&d, "Revisar eslingas").level, 1);
    assert_eq!(bloque(&d, "Sin vineta").kind, Kind::Paragraph);
    // Cuadro de texto con numeracion y con vineta propia (Wingdings 0xFC = visto bueno).
    assert_eq!(bloque(&d, "Primero").marker.as_deref(), Some("a)"));
    assert_eq!(bloque(&d, "Segundo").marker.as_deref(), Some("b)"));
    assert!(bloque(&d, "Segundo").ordered);
    assert_eq!(bloque(&d, "Listo").marker.as_deref(), Some("\u{2713}"));
    // Numero de diapositiva y diapositiva oculta: fuera.
    assert!(!t.iter().any(|s| s == "1" || s == "Respaldo oculto"));
    assert!(d.warnings.iter().any(|w| w.contains("1 diapositivas ocultas")));
}

#[test]
fn powerpoint_tablas_grupos_imagenes_y_notas() {
    let d = read(&presentacion()).unwrap();
    let tabla = d.content.iter().find(|b| b.kind == Kind::Table).expect("tabla");
    let filas: Vec<Vec<(String, u32, u32, bool)>> = tabla.rows.iter().map(|r| r.iter().map(|c| (c.text.clone(), c.colspan, c.rowspan, c.merged)).collect()).collect();
    assert_eq!(
        filas,
        vec![
            vec![("Equipo".into(), 2, 1, false), ("Riesgo".into(), 1, 2, false)],
            vec![("Grua".into(), 1, 1, false), ("50 t".into(), 1, 1, false), (String::new(), 1, 1, true)],
        ]
    );
    let t = textos(&d);
    let i = t.iter().position(|s| s == "Grupo izquierda").unwrap();
    assert_eq!(t[i + 1], "Grupo derecha");
    let img = d.content.iter().find(|b| b.kind == Kind::Image).expect("imagen");
    assert_eq!((img.name.as_deref(), img.text.as_str(), img.page), (Some("image1.png"), "Diagrama del circuito", Some(2)));
    // El orden completo: tabla (y=3M) antes del grupo (y=4M) antes de la imagen (y=5M).
    let pos = |k: Kind| d.content.iter().position(|b| b.kind == k).unwrap();
    assert!(pos(Kind::Table) < i && i < pos(Kind::Image));
    let notas = &d.extras.iter().find(|(k, _)| *k == "notes").expect("notas").1;
    assert_eq!(notas.len(), 1);
    assert_eq!((notas[0].text.as_str(), notas[0].label.as_deref()), ("Recordar la charla de 5 minutos.", Some("2")));
    assert!(d.meta.iter().any(|(k, _)| *k == "title"));
}

fn partes_de_planilla(fecha1904: bool) -> Vec<(&'static str, String)> {
    let libro = format!(
        r#"<?xml version="1.0"?><workbook xmlns:r="{}"><workbookPr{}/><sheets><sheet name="Datos" sheetId="1" r:id="rId1"/><sheet name="Oculta" sheetId="2" state="hidden" r:id="rId2"/><sheet name="Vacia" sheetId="3" r:id="rId3"/></sheets></workbook>"#,
        REL,
        if fecha1904 { r#" date1904="1""# } else { "" }
    );
    let compartidos = r#"<?xml version="1.0"?><sst><si><t>Nombre</t></si><si><t>Fecha</t></si><si><t>Avance</t></si><si><r><rPr><b/></rPr><t>Juan </t></r><r><t>Perez</t></r><rPh><t>fonetica</t></rPh></si><si><t>Combinada</t></si><si><t>Nota</t></si></sst>"#;
    let estilos = r#"<?xml version="1.0"?><styleSheet><numFmts count="1"><numFmt numFmtId="164" formatCode="dd/mm/yyyy;@"/></numFmts><cellStyleXfs><xf numFmtId="10"/></cellStyleXfs><cellXfs><xf numFmtId="0"/><xf numFmtId="164" applyNumberFormat="1"/><xf numFmtId="10"/><xf numFmtId="22"/></cellXfs></styleSheet>"#;
    let hoja = r#"<?xml version="1.0"?><worksheet><sheetData>
        <row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c><c r="C1" t="s"><v>2</v></c><c r="E1" t="s"><v>5</v></c></row>
        <row r="2"><c r="A2" t="s"><v>3</v></c><c r="B2" s="1"><v>45000</v></c><c r="C2" s="2"><v>0.25</v></c><c r="D2" s="1"/><c r="E2" t="inlineStr"><is><t>en linea</t></is></c></row>
        <row r="3"><c r="A3"><v></v></c></row>
        <row r="4"><c r="A4"><v>0.30000000000000004</v></c><c r="B4" t="b"><v>1</v></c><c r="C4" t="str"><f>A1&amp;"x"</f><v>Nombrex</v></c><c r="E4" t="e"><v>#N/A</v></c></row>
        <row r="5"><c r="A5" t="s"><v>4</v></c><c r="B5"/><c r="C5"/></row>
        <row r="6"><c r="E6" s="3"><v>45000.75</v></c></row>
        </sheetData><mergeCells count="1"><mergeCell ref="A5:C6"/></mergeCells></worksheet>"#;
    let partes: &[(&'static str, &str)] = &[
        ("_rels/.rels", &rels(&[("rId1", "officeDocument", "xl/workbook.xml")])),
        ("xl/workbook.xml", &libro),
        ("xl/_rels/workbook.xml.rels", &rels(&[("rId1", "worksheet", "worksheets/sheet1.xml"), ("rId2", "worksheet", "worksheets/sheet2.xml"), ("rId3", "worksheet", "worksheets/sheet3.xml"), ("rId4", "sharedStrings", "sharedStrings.xml"), ("rId5", "styles", "styles.xml")])),
        ("xl/sharedStrings.xml", compartidos),
        ("xl/styles.xml", estilos),
        ("xl/worksheets/sheet1.xml", hoja),
        ("xl/worksheets/sheet2.xml", r#"<worksheet><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>secreto</t></is></c></row></sheetData></worksheet>"#),
        ("xl/worksheets/sheet3.xml", r#"<worksheet><sheetData/></worksheet>"#),
    ];
    partes.iter().map(|(n, c)| (*n, c.to_string())).collect()
}

fn planilla(fecha1904: bool) -> Vec<u8> {
    zip_de(&partes_de_planilla(fecha1904))
}

#[test]
fn excel_valores_como_se_muestran() {
    let bytes = planilla(false);
    assert_eq!(detect(&bytes).unwrap(), Format::Xlsx);
    let d = read(&bytes).unwrap();
    assert_eq!(d.kind, "xlsx");
    let encabezado = &d.content[0];
    assert_eq!((encabezado.kind, encabezado.text.as_str(), encabezado.page), (Kind::Heading, "Datos", Some(1)));
    let tabla = &d.content[1];
    assert_eq!(tabla.kind, Kind::Table);
    let celdas: Vec<Vec<(String, u32, u32, bool)>> = tabla.rows.iter().map(|r| r.iter().map(|c| (c.text.clone(), c.colspan, c.rowspan, c.merged)).collect()).collect();
    let c = |t: &str| (t.to_string(), 1, 1, false);
    // Sin la columna D ni la fila 3 (vacias) ni la celda vacia al final de la fila 5. La
    // combinacion A5:C6 conserva su alto y su ancho.
    assert_eq!(
        celdas,
        vec![
            vec![c("Nombre"), c("Fecha"), c("Avance"), c("Nota")],
            vec![c("Juan Perez"), c("2023-03-15"), c("25%"), c("en linea")],
            vec![c("0.3"), c("TRUE"), c("Nombrex"), c("#N/A")],
            vec![("Combinada".into(), 3, 2, false)],
            vec![(String::new(), 3, 1, true), c("2023-03-15T18:00:00")],
        ]
    );
    // Hoja oculta: no se lee. Hoja vacia: no deja bloques.
    assert!(!d.content.iter().any(|b| b.text.contains("secreto") || b.text == "Vacia"));
    assert!(d.warnings.iter().any(|w| w.contains("1 hojas ocultas")));
    assert_eq!(d.page_count, Some(2));
}

#[test]
fn excel_sistema_de_fechas_1904() {
    let d = read(&planilla(true)).unwrap();
    assert_eq!(d.content[1].rows[1][1].text, "2027-03-16");
}

#[test]
fn excel_con_prefijo_de_espacio_de_nombres() {
    // Asi escriben algunos generadores (por ejemplo, el Open XML SDK de .NET).
    let libro = format!(r#"<x:workbook xmlns:x="m" xmlns:r="{}"><x:sheets><x:sheet name="Hoja1" sheetId="1" r:id="rId1"/></x:sheets></x:workbook>"#, REL);
    let hoja = r#"<x:worksheet xmlns:x="m"><x:sheetData><x:row r="1"><x:c r="A1" t="s"><x:v>0</x:v></x:c><x:c r="B1" s="1"><x:v>45000</x:v></x:c></x:row></x:sheetData><x:mergeCells><x:mergeCell ref="A1:A2"/></x:mergeCells></x:worksheet>"#;
    let bytes = zip(&[
        ("_rels/.rels", &rels(&[("rId1", "officeDocument", "xl/workbook.xml")])),
        ("xl/workbook.xml", &libro),
        ("xl/_rels/workbook.xml.rels", &rels(&[("rId1", "worksheet", "worksheets/sheet1.xml"), ("rId2", "sharedStrings", "sharedStrings.xml"), ("rId3", "styles", "styles.xml")])),
        ("xl/sharedStrings.xml", r#"<x:sst xmlns:x="m"><x:si><x:r><x:t>Turno</x:t></x:r><x:r><x:t xml:space="preserve"> A</x:t></x:r></x:si></x:sst>"#),
        ("xl/styles.xml", r#"<x:styleSheet xmlns:x="m"><x:cellXfs><x:xf numFmtId="0"/><x:xf numFmtId="14"/></x:cellXfs></x:styleSheet>"#),
        ("xl/worksheets/sheet1.xml", hoja),
    ]);
    let d = read(&bytes).unwrap();
    assert_eq!(d.content[0].text, "Hoja1");
    let fila: Vec<&str> = d.content[1].rows[0].iter().map(|c| c.text.as_str()).collect();
    assert_eq!(fila, ["Turno A", "2023-03-15"]);
}

#[test]
fn zip_que_no_es_office_sigue_siendo_error_de_word() {
    let bytes = zip(&[("hola.txt", "hola")]);
    assert_eq!(detect(&bytes).unwrap(), Format::Docx);
    assert!(read(&bytes).is_err());
}

// ---------------------------------------------------------------------------------------------
// Archivos hostiles o raros: topes globales, referencias repetidas, archivos truncados.
// ---------------------------------------------------------------------------------------------

/// Libro de una hoja con el XML de `sheetData` y `mergeCells` dado.
fn libro_con_hoja(hoja: &str) -> Vec<u8> {
    let libro = format!(r#"<?xml version="1.0"?><workbook xmlns:r="{}"><sheets><sheet name="H" sheetId="1" r:id="rId1"/></sheets></workbook>"#, REL);
    zip(&[
        ("_rels/.rels", &rels(&[("rId1", "officeDocument", "xl/workbook.xml")])),
        ("xl/workbook.xml", &libro),
        ("xl/_rels/workbook.xml.rels", &rels(&[("rId1", "worksheet", "worksheets/sheet1.xml")])),
        ("xl/worksheets/sheet1.xml", &format!("<worksheet>{}</worksheet>", hoja)),
    ])
}

/// Letras de columna de Excel (1 = A).
fn columna(mut n: usize) -> String {
    let mut s = String::new();
    while n > 0 {
        s.insert(0, (b'A' + ((n - 1) % 26) as u8) as char);
        n = (n - 1) / 26;
    }
    s
}

fn celdas_de_tablas(d: &Document) -> usize {
    d.content.iter().filter(|b| b.kind == Kind::Table).map(|b| b.rows.iter().map(Vec::len).sum::<usize>()).sum()
}

#[test]
fn excel_hoja_rala_no_se_vuelve_densa() {
    // 2.000 celdas en diagonal: como tabla densa serian 4 millones de celdas.
    let mut filas = String::new();
    for i in 1..=2000usize {
        filas.push_str(&format!(r#"<row r="{0}"><c r="{1}{0}" t="inlineStr"><is><t>v{0}</t></is></c></row>"#, i, columna(i)));
    }
    let d = read(&libro_con_hoja(&format!("<sheetData>{}</sheetData>", filas))).unwrap();
    let total = celdas_de_tablas(&d);
    assert!(total <= 2 * 2000 + 2, "celdas materializadas: {}", total);
    // Ningun valor se pierde: salen como celda y valor.
    let fila = &d.content[1].rows[1];
    assert_eq!((fila[0].text.as_str(), fila[1].text.as_str()), ("B2", "v2"));
    assert_eq!(d.content[1].rows[1999][1].text, "v2000");
    assert!(d.warnings.iter().any(|w| w.contains("dispers")), "{:?}", d.warnings);
}

#[test]
fn excel_combinaciones_solapadas_no_explotan() {
    // Tabla densa de 1000 x 1000 (las primeras 70 filas llenas y la columna A) y 1000
    // combinaciones solapadas A{k}:ALL1000, de abajo hacia arriba: sin tope serian ~5 * 10^8
    // inserciones.
    let mut filas = String::new();
    for r in 1..=1000usize {
        filas.push_str(&format!(r#"<row r="{}">"#, r));
        let ancho = if r <= 70 { 1000 } else { 1 };
        for c in 1..=ancho {
            filas.push_str(&format!(r#"<c r="{}{}"><v>{}</v></c>"#, columna(c), r, c));
        }
        filas.push_str("</row>");
    }
    let mut comb = String::new();
    for k in (1..=1000usize).rev() {
        comb.push_str(&format!(r#"<mergeCell ref="A{}:ALL1000"/>"#, k));
    }
    let inicio = std::time::Instant::now();
    let d = read(&libro_con_hoja(&format!("<sheetData>{}</sheetData><mergeCells>{}</mergeCells>", filas, comb))).unwrap();
    assert!(inicio.elapsed().as_secs() < 20, "tardo {:?}", inicio.elapsed());
    // Excel no admite combinaciones solapadas: vale la primera (A1000:ALL1000, una fila).
    let tabla = d.content.iter().find(|b| b.kind == Kind::Table).unwrap();
    assert_eq!((tabla.rows[999][0].colspan, tabla.rows[999][0].rowspan), (1000, 1));
    assert_eq!(tabla.rows[998][0].colspan, 1);
}

#[test]
fn excel_hojas_repetidas_se_leen_una_vez() {
    let mut hojas = String::new();
    for i in 0..5000 {
        hojas.push_str(&format!(r#"<sheet name="H{}" sheetId="{}" r:id="rId1"/>"#, i, i + 1));
    }
    let libro = format!(r#"<?xml version="1.0"?><workbook xmlns:r="{}"><sheets>{}</sheets></workbook>"#, REL, hojas);
    let bytes = zip(&[
        ("_rels/.rels", &rels(&[("rId1", "officeDocument", "xl/workbook.xml")])),
        ("xl/workbook.xml", &libro),
        ("xl/_rels/workbook.xml.rels", &rels(&[("rId1", "worksheet", "worksheets/sheet1.xml")])),
        ("xl/worksheets/sheet1.xml", r#"<worksheet><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>dato</t></is></c></row></sheetData></worksheet>"#),
    ]);
    let d = read(&bytes).unwrap();
    assert_eq!(d.content.iter().filter(|b| b.kind == Kind::Table).count(), 1);
    assert!(d.warnings.iter().any(|w| w.contains("repetid")), "{:?}", d.warnings);
}

#[test]
fn powerpoint_diapositivas_repetidas_se_leen_una_vez() {
    let mut ids = String::new();
    for i in 0..10_000 {
        ids.push_str(&format!(r#"<p:sldId id="{}" r:id="rId1"/>"#, 256 + i));
    }
    let bytes = zip(&[
        ("_rels/.rels", &rels(&[("rId1", "officeDocument", "ppt/presentation.xml")])),
        ("ppt/presentation.xml", &format!(r#"<?xml version="1.0"?><p:presentation {}><p:sldIdLst>{}</p:sldIdLst></p:presentation>"#, NS, ids)),
        ("ppt/_rels/presentation.xml.rels", &rels(&[("rId1", "slide", "slides/slide1.xml")])),
        ("ppt/slides/slide1.xml", &diapositiva(&forma("", 0, 0, "<a:p><a:r><a:t>Unica</a:t></a:r></a:p>"), false)),
    ]);
    let d = read(&bytes).unwrap();
    assert_eq!(textos(&d), vec!["Unica"]);
    assert_eq!(d.page_count, Some(1));
    assert!(d.warnings.iter().any(|w| w.contains("repetid")), "{:?}", d.warnings);
}

/// Presentacion de una diapositiva con las formas dadas.
fn presentacion_con(formas: &str) -> Vec<u8> {
    zip(&[
        ("_rels/.rels", &rels(&[("rId1", "officeDocument", "ppt/presentation.xml")])),
        ("ppt/presentation.xml", &format!(r#"<?xml version="1.0"?><p:presentation {}><p:sldIdLst><p:sldId id="256" r:id="rId1"/></p:sldIdLst></p:presentation>"#, NS)),
        ("ppt/_rels/presentation.xml.rels", &rels(&[("rId1", "slide", "slides/slide1.xml")])),
        ("ppt/slides/slide1.xml", &diapositiva(formas, false)),
    ])
}

#[test]
fn powerpoint_numeracion_con_inicio_enorme() {
    for inicio in ["2000000000", "9223372036854775807", "-9223372036854775808", "0"] {
        let p = format!(
            r#"<a:p><a:pPr><a:buAutoNum type="alphaLcPeriod" startAt="{0}"/></a:pPr><a:r><a:t>Uno</a:t></a:r></a:p><a:p><a:pPr><a:buAutoNum type="alphaLcPeriod" startAt="{0}"/></a:pPr><a:r><a:t>Dos</a:t></a:r></a:p>"#,
            inicio
        );
        let d = read(&presentacion_con(&forma("", 0, 0, &p))).unwrap();
        for b in &d.content {
            assert!(b.marker.as_deref().unwrap_or("").len() < 40, "inicio {}: {:?}", inicio, b.marker);
        }
    }
}

#[test]
fn powerpoint_simbolos_solo_en_el_area_privada() {
    // `a:sym` es la fuente de los caracteres de simbolo (U+F0xx); el resto del tramo es texto.
    let p = r#"<a:p><a:r><a:rPr><a:sym typeface="Wingdings"/></a:rPr><a:t>Listo &#xF0FC; revisado</a:t></a:r></a:p>"#;
    let d = read(&presentacion_con(&forma("", 0, 0, p))).unwrap();
    assert_eq!(textos(&d), vec!["Listo \u{2713} revisado"]);
    // Con `a:latin` en Wingdings todo el tramo se dibuja con esa fuente.
    let p = r#"<a:p><a:r><a:rPr><a:latin typeface="Wingdings"/></a:rPr><a:t>&#xFC;</a:t></a:r></a:p>"#;
    let d = read(&presentacion_con(&forma("", 0, 0, p))).unwrap();
    assert_eq!(textos(&d), vec!["\u{2713}"]);
}

#[test]
fn powerpoint_grupos_muy_anidados() {
    let n = 50_000;
    let abre = r#"<p:grpSp><p:nvGrpSpPr><p:cNvPr id="7" name="g"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>"#.repeat(n);
    let formas = format!("{}{}{}", abre, forma("", 0, 0, "<a:p><a:r><a:t>Al fondo</a:t></a:r></a:p>"), "</p:grpSp>".repeat(n));
    let d = read(&presentacion_con(&formas)).unwrap();
    assert_eq!(textos(&d), vec!["Al fondo"]);
}

#[test]
fn parte_principal_declarada_que_no_existe() {
    // Las relaciones apuntan a una presentacion que no esta, pero el paquete es un Word: antes
    // de los lectores de Office se leia como Word y debe seguir asi.
    let documento = r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hola</w:t></w:r></w:p></w:body></w:document>"#;
    let bytes = zip(&[("_rels/.rels", &rels(&[("rId1", "officeDocument", "ppt/presentation.xml")])), ("word/document.xml", documento)]);
    assert_eq!(detect(&bytes).unwrap(), Format::Docx);
    assert_eq!(textos(&read(&bytes).unwrap()), vec!["Hola"]);
}

#[test]
fn excel_filas_y_columnas_fuera_de_rango() {
    let hoja = r#"<sheetData><row r="18446744073709551615"><c r="A18446744073709551615" t="inlineStr"><is><t>x</t></is></c></row><row><c t="inlineStr"><is><t>y</t></is></c></row><row r="4294967295"/><row><c r="ZZZ1" t="inlineStr"><is><t>z</t></is></c></row></sheetData>"#;
    let d = read(&libro_con_hoja(hoja)).unwrap();
    let todo: Vec<&str> = d.content.iter().flat_map(|b| b.rows.iter().flatten().map(|c| c.text.as_str())).collect();
    for v in ["x", "y", "z"] {
        assert!(todo.contains(&v), "falta {} en {:?}", v, todo);
    }
}

#[test]
fn archivos_truncados_no_hacen_caer_la_lectura() {
    for bytes in [presentacion(), planilla(false)] {
        for corte in (0..bytes.len()).step_by(37) {
            let _ = read(&bytes[..corte]);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Topes globales: alias de una misma parte, topes de diapositivas, hojas, celdas y texto, y
// costos que no deben crecer con el cuadrado del archivo.
// ---------------------------------------------------------------------------------------------

/// Presentacion cuya lista de diapositivas cita `destinos` en orden (destinos de relacion,
/// relativos a `ppt/`), con las partes y los alias del ZIP dados.
fn presentacion_que_cita(destinos: &[String], partes: &[(&str, &[u8])], alias: &[(String, &str)]) -> Vec<u8> {
    let ids: String = (1..=destinos.len()).map(|i| format!(r#"<p:sldId id="{}" r:id="rId{}"/>"#, 255 + i, i)).collect();
    let lista: Vec<(String, &str, &str)> = destinos.iter().enumerate().map(|(i, d)| (format!("rId{}", i + 1), "slide", d.as_str())).collect();
    let lista: Vec<(&str, &str, &str)> = lista.iter().map(|(id, t, d)| (id.as_str(), *t, *d)).collect();
    let raiz = rels(&[("rId1", "officeDocument", "ppt/presentation.xml")]);
    let presentacion = format!(r#"<?xml version="1.0"?><p:presentation {}><p:sldIdLst>{}</p:sldIdLst></p:presentation>"#, NS, ids);
    let relaciones = rels(&lista);
    let mut todas: Vec<(&str, &[u8])> = vec![("_rels/.rels", raiz.as_bytes()), ("ppt/presentation.xml", presentacion.as_bytes()), ("ppt/_rels/presentation.xml.rels", relaciones.as_bytes())];
    todas.extend_from_slice(partes);
    zip_con_alias(&todas, alias)
}

/// Libro cuyas hojas H1, H2... citan `destinos` en orden (relativos a `xl/`), con los textos
/// compartidos y los estilos dados (vacios: sin esas partes), las partes y los alias del ZIP.
fn libro_que_cita(destinos: &[String], textos: &str, estilos: &str, partes: &[(&str, &[u8])], alias: &[(String, &str)]) -> Vec<u8> {
    let hojas: String = (1..=destinos.len()).map(|i| format!(r#"<sheet name="H{0}" sheetId="{0}" r:id="rId{0}"/>"#, i)).collect();
    let mut lista: Vec<(String, &str, &str)> = destinos.iter().enumerate().map(|(i, d)| (format!("rId{}", i + 1), "worksheet", d.as_str())).collect();
    lista.push(("rIdTextos".into(), "sharedStrings", "sharedStrings.xml"));
    lista.push(("rIdEstilos".into(), "styles", "styles.xml"));
    let lista: Vec<(&str, &str, &str)> = lista.iter().map(|(id, t, d)| (id.as_str(), *t, *d)).collect();
    let raiz = rels(&[("rId1", "officeDocument", "xl/workbook.xml")]);
    let libro = format!(r#"<?xml version="1.0"?><workbook xmlns:r="{}"><sheets>{}</sheets></workbook>"#, REL, hojas);
    let relaciones = rels(&lista);
    let mut todas: Vec<(&str, &[u8])> = vec![("_rels/.rels", raiz.as_bytes()), ("xl/workbook.xml", libro.as_bytes()), ("xl/_rels/workbook.xml.rels", relaciones.as_bytes())];
    if !textos.is_empty() {
        todas.push(("xl/sharedStrings.xml", textos.as_bytes()));
    }
    if !estilos.is_empty() {
        todas.push(("xl/styles.xml", estilos.as_bytes()));
    }
    todas.extend_from_slice(partes);
    zip_con_alias(&todas, alias)
}

fn tablas(d: &Document) -> Vec<&Block> {
    d.content.iter().filter(|b| b.kind == Kind::Table).collect()
}

/// Rutas que `find` toma por la misma parte (sin distinguir mayusculas ni barras iniciales),
/// escritas como destinos de relacion desde la carpeta `carpeta`.
fn alias_de(carpeta: &str, parte: &str) -> Vec<String> {
    vec![
        parte.to_string(),
        parte.replacen('s', "S", 1),
        format!("/{}/{}", carpeta, parte),
        format!("//{}/{}", carpeta, parte),
        format!("../{}/{}", carpeta, parte).to_uppercase(),
    ]
}

#[test]
fn powerpoint_alias_por_mayusculas_y_barras_se_leen_una_vez() {
    let destinos = alias_de("ppt", "slides/slide1.xml");
    let s1 = diapositiva(&forma("", 0, 0, "<a:p><a:r><a:t>Unica</a:t></a:r></a:p>"), false);
    let d = read(&presentacion_que_cita(&destinos, &[("ppt/slides/slide1.xml", s1.as_bytes())], &[])).unwrap();
    assert_eq!(textos(&d), vec!["Unica"], "destinos {:?}", destinos);
    assert_eq!(d.page_count, Some(1));
    assert!(d.warnings.iter().any(|w| w.contains("4 referencias repetidas")), "{:?}", d.warnings);
}

#[test]
fn excel_alias_por_mayusculas_y_barras_se_leen_una_vez() {
    let destinos = alias_de("xl", "worksheets/sheet1.xml");
    let hoja = r#"<worksheet><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>dato</t></is></c></row></sheetData></worksheet>"#;
    let d = read(&libro_que_cita(&destinos, "", "", &[("xl/worksheets/sheet1.xml", hoja.as_bytes())], &[])).unwrap();
    assert_eq!(tablas(&d).len(), 1, "destinos {:?}", destinos);
    assert_eq!(d.page_count, Some(1));
    assert!(d.warnings.iter().any(|w| w.contains("4 referencias repetidas")), "{:?}", d.warnings);
}

#[test]
fn powerpoint_tope_de_10000_diapositivas() {
    // 10.001 diapositivas distintas (el mismo contenido, copiado en cada parte: los alias de
    // los mismos datos cuentan como una sola).
    let destinos: Vec<String> = (1..=10_001).map(|i| format!("slides/slide{}.xml", i)).collect();
    let nombres: Vec<String> = destinos.iter().map(|d| format!("ppt/{}", d)).collect();
    let s1 = diapositiva(&forma("", 0, 0, "<a:p><a:r><a:t>Texto</a:t></a:r></a:p>"), false);
    let partes: Vec<(&str, &[u8])> = nombres.iter().map(|n| (n.as_str(), s1.as_bytes())).collect();
    let d = read(&presentacion_que_cita(&destinos, &partes, &[])).unwrap();
    assert_eq!(d.page_count, Some(10_000));
    assert_eq!(d.content.len(), 10_000);
    assert_eq!(d.content.last().and_then(|b| b.page), Some(10_000));
    assert!(d.warnings.iter().any(|w| w.contains("mas de 10000 diapositivas")), "{:?}", d.warnings);
}

#[test]
fn excel_tope_de_4096_hojas() {
    let destinos: Vec<String> = (1..=4097).map(|i| format!("worksheets/sheet{}.xml", i)).collect();
    let nombres: Vec<String> = destinos.iter().map(|d| format!("xl/{}", d)).collect();
    let hoja = r#"<worksheet><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>dato</t></is></c></row></sheetData></worksheet>"#;
    let partes: Vec<(&str, &[u8])> = nombres.iter().map(|n| (n.as_str(), hoja.as_bytes())).collect();
    let d = read(&libro_que_cita(&destinos, "", "", &partes, &[])).unwrap();
    assert_eq!(tablas(&d).len(), 4096);
    assert_eq!(tablas(&d).last().and_then(|b| b.page), Some(4096));
    assert!(d.warnings.iter().any(|w| w.contains("mas de 4096 hojas")), "{:?}", d.warnings);
}

#[test]
fn excel_tope_de_celdas_es_del_libro() {
    // Tres hojas con el mismo contenido, de 1.000.001 celdas con valor cada una. Sin referencia,
    // las celdas pasada la ultima columna caen en la misma: la tabla queda chica, pero las
    // celdas leidas cuentan para el tope de 2 millones, que es del libro y no de cada hoja.
    let hoja = format!("<worksheet><sheetData>{}</sheetData></worksheet>", "<c><v>1</v></c>".repeat(1_000_001));
    // La hoja H4 cita una parte que no esta en el ZIP: con el tope agotado se omite sin intentar
    // leerla, asi que tiene su aviso (leida primero, se habria saltado sin aviso).
    let destinos: Vec<String> = (1..=4).map(|i| format!("worksheets/sheet{}.xml", i)).collect();
    let nombres: Vec<String> = destinos[..3].iter().map(|d| format!("xl/{}", d)).collect();
    let partes: Vec<(&str, &[u8])> = nombres.iter().map(|n| (n.as_str(), hoja.as_bytes())).collect();
    let d = read(&libro_que_cita(&destinos, "", "", &partes, &[])).unwrap();
    let avisos = |hoja: &str| d.warnings.iter().filter(|w| w.starts_with(&format!("Hoja «{}»", hoja))).cloned().collect::<Vec<_>>();
    assert!(avisos("H1").is_empty(), "{:?}", d.warnings);
    assert!(avisos("H2").iter().any(|w| w.contains("solo una parte") && w.contains("2000000 celdas")), "{:?}", d.warnings);
    assert!(avisos("H3").iter().any(|w| w.contains("se omitio") && w.contains("2000000 celdas")), "{:?}", d.warnings);
    assert!(avisos("H4").iter().any(|w| w.contains("se omitio") && w.contains("2000000 celdas")), "{:?}", d.warnings);
    assert_eq!(tablas(&d).len(), 2);
}

/// Lee con el limite de caracteres de la lectura (50 millones por defecto) por encima del tope de
/// texto propio de los lectores de PowerPoint y Excel (64 MB), para probar ese tope.
fn sin_limite_de_caracteres(bytes: &[u8]) -> Document {
    doc2json::read_with_limits(bytes, doc2json::budget::Limits { max_chars: 1 << 30, ..Default::default() }).unwrap()
}

#[test]
fn excel_tope_de_texto_con_un_texto_compartido() {
    // Un texto de 64 KB citado por 5.000 celdas serian 320 MB: el texto del libro tiene un tope
    // de 64 MB.
    let largo = 64 * 1024;
    let textos = format!("<sst><si><t>{}</t></si></sst>", "x".repeat(largo));
    let fila: String = (1..=5000).map(|c| format!(r#"<c r="{}1" t="s"><v>0</v></c>"#, columna(c))).collect();
    let hoja = format!(r#"<worksheet><sheetData><row r="1">{}</row></sheetData></worksheet>"#, fila);
    let destinos = vec!["worksheets/sheet1.xml".to_string()];
    let d = sin_limite_de_caracteres(&libro_que_cita(&destinos, &textos, "", &[("xl/worksheets/sheet1.xml", hoja.as_bytes())], &[]));
    let tabla = tablas(&d)[0];
    let total: usize = tabla.rows.iter().flatten().map(|c| c.text.len()).sum();
    assert!(total <= 64 << 20, "texto: {} bytes", total);
    assert!(total > (64 << 20) - largo, "se leyo de menos: {} bytes", total);
    assert_eq!(tabla.rows[0][0].text.len(), largo);
    assert!(d.warnings.iter().any(|w| w.contains("MB de texto")), "{:?}", d.warnings);
}

#[test]
fn excel_texto_compartido_en_blanco_no_se_recorre_en_cada_celda() {
    // Un texto de 1 MB de espacios citado por 50.000 celdas: copiarlo o recorrerlo en cada celda
    // serian 50 GB de trabajo para celdas que quedan vacias.
    let textos = format!("<sst><si><t>{}</t></si><si><t>dato</t></si></sst>", " ".repeat(1 << 20));
    let filas: String = (1..=50_000).map(|r| format!(r#"<row r="{0}"><c r="A{0}" t="s"><v>0</v></c></row>"#, r)).collect();
    let hoja = format!(r#"<worksheet><sheetData>{}<row r="50001"><c r="B50001" t="s"><v>1</v></c></row></sheetData></worksheet>"#, filas);
    let destinos = vec!["worksheets/sheet1.xml".to_string()];
    let inicio = std::time::Instant::now();
    let d = read(&libro_que_cita(&destinos, &textos, "", &[("xl/worksheets/sheet1.xml", hoja.as_bytes())], &[])).unwrap();
    assert!(inicio.elapsed().as_secs() < 20, "tardo {:?}", inicio.elapsed());
    let celdas: Vec<&str> = tablas(&d)[0].rows.iter().flatten().map(|c| c.text.as_str()).collect();
    assert_eq!(celdas, ["dato"]);
}

#[test]
fn powerpoint_tope_de_texto_total() {
    // 70 diapositivas de 1 MB de texto cada una: sin el tope de 64 MB, 70 MB.
    let n = 70;
    let s1 = diapositiva(&forma("", 0, 0, &format!("<a:p><a:r><a:t>{}</a:t></a:r></a:p>", "x".repeat(1 << 20))), false);
    let destinos: Vec<String> = (1..=n).map(|i| format!("slides/slide{}.xml", i)).collect();
    let nombres: Vec<String> = destinos.iter().map(|d| format!("ppt/{}", d)).collect();
    let partes: Vec<(&str, &[u8])> = nombres.iter().map(|n| (n.as_str(), s1.as_bytes())).collect();
    let d = sin_limite_de_caracteres(&presentacion_que_cita(&destinos, &partes, &[]));
    let total: usize = d.content.iter().map(|b| b.text.len()).sum();
    assert!(total <= 64 << 20, "texto: {} bytes", total);
    assert_eq!(d.content.len(), 64);
    assert!(d.warnings.iter().any(|w| w.contains("MB de texto")), "{:?}", d.warnings);
}

#[test]
fn excel_estilos_con_muchos_formatos_no_son_cuadraticos() {
    // 100.000 formatos propios y 100.000 estilos que no los usan: buscar cada estilo en la lista
    // de formatos serian 10^10 comparaciones.
    let n = 100_000;
    let mut estilos = String::from("<styleSheet><numFmts>");
    for i in 0..n {
        estilos.push_str(&format!(r#"<numFmt numFmtId="{}" formatCode="0.00"/>"#, 1000 + i));
    }
    // Un formato repetido: vale el ultimo, como antes.
    estilos.push_str(r#"<numFmt numFmtId="164" formatCode="dd/mm/yyyy"/><numFmt numFmtId="164" formatCode="0%"/></numFmts><cellXfs>"#);
    estilos.push_str(&r#"<xf numFmtId="0"/>"#.repeat(n));
    estilos.push_str(r#"<xf numFmtId="164"/><xf numFmtId="14"/></cellXfs></styleSheet>"#);
    let hoja = format!(r#"<worksheet><sheetData><row r="1"><c r="A1"><v>0.5</v></c><c r="B1" s="{}"><v>0.5</v></c><c r="C1" s="{}"><v>45000</v></c></row></sheetData></worksheet>"#, n, n + 1);
    let destinos = vec!["worksheets/sheet1.xml".to_string()];
    let inicio = std::time::Instant::now();
    let d = read(&libro_que_cita(&destinos, "", &estilos, &[("xl/worksheets/sheet1.xml", hoja.as_bytes())], &[])).unwrap();
    assert!(inicio.elapsed().as_secs() < 20, "tardo {:?}", inicio.elapsed());
    let fila: Vec<&str> = tablas(&d)[0].rows[0].iter().map(|c| c.text.as_str()).collect();
    assert_eq!(fila, ["0.5", "50%", "2023-03-15"]);
}

#[test]
fn powerpoint_sin_lista_de_diapositivas_con_muchas_entradas() {
    // Sin `p:sldIdLst` las diapositivas se buscan por numero hasta el numero de entradas del ZIP.
    // Con 65.000 entradas, buscar cada numero recorriendo la lista serian 4 * 10^9 comparaciones.
    let presentacion = format!(r#"<?xml version="1.0"?><p:presentation {}/>"#, NS);
    let raiz = rels(&[("rId1", "officeDocument", "ppt/presentation.xml")]);
    let s1 = diapositiva(&forma("", 0, 0, "<a:p><a:r><a:t>Uno</a:t></a:r></a:p>"), false);
    let s3 = diapositiva(&forma("", 0, 0, "<a:p><a:r><a:t>Tres</a:t></a:r></a:p>"), false);
    let s9 = diapositiva(&forma("", 0, 0, "<a:p><a:r><a:t>Nueve</a:t></a:r></a:p>"), false);
    let partes: Vec<(&str, &[u8])> = vec![
        ("_rels/.rels", raiz.as_bytes()),
        ("ppt/presentation.xml", presentacion.as_bytes()),
        ("ppt/slides/slide1.xml", s1.as_bytes()),
        // Con otras mayusculas: `find` la encuentra, igual que antes.
        ("PPT/Slides/Slide3.xml", s3.as_bytes()),
        // Con un cero a la izquierda no es la diapositiva 9.
        ("ppt/slides/slide09.xml", s9.as_bytes()),
        ("ppt/media/vacio.bin", b""),
    ];
    // Relleno con el largo de "ppt/slides/slide12345.xml", para que cada comparacion cueste.
    let alias: Vec<(String, &str)> = (0..64_990).map(|i| (format!("ppt/slides/x{:09}.xml", i), "ppt/media/vacio.bin")).collect();
    let inicio = std::time::Instant::now();
    let d = read(&zip_con_alias(&partes, &alias)).unwrap();
    assert!(inicio.elapsed().as_secs() < 20, "tardo {:?}", inicio.elapsed());
    assert_eq!(textos(&d), vec!["Uno", "Tres"]);
    assert_eq!(d.page_count, Some(2));
}

#[test]
fn partes_xml_truncadas_no_hacen_caer_la_lectura() {
    // El ZIP sigue siendo valido y lo que se corta es el XML de una de sus partes, en cada
    // posicion: asi se ejercitan los lectores de relaciones, presentacion, diapositivas, notas,
    // libro, textos compartidos, estilos y hojas, no solo el error de ZIP danado.
    for (partes, tipo) in [(partes_de_presentacion(), "pptx"), (partes_de_planilla(false), "xlsx")] {
        for k in 0..partes.len() {
            for corte in (0..partes[k].1.len()).step_by(3) {
                let cortadas: Vec<(&str, &[u8])> =
                    partes.iter().enumerate().map(|(i, (n, c))| (*n, if i == k { &c.as_bytes()[..corte] } else { c.as_bytes() })).collect();
                let bytes = zip_con_alias(&cortadas, &[]);
                match read(&bytes) {
                    Ok(d) => assert_eq!(d.kind, tipo, "{} cortada en {}", partes[k].0, corte),
                    Err(e) => panic!("{} cortada en {}: {}", partes[k].0, corte, e),
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Copias antes de los topes y alias de los mismos datos.
// ---------------------------------------------------------------------------------------------

#[test]
fn relacion_con_destino_enorme_se_descarta() {
    // Un destino de 3 MB citado por 150 hojas: copiarlo en cada referencia serian 450 MB antes de
    // deduplicar. Se descarta al leer las relaciones, y esas hojas quedan sin parte; la hoja con
    // un destino normal se lee igual.
    let enorme = "x".repeat(3 << 20);
    let mut hojas: String = (1..=150).map(|i| format!(r#"<sheet name="H{0}" sheetId="{0}" r:id="rId1"/>"#, i)).collect();
    hojas.push_str(r#"<sheet name="Datos" sheetId="151" r:id="rId2"/>"#);
    let libro = format!(r#"<?xml version="1.0"?><workbook xmlns:r="{}"><sheets>{}</sheets></workbook>"#, REL, hojas);
    let hoja = r#"<worksheet><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>dato</t></is></c></row></sheetData></worksheet>"#;
    let bytes = zip(&[
        ("_rels/.rels", &rels(&[("rId1", "officeDocument", "xl/workbook.xml")])),
        ("xl/workbook.xml", &libro),
        ("xl/_rels/workbook.xml.rels", &rels(&[("rId1", "worksheet", &enorme), ("rId2", "worksheet", "worksheets/sheet1.xml")])),
        ("xl/worksheets/sheet1.xml", hoja),
    ]);
    let d = read(&bytes).unwrap();
    // Con el destino conservado, las 150 hojas citarian la misma ruta de 3 MB: 149 repetidas.
    assert!(!d.warnings.iter().any(|w| w.contains("repetid")), "{:?}", d.warnings);
    assert_eq!(tablas(&d).len(), 1);
    assert_eq!(tablas(&d)[0].rows[0][0].text, "dato");
}

#[test]
fn alias_de_los_mismos_datos_se_leen_una_vez() {
    // 2.000 nombres distintos en el ZIP que apuntan a los datos de una misma parte: una hoja
    // grande de celdas vacias (que no gastan ningun tope) y una diapositiva. Cada una se lee una
    // vez; las demas son referencias repetidas.
    let n = 2000;
    let hoja = format!("<worksheet><sheetData><row r=\"1\">{}</row></sheetData></worksheet>", "<c/>".repeat(25_000));
    let destinos: Vec<String> = (1..=n).map(|i| format!("worksheets/sheet{}.xml", i)).collect();
    let alias: Vec<(String, &str)> = (2..=n).map(|i| (format!("xl/worksheets/sheet{}.xml", i), "xl/worksheets/sheet1.xml")).collect();
    let d = read(&libro_que_cita(&destinos, "", "", &[("xl/worksheets/sheet1.xml", hoja.as_bytes())], &alias)).unwrap();
    assert_eq!(d.page_count, Some(1));
    assert!(d.warnings.iter().any(|w| w.contains(&format!("{} referencias repetidas a hojas", n - 1))), "{:?}", d.warnings);

    let s1 = diapositiva(&forma("", 0, 0, "<a:p><a:r><a:t>Unica</a:t></a:r></a:p>"), false);
    let destinos: Vec<String> = (1..=n).map(|i| format!("slides/slide{}.xml", i)).collect();
    let alias: Vec<(String, &str)> = (2..=n).map(|i| (format!("ppt/slides/slide{}.xml", i), "ppt/slides/slide1.xml")).collect();
    let d = read(&presentacion_que_cita(&destinos, &[("ppt/slides/slide1.xml", s1.as_bytes())], &alias)).unwrap();
    assert_eq!(textos(&d), vec!["Unica"]);
    assert_eq!(d.page_count, Some(1));
    assert!(d.warnings.iter().any(|w| w.contains(&format!("{} referencias repetidas a diapositivas", n - 1))), "{:?}", d.warnings);
}
