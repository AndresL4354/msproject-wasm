//! Markdown y fragmentos con entradas hostiles o con casos limite: celdas que dicen ocupar millones
//! de columnas, HTML y enlaces peligrosos en el texto, tildes y emojis en los cortes.

use doc2json::doc::Kind;
use std::time::Instant;

/// ZIP sin compresion (metodo 0) con varias partes de texto.
fn zip(partes: &[(&str, &str)]) -> Vec<u8> {
    let le16 = |v: usize| (v as u16).to_le_bytes();
    let le32 = |v: usize| (v as u32).to_le_bytes();
    let (mut out, mut central) = (Vec::new(), Vec::new());
    for (nombre, contenido) in partes {
        let (n, c) = (nombre.as_bytes(), contenido.as_bytes());
        let offset = out.len();
        out.extend_from_slice(b"PK\x03\x04\x14\0\0\0\0\0\0\0\0\0\0\0\0\0");
        out.extend_from_slice(&le32(c.len()));
        out.extend_from_slice(&le32(c.len()));
        out.extend_from_slice(&le16(n.len()));
        out.extend_from_slice(&le16(0));
        out.extend_from_slice(n);
        out.extend_from_slice(c);
        central.extend_from_slice(b"PK\x01\x02\x14\0\x14\0\0\0\0\0\0\0\0\0\0\0\0\0");
        central.extend_from_slice(&le32(c.len()));
        central.extend_from_slice(&le32(c.len()));
        central.extend_from_slice(&le16(n.len()));
        central.extend_from_slice(&[0; 12]);
        central.extend_from_slice(&le32(offset));
        central.extend_from_slice(n);
    }
    let inicio = out.len();
    out.extend_from_slice(&central);
    out.extend_from_slice(b"PK\x05\x06\0\0\0\0");
    out.extend_from_slice(&le16(partes.len()));
    out.extend_from_slice(&le16(partes.len()));
    out.extend_from_slice(&le32(central.len()));
    out.extend_from_slice(&le32(inicio));
    out.extend_from_slice(&le16(0));
    out
}

const W: &str = r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships""#;

/// Un .docx minimo con ese cuerpo y, si hace falta, relaciones del documento.
fn docx(cuerpo: &str, rels: &str) -> Vec<u8> {
    docx_con_notas(cuerpo, rels, "")
}

