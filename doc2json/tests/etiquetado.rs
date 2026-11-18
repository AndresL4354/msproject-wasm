//! PDF etiquetado de punta a punta: archivos armados en el codigo que pasan por el lector del
//! arbol de estructura y por el contenido marcado de las paginas, incluidos arboles danados o
//! malintencionados.

use doc2json::doc::{Block, Kind};

/// Arma un PDF con los objetos dados (el primero es el 1) y su tabla xref.
fn pdf(objs: &[String]) -> Vec<u8> {
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offs = Vec::with_capacity(objs.len());
    for (i, o) in objs.iter().enumerate() {
        offs.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", i + 1, o).as_bytes());
    }
    let x = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
    for o in offs {
        out.extend_from_slice(format!("{:010} 00000 n \n", o).as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", objs.len() + 1, x).as_bytes());
    out
}

fn stream(dict: &str, data: &str) -> String {
    format!("<< {} /Length {} >>\nstream\n{}\nendstream", dict, data.len(), data)
}

/// Texto en (x, y) dentro de contenido marcado con el MCID dado.
fn marcado(mcid: i64, x: i32, y: i32, texto: &str) -> String {
    format!("/P << /MCID {} >> BDC BT /F1 10 Tf {} {} Td ({}) Tj ET EMC\n", mcid, x, y, texto)
}

/// PDF de una pagina etiquetado: 1 catalogo, 2 paginas, 3 pagina, 4 contenido, 5 raiz de la
/// estructura (con `raiz` como /K), 6 fuente y los objetos `mas` desde el 7.
fn etiquetado(contenido: &str, raiz: &str, mas: &[String]) -> Vec<u8> {
    etiquetado_con(contenido, &format!("<< /Type /StructTreeRoot /K {} >>", raiz), "", mas)
}

fn etiquetado_con(contenido: &str, raiz: &str, recursos: &str, mas: &[String]) -> Vec<u8> {
    let mut objs = vec![
        "<< /Type /Catalog /Pages 2 0 R /StructTreeRoot 5 0 R /MarkInfo << /Marked true >> >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 6 0 R >> {} >> >>", recursos),
        stream("", contenido),
        raiz.to_string(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_string(),
    ];
    objs.extend(mas.iter().cloned());
    pdf(&objs)
}

fn etiquetado_si(d: &doc2json::Document) -> bool {
    d.meta.iter().any(|(k, v)| *k == "tagged" && matches!(v, doc2json::doc::Meta::Bool(true)))
}

fn textos(d: &doc2json::Document) -> Vec<&str> {
    d.content.iter().map(|b| b.text.as_str()).collect()
}

fn celdas(bloques: &[Block]) -> usize {
    bloques.iter().filter(|b| b.kind == Kind::Table).map(|b| b.rows.iter().map(|r| r.len()).sum::<usize>()).sum()
}

#[test]
fn parrafo_que_cruza_de_columna_sigue_el_orden_del_arbol() {
    // El parrafo empieza abajo en la columna izquierda y sigue arriba en la derecha: el orden
    // de lectura es el de /K, no el de la posicion en la pagina.
    let c = marcado(0, 72, 100, "Inicio del parrafo en la columna izquierda") + &marcado(1, 330, 700, "y su final arriba en la derecha.") + &marcado(2, 72, 60, "Otro parrafo.");
    let d = doc2json::read(&etiquetado(&c, "7 0 R", &["<< /S /Document /K [8 0 R 9 0 R] >>".into(), "<< /S /P /Pg 3 0 R /K [0 1] >>".into(), "<< /S /P /Pg 3 0 R /K 2 >>".into()])).unwrap();
    assert!(etiquetado_si(&d));
    assert_eq!(textos(&d), ["Inicio del parrafo en la columna izquierda y su final arriba en la derecha.", "Otro parrafo."]);
}

#[test]
fn tramos_de_una_misma_linea_se_unen_sin_espacio_de_mas() {
    // Una palabra partida en dos MCID (negrita en medio de la palabra) sigue siendo una palabra.
    let c = "/P << /MCID 0 >> BDC BT /F1 10 Tf 72 700 Td (Aislar las fuen) Tj ET EMC\n/Span << /MCID 1 >> BDC BT /F1 10 Tf 134.8 700 Td (tes) Tj ET EMC\n/P << /MCID 2 >> BDC BT /F1 10 Tf 148.14 700 Td ( de energia.) Tj ET EMC\n";
    let d = doc2json::read(&etiquetado(c, "7 0 R", &["<< /S /P /Pg 3 0 R /K [0 << /Type /MCR /MCID 1 >> 2] >>".into()])).unwrap();
    assert_eq!(textos(&d), ["Aislar las fuentes de energia."]);
}

#[test]
fn espacio_al_final_de_un_tramo_separa_del_siguiente() {
    // El primer MCID termina en espacio ("Hola "): su fin incluye el avance de ese espacio, asi
    // que el hueco hasta "mundo" es cero, pero son dos palabras.
    let c = "/P << /MCID 0 >> BDC BT /F1 10 Tf 72 700 Td (Hola ) Tj ET EMC\n/Span << /MCID 1 >> BDC BT /F1 10 Tf 95.34 700 Td (mundo) Tj ET EMC\n";
    let d = doc2json::read(&etiquetado(c, "7 0 R", &["<< /S /P /Pg 3 0 R /K [0 << /Type /MCR /MCID 1 >>] >>".into()])).unwrap();
    assert_eq!(textos(&d), ["Hola mundo"]);
}

#[test]
fn subindice_en_su_propio_contenido_queda_pegado() {
    // "p0-calculadora" con el 0 como subindice (mas chico y mas abajo) en otro MCID: sigue siendo
    // una palabra, como cuando la pagina se arma por posicion.
    let c = "/P << /MCID 0 >> BDC BT /F1 10 Tf 72 700 Td (Use la p) Tj ET EMC\n/Span << /MCID 1 >> BDC BT /F1 6.5 Tf 108.68 696.5 Td (0) Tj ET EMC\n/P << /MCID 2 >> BDC BT /F1 10 Tf 112.29 700 Td (-calculadora.) Tj ET EMC\n";
    let d = doc2json::read(&etiquetado(c, "7 0 R", &["<< /S /P /Pg 3 0 R /K [0 << /Type /MCR /MCID 1 >> 2] >>".into()])).unwrap();
    assert_eq!(textos(&d), ["Use la p0-calculadora."]);
}

#[test]
fn dos_columnas_a_la_misma_altura_no_se_mezclan() {
    // El parrafo termina abajo a la izquierda y sigue arriba a la derecha, con otra linea suya a la
    // misma altura que el final: no se arma como una sola linea con las dos columnas.
    let c = marcado(0, 72, 700, "Primera linea izquierda") + &marcado(0, 72, 686, "segunda linea izquierda") + &marcado(1, 330, 686, "y sigue a la derecha.");
    let d = doc2json::read(&etiquetado(&c, "7 0 R", &["<< /S /P /Pg 3 0 R /K [0 1] >>".into()])).unwrap();
    assert_eq!(textos(&d), ["Primera linea izquierda segunda linea izquierda y sigue a la derecha."]);
}

#[test]
fn artefacto_sin_emc_no_se_traga_el_resto_de_la_pagina() {
    // Falta el EMC del encabezado: sin la correccion, todo lo que sigue quedaba como artefacto.
    let c = marcado(0, 72, 700, "Primer parrafo del procedimiento.") + "/Artifact BMC BT /F1 10 Tf 72 760 Td (Encabezado) Tj ET\n" + &marcado(1, 72, 650, "Segundo parrafo, despues del artefacto abierto.");
    let d = doc2json::read(&etiquetado(&c, "[7 0 R 8 0 R]", &["<< /S /P /Pg 3 0 R /K 0 >>".into(), "<< /S /P /Pg 3 0 R /K 1 >>".into()])).unwrap();
    assert!(d.content.iter().any(|b| b.text == "Segundo parrafo, despues del artefacto abierto."), "{:?}", textos(&d));
    assert!(d.content.iter().any(|b| b.text.contains("Encabezado")), "{:?}", textos(&d));
}

#[test]
fn artefacto_sin_emc_dentro_de_un_formulario_no_sale_de_el() {
    // Un formulario (XObject) que abre un artefacto y no lo cierra no afecta a la pagina.
    let form = stream("/Type /XObject /Subtype /Form /BBox [0 0 612 792] /Resources << /Font << /F1 6 0 R >> >>", "/Artifact BMC BT /F1 10 Tf 72 760 Td (Logo) Tj ET");
    let c = marcado(0, 72, 700, "Antes del formulario.") + "/Fm1 Do\n" + &marcado(1, 72, 650, "Despues del formulario.");
    let raiz = "<< /Type /StructTreeRoot /K [7 0 R 8 0 R] >>";
    let d = doc2json::read(&etiquetado_con(&c, raiz, "/XObject << /Fm1 9 0 R >>", &["<< /S /P /Pg 3 0 R /K 0 >>".into(), "<< /S /P /Pg 3 0 R /K 1 >>".into(), form])).unwrap();
    assert!(d.content.iter().any(|b| b.text == "Despues del formulario."), "{:?}", textos(&d));
}

#[test]
fn tabla_con_relleno_enorme_no_agota_la_memoria() {
    // Una celda de 999 columnas y otra combinada hacia abajo 1000 filas, seguidas de 1000 filas
    // vacias: cada fila se rellenaba con 1000 celdas (un millon en total, con unos pocos KB).
    let filas = "<< /S /TR >> ".repeat(1000);
    let tabla = format!("<< /S /Table /Pg 3 0 R /K [<< /S /TR /K [<< /S /TD /A << /ColSpan 999 >> /K 0 >> << /S /TD /A << /RowSpan 1000 >> /K 1 >>] >> {}] >>", filas);
    let c = marcado(0, 72, 700, "Riesgo") + &marcado(1, 300, 700, "Control");
    let d = doc2json::read(&etiquetado(&c, "7 0 R", &[tabla])).unwrap();
    assert!(celdas(&d.content) <= 200_000, "{} celdas", celdas(&d.content));
    // El texto no se pierde: va como parrafos, con un aviso.
    assert!(d.content.iter().any(|b| b.text.contains("Riesgo")) && d.content.iter().any(|b| b.text.contains("Control")), "{:?}", textos(&d));
    assert!(d.warnings.iter().any(|w| w.contains("tabla")), "{:?}", d.warnings);
}

#[test]
fn tabla_mas_ancha_que_el_limite_va_como_parrafos() {
    // 50 celdas de 1000 columnas: 50.000 columnas, que se rellenaban en cada fila.
    let celdas_ = "<< /S /TD /A << /ColSpan 1000 >> >> ".repeat(49);
    let tabla = format!("<< /S /Table /Pg 3 0 R /K [<< /S /TR /K [{} << /S /TD /A << /RowSpan 3 >> /K 0 >>] >> << /S /TR >> << /S /TR >>] >>", celdas_);
    let d = doc2json::read(&etiquetado(&marcado(0, 72, 700, "Celda final"), "7 0 R", &[tabla])).unwrap();
    assert_eq!(celdas(&d.content), 0);
    assert!(d.content.iter().any(|b| b.text == "Celda final"), "{:?}", textos(&d));
}

#[test]
fn muchas_tablas_grandes_tienen_un_tope_por_documento() {
    // Seis tablas de casi 200.000 celdas cada una: cada una cabe, todas juntas no.
    let filas = "<< /S /TR >> ".repeat(999);
    let mut mas = vec![format!("<< /S /Document /K [{}] >>", (8..14).map(|n| format!("{} 0 R", n)).collect::<Vec<_>>().join(" "))];
    let mut c = String::new();
    for i in 0..6 {
        c += &marcado(i, 72, 700 - 20 * i as i32, &format!("Tabla {}", i + 1));
        mas.push(format!("<< /S /Table /Pg 3 0 R /K [<< /S /TR /K [<< /S /TD /A << /ColSpan 198 >> /K {} >> << /S /TD /A << /RowSpan 1000 >> >>] >> {}] >>", i, filas));
    }
    let d = doc2json::read(&etiquetado(&c, "7 0 R", &mas)).unwrap();
    assert!(celdas(&d.content) <= 1_000_000, "{} celdas", celdas(&d.content));
    for i in 1..=6 {
        assert!(d.content.iter().any(|b| b.text.contains(&format!("Tabla {}", i)) || b.rows.iter().flatten().any(|c| c.text == format!("Tabla {}", i))), "falta la tabla {}", i);
    }
    assert!(d.warnings.iter().any(|w| w.contains("tabla")), "{:?}", d.warnings);
}

#[test]
fn cadena_larga_de_arreglos_indirectos_no_desborda_la_pila() {
    // /K -> [8 0 R] -> [9 0 R] -> ... : antes la profundidad solo se contaba en los elementos.
    let n = 100_000;
    let mut mas: Vec<String> = (0..n).map(|i| format!("[{} 0 R]", 8 + i)).collect();
    mas.push("<< /S /P /Pg 3 0 R /K 0 >>".into());
    let d = doc2json::read(&etiquetado(&marcado(0, 72, 700, "Texto al final de la cadena."), "7 0 R", &mas)).unwrap();
    assert!(d.pages[0].text.contains("Texto al final de la cadena."));
}

#[test]
fn arbol_con_ciclos_no_se_cuelga_ni_repite() {
    let c = marcado(0, 72, 700, "Unico parrafo.");
    let d = doc2json::read(&etiquetado(&c, "[7 0 R 7 0 R]", &["<< /S /Div /K [8 0 R 7 0 R] >>".into(), "<< /S /P /Pg 3 0 R /K [0 8 0 R 7 0 R] >>".into()])).unwrap();
    assert_eq!(textos(&d), ["Unico parrafo."]);
}

#[test]
fn raiz_de_estructura_danada_usa_la_posicion() {
    let c = marcado(0, 72, 700, "Texto de la pagina.");
    for raiz in ["99 0 R", "(texto)", "[<< /S /P /K 0 >>]", "<< /S /P /Pg 77 0 R /K 0 >>", "null"] {
        let d = doc2json::read(&etiquetado(&c, raiz, &[])).unwrap();
        assert_eq!(textos(&d), ["Texto de la pagina."], "raiz {}", raiz);
        assert!(!etiquetado_si(&d), "raiz {}", raiz);
    }
    // La raiz misma no es un diccionario.
    let mut b = etiquetado_con(&c, "42", "", &[]);
    assert_eq!(textos(&doc2json::read(&b).unwrap()), ["Texto de la pagina."]);
    // Archivo cortado a la mitad: se reconstruye el indice y no hay panic.
    b.truncate(b.len() / 2);
    let _ = doc2json::read(&b);
}

#[test]
fn figura_con_imagen_cuenta_una_sola_vez() {
    let imagen = "<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8 /Length 1 >>\nstream\nX\nendstream".to_string();
    let c = marcado(0, 72, 700, "Pictograma de casco.") + "/Figure << /MCID 1 >> BDC q 50 0 0 50 72 600 cm /Im1 Do Q EMC\n";
    let raiz = "<< /Type /StructTreeRoot /K [7 0 R 8 0 R] >>";
    let d = doc2json::read(&etiquetado_con(&c, raiz, "/XObject << /Im1 9 0 R >>", &["<< /S /P /Pg 3 0 R /K 0 >>".into(), "<< /S /Figure /Alt (Casco) /Pg 3 0 R /K 1 >>".into(), imagen])).unwrap();
    assert!(d.content.iter().any(|b| b.kind == Kind::Image && b.text == "Casco"));
    let json = doc2json::doc::to_json(&d, false);
    let desde = &json[json.find("\"stats\"").unwrap()..];
    let stats = &desde[..desde.find('}').unwrap() + 1];
    assert!(stats.contains("\"images\":1"), "{}", stats);
}

/// Corre `f` y falla si tarda mas de `segundos`. Los casos de costo cuadratico tardaban minutos.
fn a_tiempo<T>(segundos: u64, f: impl FnOnce() -> T) -> T {
    let t = std::time::Instant::now();
    let r = f();
    assert!(t.elapsed() < std::time::Duration::from_secs(segundos), "tardo {:?}", t.elapsed());
    r
}

fn largo_del_texto(d: &doc2json::Document) -> usize {
    d.content.iter().map(|b| b.text.len()).sum()
}

#[test]
fn texto_alternativo_compartido_tiene_un_tope() {
    // 10.000 elementos que apuntan a una misma cadena de 8 KB: cada uno se quedaba con su copia
    // (80 MB; con 64.000 de 64 KB, varios GB). Pasado el tope, los demas se leen sin ese texto.
    let larga = format!("({})", "x".repeat(8 * 1024));
    let elementos = "<< /S /P /ActualText 8 0 R >> << /S /Figure /Alt 8 0 R >> ".repeat(5_000);
    let mas = [format!("<< /S /Document /K [9 0 R {}] >>", elementos), larga, "<< /S /P /Pg 3 0 R /K 0 >>".into()];
    let d = doc2json::read(&etiquetado(&marcado(0, 72, 700, "Parrafo real."), "7 0 R", &mas)).unwrap();
    assert!(largo_del_texto(&d) <= 20 << 20, "{} bytes", largo_del_texto(&d));
    assert_eq!(d.content.first().map(|b| b.text.as_str()), Some("Parrafo real."));
    assert!(d.warnings.iter().any(|w| w.contains("texto alternativo")), "{:?}", d.warnings);
}

#[test]
fn atributos_compartidos_se_leen_una_vez() {
    // 20.000 elementos con el mismo /A, un arreglo de 20.000 diccionarios: se recorria entero por
    // cada elemento (400 millones de diccionarios).
    let n = 20_000;
    let atributos = format!("[{}]", "<< /O /Layout >> ".repeat(n));
    let elementos = "<< /S /Span /A 8 0 R >> ".repeat(n);
    let mas = [format!("<< /S /Document /K [9 0 R {}] >>", elementos), atributos, "<< /S /P /Pg 3 0 R /K 0 >>".into()];
    let d = a_tiempo(5, || doc2json::read(&etiquetado(&marcado(0, 72, 700, "Parrafo real."), "7 0 R", &mas)).unwrap());
    assert_eq!(textos(&d), ["Parrafo real."]);
}

#[test]
fn rolemap_grande_no_se_recorre_por_cada_elemento() {
    // /RoleMap de 80.000 tipos y 80.000 elementos del ultimo: cada busqueda recorria el mapa entero.
    let n = 80_000;
    let mapa: String = (0..n).map(|i| format!("/R{:06} /P ", i)).collect();
    let elementos = format!("<< /S /R{:06} >> ", n - 1).repeat(n);
    let raiz = format!("<< /Type /StructTreeRoot /RoleMap << {} >> /K 7 0 R >>", mapa);
    let mas = [format!("<< /S /Document /K [8 0 R {}] >>", elementos), format!("<< /S /R{:06} /Pg 3 0 R /K 0 >>", n - 1)];
    let d = a_tiempo(5, || doc2json::read(&etiquetado_con(&marcado(0, 72, 700, "Parrafo con tipo propio."), &raiz, "", &mas)).unwrap());
    assert!(etiquetado_si(&d));
    assert_eq!(textos(&d), ["Parrafo con tipo propio."]);
}

/// PDF etiquetado de `n` paginas: la primera (objeto 3) con `contenido` y las demas con texto
/// suelto ("x", sin marcar). 1 catalogo, 2 paginas, 3 primera pagina, 4 su
/// contenido, 5 raiz de la estructura, 6 fuente, 7 contenido de las demas paginas, `mas` desde el
/// 8 y las demas paginas al final.
fn paginas(n: usize, contenido: &str, raiz: &str, mas: &[String]) -> Vec<u8> {
    let primera = 8 + mas.len();
    let kids: String = std::iter::once("3 0 R ".to_string()).chain((0..n - 1).map(|i| format!("{} 0 R ", primera + i))).collect();
    let pagina = |c: usize| format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents {} 0 R /Resources << /Font << /F1 6 0 R >> >> >>", c);
    let mut objs = vec![
        "<< /Type /Catalog /Pages 2 0 R /StructTreeRoot 5 0 R /MarkInfo << /Marked true >> >>".to_string(),
        format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids, n),
        pagina(4),
        stream("", contenido),
        raiz.to_string(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_string(),
        stream("", "BT /F1 10 Tf 72 400 Td (x) Tj ET"),
    ];
    objs.extend(mas.iter().cloned());
    objs.extend((0..n - 1).map(|_| pagina(7)));
    pdf(&objs)
}

#[test]
fn texto_suelto_de_muchas_paginas_no_es_cuadratico() {
    // 10.000 paginas con texto suelto y 100.000 bloques: cada pagina buscaba su lugar entre todos
    // los bloques y los corria para abrirlo (paginas x bloques).
    let n = 10_000;
    let c = marcado(0, 72, 700, &"y".repeat(12_000));
    let elementos = "<< /S /P /ActualText (a) >> ".repeat(100_000);
    let mas = [format!("<< /S /Document /K [9 0 R {}] >>", elementos), "<< /S /P /Pg 3 0 R /K 0 >>".into()];
    let d = a_tiempo(5, || doc2json::read(&paginas(n, &c, "<< /Type /StructTreeRoot /K 8 0 R >>", &mas)).unwrap());
    assert!(etiquetado_si(&d));
    assert_eq!(d.content.len(), 1 + 100_000 + n - 1);
    assert_eq!(d.content.iter().filter(|b| b.text == "x").count(), n - 1);
}

#[test]
fn texto_suelto_va_despues_de_los_bloques_de_su_pagina() {
    // Bloques del arbol en las paginas 2 y 3, el de la 3 primero en /K. El texto suelto de la
    // pagina 1 va al comienzo, y el de cada pagina despues del ultimo bloque de esa pagina o de una
    // anterior.
    let suelto = |t: &str| format!("BT /F1 10 Tf 72 400 Td ({}) Tj ET\n", t);
    let pagina = |c: usize| format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents {} 0 R /Resources << /Font << /F1 13 0 R >> >> >>", c);
    let objs = [
        "<< /Type /Catalog /Pages 2 0 R /StructTreeRoot 3 0 R /MarkInfo << /Marked true >> >>".to_string(),
        "<< /Type /Pages /Kids [4 0 R 5 0 R 6 0 R] /Count 3 >>".into(),
        "<< /Type /StructTreeRoot /K 7 0 R >>".into(),
        pagina(8),
        pagina(9),
        pagina(10),
        "<< /S /Document /K [11 0 R 12 0 R] >>".into(),
        stream("", &suelto("x")),
        stream("", &(marcado(0, 72, 700, "Parrafo de la pagina dos.") + &suelto("w"))),
        stream("", &(marcado(0, 72, 700, "Parrafo de la pagina tres.") + &suelto("z"))),
        "<< /S /P /Pg 6 0 R /K 0 >>".into(),
        "<< /S /P /Pg 5 0 R /K 0 >>".into(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".into(),
    ];
    let d = doc2json::read(&pdf(&objs)).unwrap();
    assert!(etiquetado_si(&d));
    assert_eq!(textos(&d), ["x", "Parrafo de la pagina tres.", "Parrafo de la pagina dos.", "w", "z"]);
}

fn extra<'d>(d: &'d doc2json::Document, clave: &str) -> Vec<&'d str> {
    d.extras.iter().filter(|(k, _)| *k == clave).flat_map(|(_, b)| b.iter().map(|b| b.text.as_str())).collect()
}

#[test]
fn bmc_bdc_y_emc_llegan_al_interprete() {
    // Los tres operadores del contenido marcado, con propiedades en linea y por nombre: si el
    // interprete deja de verlos (p. ej. al filtrar operadores), el PDF deja de leerse etiquetado.
    let c = "/Artifact << /Type /Pagination /Subtype /Header >> BDC BT /F1 10 Tf 72 760 Td (Encabezado del documento) Tj ET EMC\n".to_string()
        + "/P /MC0 BDC BT /F1 10 Tf 72 700 Td (Texto con propiedades por nombre.) Tj ET EMC\n"
        + "/Artifact BMC BT /F1 10 Tf 72 30 Td (Pie del documento) Tj ET EMC\n";
    let raiz = "<< /Type /StructTreeRoot /K 7 0 R >>";
    let d = doc2json::read(&etiquetado_con(&c, raiz, "/Properties << /MC0 << /MCID 0 >> >>", &["<< /S /P /Pg 3 0 R /K 0 >>".into()])).unwrap();
    assert!(etiquetado_si(&d));
    assert_eq!(textos(&d), ["Texto con propiedades por nombre."]);
    assert_eq!(extra(&d, "headers"), ["Encabezado del documento"]);
    assert_eq!(extra(&d, "footers"), ["Pie del documento"]);
}

#[test]
fn emc_sobrante_en_un_formulario_no_cierra_el_contenido_de_la_pagina() {
    // El formulario cierra dos veces lo que no abrio: antes cerraba el contenido marcado de la
    // pagina y el resto del parrafo (o del pie) quedaba fuera de su elemento.
    let form = stream("/Type /XObject /Subtype /Form /BBox [0 0 612 792]", "EMC EMC");
    let recursos = "/XObject << /Fm1 8 0 R >>";
    let raiz = "<< /Type /StructTreeRoot /K 7 0 R >>";
    let c = "/P << /MCID 0 >> BDC BT /F1 10 Tf 72 700 Td (Inicio del parrafo, antes del formulario.) Tj ET\n/Fm1 Do\nBT /F1 10 Tf 72 300 Td (Final del mismo parrafo.) Tj ET EMC\n";
    let d = doc2json::read(&etiquetado_con(c, raiz, recursos, &["<< /S /P /Pg 3 0 R /K 0 >>".into(), form.clone()])).unwrap();
    assert_eq!(textos(&d), ["Inicio del parrafo, antes del formulario. Final del mismo parrafo."]);
    // Un pie de pagina que invoca al formulario sigue siendo artefacto despues de el.
    let c = marcado(0, 72, 700, "Contenido de la pagina.") + "/Artifact << /Type /Pagination /Subtype /Footer >> BDC BT /F1 10 Tf 72 40 Td (Pie) Tj ET /Fm1 Do BT /F1 10 Tf 300 40 Td (Uso interno) Tj ET EMC\n";
    let d = doc2json::read(&etiquetado_con(&c, raiz, recursos, &["<< /S /P /Pg 3 0 R /K 0 >>".into(), form])).unwrap();
    assert_eq!(textos(&d), ["Contenido de la pagina."]);
    assert!(extra(&d, "footers").iter().any(|t| t.contains("Uso interno")), "{:?}", extra(&d, "footers"));
}

#[test]
fn mas_de_256_niveles_y_emc_sobrantes() {
    // 300 niveles dentro del parrafo: pasado el tope de 256 solo se cuentan, y sus EMC no cierran
    // el parrafo. Un formulario con un EMC de mas en el medio y los EMC que sobran al final no
    // cambian nada.
    let form = stream("/Type /XObject /Subtype /Form /BBox [0 0 612 792]", "EMC");
    let c = "/P << /MCID 0 >> BDC BT /F1 10 Tf 72 700 Td (Antes de los niveles.) Tj ET\n".to_string()
        + &"/Span BMC ".repeat(300)
        + "BT /F1 10 Tf 72 680 Td (Adentro.) Tj ET /Fm1 Do\n"
        + &"EMC ".repeat(300)
        + "BT /F1 10 Tf 72 660 Td (Despues de los niveles.) Tj ET EMC\nEMC EMC\n"
        + &marcado(1, 72, 600, "Otro parrafo.");
    let raiz = "<< /Type /StructTreeRoot /K [7 0 R 8 0 R] >>";
    let mas = ["<< /S /P /Pg 3 0 R /K 0 >>".to_string(), "<< /S /P /Pg 3 0 R /K 1 >>".into(), form];
    let d = doc2json::read(&etiquetado_con(&c, raiz, "/XObject << /Fm1 9 0 R >>", &mas)).unwrap();
    assert_eq!(textos(&d), ["Antes de los niveles. Adentro. Despues de los niveles.", "Otro parrafo."]);
}