/// Como `docx`, con notas al pie (`word/footnotes.xml`) si `notas` no esta vacio.
fn docx_con_notas(cuerpo: &str, rels: &str, notas: &str) -> Vec<u8> {
    let documento = format!(r#"<?xml version="1.0"?><w:document {}><w:body>{}</w:body></w:document>"#, W, cuerpo);
    let rels_doc = format!(r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{}</Relationships>"#, rels);
    let notas_xml = format!(r#"<?xml version="1.0"?><w:footnotes {}>{}</w:footnotes>"#, W, notas);
    let mut partes = vec![
        ("_rels/.rels", r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#),
        ("word/document.xml", documento.as_str()),
        ("word/_rels/document.xml.rels", rels_doc.as_str()),
    ];
    if !notas.is_empty() {
        partes.push(("word/footnotes.xml", notas_xml.as_str()));
    }
    zip(&partes)
}

fn parrafo(texto: &str) -> String {
    format!(r#"<w:p><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#, texto)
}

fn celda(texto: &str, span: &str) -> String {
    let pr = if span.is_empty() { String::new() } else { format!(r#"<w:tcPr><w:gridSpan w:val="{}"/></w:tcPr>"#, span) };
    format!("<w:tc>{}{}</w:tc>", pr, parrafo(texto))
}

#[test]
fn grid_span_enorme_se_acota_al_leer_y_al_armar_la_tabla() {
    let filas = [
        format!("<w:tr>{}</w:tr>", celda("Encabezado", "100000000")),
        format!("<w:tr>{}{}</w:tr>", celda("b", ""), celda("c", "4294967295")),
        format!("<w:tr>{}{}</w:tr>", celda("x", "4294967295"), celda("y", "-3")),
    ];
    let bytes = docx(&format!("<w:tbl>{}</w:tbl>", filas.concat()), "");
    assert!(bytes.len() < 2048);
    let d = doc2json::read(&bytes).unwrap();
    let t = d.content.iter().find(|b| b.kind == Kind::Table).unwrap();
    assert!(t.rows.iter().flatten().all(|c| (1..=doc2json::doc::MAX_COLSPAN).contains(&c.colspan)), "{:?}", t.rows);
    let inicio = Instant::now();
    let md = doc2json::markdown::to_markdown(&d);
    assert!(md.len() < 20_000, "{} bytes", md.len());
    for t in ["Encabezado", "| b |", "c", "x", "y"] {
        assert!(md.contains(t), "{}: {}", t, md);
    }
    let c = doc2json::markdown::chunks(&d, 300);
    assert!(c.iter().map(|x| x.text.len()).sum::<usize>() < 20_000);
    assert!(inicio.elapsed().as_secs() < 20);
}

#[test]
fn una_fila_con_miles_de_celdas_no_multiplica_las_demas() {
    // 3.000 celdas reales en una fila y 2.000 filas de una celda: sin tope serian 6 millones.
    let ancha: String = (0..3000).map(|i| celda(&format!("c{}", i), "")).collect();
    let mut filas = format!("<w:tr>{}</w:tr>", ancha);
    for i in 0..2000 {
        filas.push_str(&format!("<w:tr>{}</w:tr>", celda(&format!("f{}", i), "")));
    }
    let d = doc2json::read(&docx(&format!("<w:tbl>{}</w:tbl>", filas), "")).unwrap();
    let md = doc2json::markdown::to_markdown(&d);
    assert!(md.len() < 2_000_000, "{} bytes", md.len());
    // No se pierde texto: las celdas que no caben en el ancho se juntan en la ultima columna.
    assert!(md.contains("c0 |") && md.contains("c2999") && md.contains("f1999"));
}

#[test]
fn el_texto_no_trae_html_ni_enlaces_peligrosos() {
    let rels = r#"<Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="javascript:alert(1)" TargetMode="External"/><Relationship Id="rId8" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="HTTPS://ejemplo.cl/a b(c)" TargetMode="External"/><Relationship Id="rId7" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="httpx://malo" TargetMode="External"/>"#;
    let enlace = |id: &str, t: &str| format!(r#"<w:hyperlink r:id="{}"><w:r><w:rPr><w:b/></w:rPr><w:t>{}</w:t></w:r></w:hyperlink>"#, id, t);
    let cuerpo = [
        parrafo("&lt;img src=x onerror=alert(1)&gt; y &lt;script&gt;"),
        parrafo("[clic](javascript:alert(1)) y ![x](javascript:alert(2))"),
        parrafo("[1]: javascript:alert(3)"),
        parrafo("Entidad &amp;lt;b&amp;gt; y barra \\&lt;i&gt;"),
        format!("<w:p>{}{}{}</w:p>", enlace("rId9", "uno"), enlace("rId8", "dos"), enlace("rId7", "tres")),
        r#"<w:p><w:r><w:t>linea</w:t></w:r><w:r><w:br/><w:t># no titulo</w:t></w:r><w:r><w:br/><w:t>---</w:t></w:r></w:p>"#.to_string(),
        format!("<w:tbl><w:tr>{}{}</w:tr><w:tr>{}{}</w:tr></w:tbl>", celda("A", ""), celda("B", ""), celda("&lt;b onclick=x&gt;", ""), celda("[a](javascript:x)", "")),
    ]
    .concat();
    let md = doc2json::to_markdown(&docx(&cuerpo, rels)).unwrap();
    // Todo "<" y todo "[" del texto van escapados (con un numero impar de barras antes); los unicos
    // sin escapar son los que arma el conversor: "<br>" en las celdas y los enlaces permitidos.
    let sin_escapar = |c: char| {
        md.match_indices(c)
            .filter(|(i, _)| md[..*i].chars().rev().take_while(|x| *x == '\\').count() % 2 == 0)
            .map(|(i, _)| md[i..].chars().take(12).collect::<String>())
            .collect::<Vec<_>>()
    };
    assert!(sin_escapar('<').iter().all(|s| s.starts_with("<br>")), "{:?}\n{}", sin_escapar('<'), md);
    assert_eq!(sin_escapar('[').iter().map(|s| s.as_str()).collect::<Vec<_>>(), ["[**dos**](HT"], "{}", md);
    assert!(md.contains("\\<img src=x onerror=alert(1)> y \\<script>"), "{}", md);
    assert!(md.contains("\\[clic\\](javascript:alert(1)) y !\\[x\\](javascript:alert(2))"), "{}", md);
    assert!(md.contains("\\\\\\<i>"), "{}", md);
    assert!(md.contains("\\[1\\]: javascript:alert(3)"), "{}", md);
    // Una entidad escrita en el texto no se convierte en el caracter.
    assert!(md.contains("\\&lt;b\\&gt;"), "{}", md);
    // Los enlaces: solo http(s) y mailto, codificados; el resto queda como texto.
    assert!(md.contains("**uno**") && !md.contains("[**uno**]"), "{}", md);
    assert!(md.contains("[**dos**](HTTPS://ejemplo.cl/a%20b%28c%29)"), "{}", md);
    assert!(md.contains("**tres**") && !md.contains("httpx"), "{}", md);
    // Un salto de linea dentro del parrafo no abre un titulo ni una linea horizontal.
    assert!(md.contains("\\# no titulo") && md.contains("\\---"), "{}", md);
}

#[test]
fn el_formato_se_conserva_con_espacios_al_final() {
    let cuerpo = r#"<w:p><w:r><w:rPr><w:b/></w:rPr><w:t xml:space="preserve">Importante: </w:t></w:r><w:r><w:t xml:space="preserve">use casco.  </w:t></w:r></w:p><w:p><w:r><w:t xml:space="preserve">  </w:t></w:r><w:r><w:rPr><w:i/></w:rPr><w:t xml:space="preserve">cursiva </w:t></w:r></w:p>"#;
    let md = doc2json::to_markdown(&docx(cuerpo, "")).unwrap();
    assert_eq!(md, "**Importante:** use casco.\n\n*cursiva*\n");
}

#[test]
fn tildes_enes_y_emojis_en_los_cortes() {
    let frase = "Señalización añadida: ¿Está el área bloqueada? Sí 👷‍♀️🦺. ";
    let largo = frase.repeat(80);
    let cuerpo = [parrafo("Título"), parrafo(&largo), parrafo(&"ñ".repeat(900)), parrafo(&"🦺".repeat(700))].concat();
    let d = doc2json::read(&docx(&cuerpo, "")).unwrap();
    for max in [200, 333, 1000] {
        let c = doc2json::markdown::chunks(&d, max);
        assert!(c.iter().all(|x| x.text.chars().count() <= max), "{}: {:?}", max, c.iter().map(|x| x.text.chars().count()).collect::<Vec<_>>());
        let todo: String = c.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(todo.matches("Señalización").count(), 80);
        assert_eq!(todo.matches('ñ').count(), 80 * 2 + 900);
        assert_eq!(todo.matches('🦺').count(), 80 + 700);
    }
}

#[test]
fn archivos_cortados_no_provocan_panic() {
    let cuerpo = [parrafo("Objetivo"), format!("<w:tbl><w:tr>{}{}</w:tr></w:tbl>", celda("a", "2"), celda("b", ""))].concat();
    let bytes = docx(&cuerpo, "");
    for corte in 0..bytes.len() {
        if let Ok(d) = doc2json::read(&bytes[..corte]) {
            let _ = doc2json::markdown::to_markdown(&d);
            let _ = doc2json::markdown::chunks(&d, 200);
        }
        let _ = doc2json::to_chunks_json(&bytes[..corte], 0);
    }
}

/// Posiciones de `c` que no van escapadas (sin un numero impar de barras antes).
fn sin_escapar(md: &str, c: char) -> Vec<String> {
    md.match_indices(c)
        .filter(|(i, _)| md[..*i].chars().rev().take_while(|x| *x == '\\').count() % 2 == 0)
        .map(|(i, _)| md[i..].chars().take(8).collect())
        .collect()
}

#[test]
fn los_cortes_no_dejan_marcas_sin_escapar() {
    // Sin espacios el corte cae en cualquier caracter, tambien entre una barra y lo que escapa.
    let pegado = "&lt;img&gt;[x]".repeat(200);
    let frases = "texto # titulo 1. lista - item &gt; cita ~~~ ".repeat(40);
    // Una celda con saltos de linea: en la tabla van como "<br>", que un corte no debe partir.
    let multilinea = "<w:r><w:t>linea con &lt;b&gt;</w:t><w:br/></w:r>".repeat(60);
    let tabla = format!("<w:tbl><w:tr>{}{}</w:tr><w:tr>{}<w:tc><w:p>{}</w:p></w:tc></w:tr></w:tbl>", celda("Paso", ""), celda("Detalle", ""), celda("1", ""), multilinea);
    // Todo corte en un espacio deja al inicio "#", "-" o "1.".
    let marcas = "# a - b 1) c ".repeat(80);
    let cuerpo = [parrafo("Inicio"), parrafo(&pegado), parrafo(&frases), parrafo(&marcas), tabla].concat();
    let d = doc2json::read(&docx(&cuerpo, "")).unwrap();
    for max in 200..260 {
        for c in doc2json::markdown::chunks(&d, max) {
            assert!(c.text.chars().count() <= max, "{}: {}", max, c.text.chars().count());
            assert!(sin_escapar(&c.text, '<').iter().all(|s| s.starts_with("<br>")), "{}: {:?}\n{}", max, sin_escapar(&c.text, '<'), c.text);
            assert!(sin_escapar(&c.text, '[').is_empty(), "{}: {}", max, c.text);
            for linea in c.text.lines() {
                assert!(!linea.starts_with('#') || linea.starts_with("# Inicio"), "{}: {:?}", max, linea);
                assert!(!linea.starts_with("1)") && !linea.starts_with("- ") && !linea.starts_with("> ") && !linea.starts_with("~~~"), "{}: {:?}", max, linea);
            }
        }
    }
}

/// Palabras de un texto sin las marcas de Markdown (barras, barras invertidas, guiones de tabla).
fn palabras(s: &str) -> Vec<String> {
    let mut v: Vec<String> = s.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(str::to_string).collect();
    v.sort();
    v
}

#[test]
fn una_tabla_muy_ancha_respeta_el_tamano_de_los_fragmentos() {
    // 64 columnas con texto en todas (como una planilla ancha): ni una fila sola cabe con el
    // encabezado en 2.000 caracteres. Antes salian fragmentos de mas; ahora ninguno pasa el tope
    // y no se pierde ninguna palabra, tampoco con fragmentos chicos.
    let fila = |p: &str| format!("<w:tr>{}</w:tr>", (0..64).map(|i| celda(&format!("{}{} con algo de texto en la celda", p, i), "")).collect::<String>());
    let filas: String = std::iter::once(fila("titulo")).chain((0..3).map(|r| fila(&format!("dato{}x", r)))).collect();
    let d = doc2json::read(&docx(&format!("<w:tbl>{}</w:tbl>", filas), "")).unwrap();
    let todo = palabras(&doc2json::markdown::to_markdown(&d));
    // 200 es el minimo de un fragmento (MIN_CHUNK).
    for max in [2000, 500, 200] {
        let c = doc2json::markdown::chunks(&d, max);
        assert!(c.iter().all(|x| x.text.chars().count() <= max), "max {}: {:?}", max, c.iter().map(|x| x.text.chars().count()).max());
        let juntas = palabras(&c.iter().map(|x| x.text.as_str()).collect::<Vec<_>>().join("\n"));
        for w in &todo {
            assert!(juntas.binary_search(w).is_ok(), "max {}: falta {}", max, w);
        }
    }
}

/// Nota al pie con id 1 y un parrafo que la cita: con ella, `[1]` en el texto pasa a `[^1]`.
const NOTA: &str = r#"<w:footnote w:id="1"><w:p><w:r><w:t>Solo equipos electricos.</w:t></w:r></w:p></w:footnote>"#;
const CITA: &str = r#"<w:p><w:r><w:t>Texto con nota</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#;

#[test]
fn una_cita_de_nota_no_abre_un_enlace_ni_una_definicion() {
    // "[^1](javascript:...)" es un enlace para un lector sin notas al pie, y "[^1]: ..." al inicio
    // de una linea es una definicion que reemplazaria la nota (o el destino de sus citas).
    // Tambien con la cita y el "(" en tramos distintos: el subrayado los separa sin dejar marca.
    let tramos = r#"<w:p><w:r><w:rPr><w:u w:val="single"/></w:rPr><w:t>Otra</w:t><w:footnoteReference w:id="1"/></w:r><w:r><w:t>(javascript:alert(3))</w:t></w:r><w:r><w:rPr><w:b/></w:rPr><w:t xml:space="preserve"> fin</w:t></w:r></w:p>"#;
    let cuerpo = [CITA, &parrafo("[1](javascript:alert(1)) y [1]:x"), &parrafo("[1]: javascript:alert(2)"), tramos].concat();
    let bytes = docx_con_notas(&cuerpo, "", NOTA);
    let md = doc2json::to_markdown(&bytes).unwrap();
    assert!(md.contains("Texto con nota[^1]"), "{}", md);
    assert!(!md.contains("[^1]("), "{}", md);
    assert!(md.contains("[^1]\\(javascript:alert(1)) y [^1]\\:x"), "{}", md);
    assert!(md.contains("Otra[^1]\\(javascript:alert(3)) **fin**"), "{}", md);
    // La unica linea "[^1]: ..." es la nota que arma el conversor al final, que para un lector sin
    // notas al pie tampoco es una definicion (ver `una_nota_no_es_una_definicion_de_referencia`).
    let notas: Vec<&str> = md.lines().filter(|l| l.starts_with("[^1]:")).collect();
    assert_eq!(notas, ["[^1]: Solo equipos electricos."], "{}", md);
    let d = doc2json::read(&bytes).unwrap();
    for c in doc2json::markdown::chunks(&d, 200) {
        assert!(!c.text.contains("[^1]("), "{}", c.text);
        assert!(c.path == ["Notas"] || !c.text.lines().any(|l| l.starts_with("[^1]:")), "{}", c.text);
    }
}

#[test]
fn muchos_corchetes_sin_cerrar_con_una_nota_es_lineal() {
    // Cada "[" busca su "]" solo en el largo que puede tener una etiqueta: antes recorria todo el
    // resto del texto (un millon de corchetes: ~5 * 10^11 comparaciones).
    let cuerpo = [CITA, &parrafo(&"[".repeat(1_000_000))].concat();
    let d = doc2json::read(&docx_con_notas(&cuerpo, "", NOTA)).unwrap();
    let inicio = Instant::now();
    let md = doc2json::markdown::to_markdown(&d);
    assert!(inicio.elapsed().as_secs() < 20, "{:?}", inicio.elapsed());
    assert_eq!(md.matches("\\[").count(), 1_000_000);
}

#[test]
fn un_numero_al_inicio_se_escapa_sin_dejar_la_barra_visible() {
    // En CommonMark "\1" no es un escape (la barra se ve): se escapa el punto o el parentesis.
    let cuerpo = [parrafo("1. Objetivo"), parrafo("2) Alcance"), parrafo("10. Fin"), parrafo("2024. Un buen anio"), parrafo("3.2 Responsables")].concat();
    let md = doc2json::to_markdown(&docx(&cuerpo, "")).unwrap();
    assert_eq!(md, "1\\. Objetivo\n\n2\\) Alcance\n\n10\\. Fin\n\n2024\\. Un buen anio\n\n3.2 Responsables\n");
}

#[test]
fn numeracion_con_tabulador_y_doble_guion_no_abren_marcas() {
    // Word numera a mano con un tabulador ("1.<tab>Objetivo"): "1." y "1)" seguidos de tabulador
    // abren una lista igual que con espacio. "--" bajo una linea de parrafo la vuelve titulo.
    let tab = |n: &str| format!("<w:p><w:r><w:t>{}</w:t><w:tab/><w:t>Objetivo</w:t></w:r></w:p>", n);
    let cuerpo = [tab("1."), tab("2)"), "<w:p><w:r><w:t>Aviso</w:t><w:br/><w:t>--</w:t></w:r></w:p>".to_string()].concat();
    let md = doc2json::to_markdown(&docx(&cuerpo, "")).unwrap();
    assert_eq!(md, "1\\.\tObjetivo\n\n2\\)\tObjetivo\n\nAviso\n\\--\n");
}

#[test]
fn un_retorno_de_carro_suelto_no_abre_marcas_ni_rompe_la_tabla() {
    // CommonMark tambien termina una linea en un "\r" solo (sin "\n").
    let tabla = format!("<w:tbl><w:tr>{}{}</w:tr><w:tr>{}{}</w:tr></w:tbl>", celda("A", ""), celda("B", ""), celda("x&#13;# no titulo", ""), celda("y&#13;&#13;z", ""));
    let recuadro = format!("<w:tbl><w:tr>{}</w:tr></w:tbl>", celda("OBJETIVO&#13;# NO TITULO", ""));
    let cuerpo = [parrafo("linea&#13;# no titulo&#13;- no item&#13;1. no lista"), tabla, recuadro].concat();
    let bytes = docx(&cuerpo, "");
    let md = doc2json::to_markdown(&bytes).unwrap();
    assert!(!md.contains('\r'), "{:?}", md);
    for linea in md.split(['\n', '\r']) {
        assert!(!linea.starts_with('#') && !linea.starts_with("- ") && !linea.starts_with("1. "), "{:?}\n{:?}", linea, md);
    }
    assert!(md.contains("linea\n\\# no titulo\n\\- no item\n1\\. no lista"), "{:?}", md);
    // Dentro de una celda "#" no abre nada: basta con que la fila siga en una sola linea.
    assert!(md.contains("| x<br># no titulo | y<br>z |"), "{:?}", md);
    assert!(md.contains("| OBJETIVO<br># NO TITULO |"), "{:?}", md);
    let d = doc2json::read(&bytes).unwrap();
    for c in doc2json::markdown::chunks(&d, 200) {
        assert!(!c.text.contains('\r') && c.path.is_empty(), "{:?}", c);
    }
}

fn titulo(texto: &str) -> String {
    format!(r#"<w:p><w:pPr><w:outlineLvl w:val="0"/></w:pPr><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#, texto)
}

#[test]
fn un_titulo_largo_no_pasa_el_tamano_del_fragmento() {
    // Un titulo que casi llena el fragmento despues de una seccion corta (se juntaban y pasaban el
    // maximo) y otro mas largo que el fragmento mismo (salia entero).
    let corto = "Texto corto. ".repeat(10);
    let medio = "Otro titulo que casi llena el fragmento ".repeat(12);
    let largo = "Titulo extenso de la seccion ".repeat(40);
    let cuerpo = [parrafo(&corto), titulo(&medio), parrafo("Mas texto."), titulo(&largo), parrafo("Texto de la seccion.")].concat();
    let d = doc2json::read(&docx(&cuerpo, "")).unwrap();
    for max in [200, 300, 500, 600] {
        let c = doc2json::markdown::chunks(&d, max);
        assert!(c.iter().all(|x| x.text.chars().count() <= max), "{}: {:?}", max, c.iter().map(|x| x.text.chars().count()).collect::<Vec<_>>());
        let todo = c.iter().map(|x| x.text.as_str()).collect::<Vec<_>>().join(" ");
        assert_eq!(todo.matches("Texto corto").count(), 10);
        assert_eq!(todo.matches("Otro titulo").count(), 12);
        assert_eq!(todo.matches("Titulo extenso").count(), 40);
        // El texto de la seccion sigue con la ruta de su titulo.
        let seccion = c.iter().find(|x| x.text.contains("Texto de la seccion.")).unwrap();
        assert!(seccion.path.len() == 1 && seccion.path[0].starts_with("Titulo extenso"), "{}: {:?}", max, seccion.path);
    }
}

/// Generador pseudoaleatorio con semilla fija (xorshift): la prueba da siempre lo mismo.
struct Azar(u64);

impl Azar {
    fn hasta(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }

    fn texto(&mut self, palabras: usize) -> String {
        const P: [&str; 16] = ["casco", "bloqueo", "1.", "#", "-", "[1]", "<b>", "ñandú", "🦺", "Riesgo.", "\\", "|", "`x`", "&lt;", "OBJETIVO", "verificar"];
        (0..palabras).map(|_| P[self.hasta(P.len())]).collect::<Vec<_>>().join(" ")
    }
}

#[test]
fn ningun_fragmento_pasa_el_maximo() {
    // Documentos armados al azar con todo tipo de bloques y largos: titulos mas largos que el
    // fragmento, secciones cortas, tablas anchas y angostas, imagenes, listas y notas.
    use doc2json::doc::{Block, Cell, Document};
    let mut azar = Azar(0x2545_F491_4F6C_DD1D);
    for _ in 0..300 {
        let mut content = Vec::new();
        for _ in 0..azar.hasta(25) + 1 {
            let largo = [1, 4, 20, 80, 300][azar.hasta(5)];
            let palabras = azar.hasta(largo) + 1;
            let texto = azar.texto(palabras);
            content.push(match azar.hasta(7) {
                0 => Block { kind: Kind::Heading, level: azar.hasta(4) as u8 + 1, text: texto, ..Block::default() },
                1 => Block { kind: Kind::Title, text: texto, ..Block::default() },
                2 => Block { kind: Kind::ListItem, ordered: true, marker: Some("1.".into()), level: azar.hasta(3) as u8, text: texto, ..Block::default() },
                3 => {
                    let (filas, columnas) = (azar.hasta(6) + 1, azar.hasta(10) + 1);
                    let mut rows = Vec::new();
                    for _ in 0..filas {
                        let mut fila = Vec::new();
                        for _ in 0..columnas {
                            let palabras = azar.hasta(largo) + 1;
                            fila.push(Cell { text: azar.texto(palabras), colspan: 1, rowspan: 1, merged: false });
                        }
                        rows.push(fila);
                    }
                    Block { kind: Kind::Table, rows, ..Block::default() }
                }
                4 => Block { kind: Kind::Image, name: Some("media/imagen 1.png".into()), text: texto, ..Block::default() },
                5 => Block { kind: Kind::Table, rows: vec![vec![Cell { text: "3.2 RESPONSABLES".into(), colspan: 1, rowspan: 1, merged: false }]], ..Block::default() },
                _ => Block { kind: Kind::Paragraph, text: texto, ..Block::default() },
            });
        }
        let palabras = azar.hasta(200) + 1;
        let notas = vec![Block { label: Some("1".into()), text: azar.texto(palabras), ..Block::default() }];
        let d = Document { kind: "docx", content, extras: vec![("footnotes", notas)], ..Document::default() };
        for max in [200, 257, 400, 1000] {
            for c in doc2json::markdown::chunks(&d, max) {
                assert!(c.text.chars().count() <= max, "{}: {}\n{}", max, c.text.chars().count(), c.text);
            }
        }
    }
}
