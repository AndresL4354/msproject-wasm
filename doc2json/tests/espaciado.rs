//! Espacios entre palabras en PDF armados en el codigo: interletra declarada y no usada, anchos de
//! fuente exagerados, lineas justificadas apretadas, letras mal ubicadas y ligaduras.

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

/// El texto de una pagina con el contenido dado y la fuente `/F1` descrita por `font`.
fn texto(font: &str, contenido: &str) -> String {
    let objs = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 612 792] >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".to_string(),
        format!("<< /Length {} >>\nstream\n{}\nendstream", contenido.len(), contenido),
        font.to_string(),
    ];
    doc2json::read(&pdf(&objs)).unwrap().pages[0].text.clone()
}

const HELVETICA: &str = "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>";

#[test]
fn interletra_declarada_que_el_pdf_no_usa() {
    // `Tc` de un em, pero cada letra ubicada con `Td` justo donde termina la anterior (anchos de
    // Helvetica: S 667, E 667, L 556, R 722, espacio 278, O 778, K 667).
    let c = "BT /F1 10 Tf 10 Tc 72 700 Td (S) Tj 6.67 0 Td (E) Tj 6.67 0 Td (L) Tj 5.56 0 Td (L) Tj 5.56 0 Td (E) Tj 6.67 0 Td (R) Tj \
             7.22 0 Td ( ) Tj 2.78 0 Td (O) Tj 7.78 0 Td (K) Tj ET";
    assert_eq!(texto(HELVETICA, c), "SELLER OK");
}

#[test]
fn interletra_que_si_se_usa_no_separa_letras() {
    // Una palabra con `Tc` dentro de una sola cadena: las letras avanzan con el espaciado.
    assert_eq!(texto(HELVETICA, "BT /F1 10 Tf 3 Tc 72 700 Td (TITULO) Tj ET"), "TITULO");
}

#[test]
fn anchos_de_fuente_exagerados() {
    // La fuente declara 1500 milesimas para cada letra, pero el PDF ubica cada una 6 puntos despues
    // de la anterior: van pegadas, no montadas en palabras distintas.
    let font = "<< /Type /Font /Subtype /Type1 /BaseFont /Arial /FirstChar 32 /LastChar 90 /Widths [278 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 \
                1500 1500 1500 1500 1500 1500 1500 1500 1500 1500 1500 1500 1500 1500 1500 1500 1500 1500 1500 1500 1500 1500 1500 1500 1500 1500] >>";
    let c = "BT /F1 10 Tf 72 700 Td (S) Tj 6 0 Td (E) Tj 6 0 Td (L) Tj 6 0 Td (L) Tj 6 0 Td (E) Tj 6 0 Td (R) Tj ET";
    assert_eq!(texto(font, c), "SELLER");
}

#[test]
fn linea_justificada_apretada() {
    // Palabras separadas solo por su posicion, a 0,146 em (menos que el espacio comun de 0,15).
    let c = "BT /F1 10 Tf 72 700 Td [(The)-146(next)-146(step)-146(after)-146(correction)]TJ ET";
    assert_eq!(texto(HELVETICA, c), "The next step after correction");
}

#[test]
fn hueco_dudoso_suelto_no_parte_una_palabra() {
    // Una letra corrida en una exportacion (un hueco, o dos, de 0,11-0,12 em) sigue siendo la misma palabra.
    assert_eq!(texto(HELVETICA, "BT /F1 10 Tf 72 700 Td [(VELOC)-120(IDAD)]TJ ET"), "VELOCIDAD");
    assert_eq!(texto(HELVETICA, "BT /F1 10 Tf 72 700 Td [(expro)-110(piacio)-112(nes)]TJ ET"), "expropiaciones");
    // Letras todas separadas por igual (interletra amplia): no hay apretadas, no se inventan espacios.
    assert_eq!(texto(HELVETICA, "BT /F1 10 Tf 72 700 Td [(c)-110(o)-110(t)-110(a)-110(s)]TJ ET"), "cotas");
}

#[test]
fn ligadura_y_su_espacio_fantasma() {
    // 0xAE es la ligadura "fi" en la codificacion estandar; tras ella el PDF dibuja un espacio
    // montado sobre el final de la ligadura, que no separa nada.
    let c = "BT /F1 10 Tf 72 700 Td [(super\\256)250( cial)]TJ ET";
    assert_eq!(texto(HELVETICA, c), "superficial");
    // Un espacio de verdad tras una ligadura si separa.
    assert_eq!(texto(HELVETICA, "BT /F1 10 Tf 72 700 Td (o\\256 ce) Tj ET"), "ofi ce");
}

/// Una fuente que declara para A-Z los anchos dados (milesimas de em), y 278 para el espacio.
fn fuente_con_anchos(az: &[u32; 26]) -> String {
    let mut w = vec!["0".to_string(); 33];
    w[0] = "278".to_string();
    w.extend(az.iter().map(|v| v.to_string()));
    format!("<< /Type /Font /Subtype /Type1 /BaseFont /Arial /FirstChar 32 /LastChar 90 /Widths [{}] >>", w.join(" "))
}

/// Cada letra ubicada aparte con `Tm` en la posicion dada (tamano 10).
fn letras_ubicadas(letras: &[(char, f64)]) -> String {
    let mut c = "BT /F1 10 Tf".to_string();
    for (l, x) in letras {
        c.push_str(&format!(" 1 0 0 1 {:.2} 700 Tm ({}) Tj", x, l));
    }
    c + " ET"
}

/// Anchos de Helvetica para A-Z.
const ANCHOS_HELVETICA: [u32; 26] = [667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611];

#[test]
fn anchos_exagerados_y_palabras_separadas_solo_por_posicion() {
    // Todas las letras declaran 1500 milesimas y avanzan 6 puntos; entre palabras avanzan 9. Las
    // letras van pegadas, pero las dos palabras no.
    let mut letras = Vec::new();
    let mut x = 72.0;
    for (i, l) in "SELLERBUYER".chars().enumerate() {
        if i == 6 {
            x += 3.0;
        }
        letras.push((l, x));
        x += 6.0;
    }
    assert_eq!(texto(&fuente_con_anchos(&[1500; 26]), &letras_ubicadas(&letras)), "SELLER BUYER");

    // Anchos proporcionales declarados al doble de los reales: cada letra avanza su ancho real
    // de Helvetica y entre palabras se suma el ancho de un espacio.
    let doble: Vec<u32> = ANCHOS_HELVETICA.iter().map(|w| w * 2).collect();
    let mut letras = Vec::new();
    let mut x = 72.0;
    for palabra in ["SELLER", "BUYER", "WAREHOUSE"] {
        for l in palabra.chars() {
            letras.push((l, x));
            x += ANCHOS_HELVETICA[(l as u8 - b'A') as usize] as f64 / 100.0;
        }
        x += 2.78;
    }
    assert_eq!(texto(&fuente_con_anchos(&doble.try_into().unwrap()), &letras_ubicadas(&letras)), "SELLER BUYER WAREHOUSE");

    // Anchos declarados iguales y letras reales de distinto ancho: una letra ancha (M, W) no se
    // distingue de un espacio, asi que la palabra no se parte.
    let mut letras = Vec::new();
    let mut x = 72.0;
    for l in "TIMEWORKMAXIMUMSUBMITTAL".chars() {
        letras.push((l, x));
        x += ANCHOS_HELVETICA[(l as u8 - b'A') as usize] as f64 / 100.0;
    }
    assert_eq!(texto(&fuente_con_anchos(&[1500; 26]), &letras_ubicadas(&letras)), "TIMEWORKMAXIMUMSUBMITTAL");
}

#[test]
fn anchos_exagerados_con_primera_palabra_corta() {
    // Como arriba (1500 milesimas, avance 6, hueco 9), pero el tramo empieza con una palabra
    // corta: el espacio se decide con todo el tramo a la vista, asi que el primero tambien se ve.
    let linea = |palabras: &[&str]| {
        let mut letras = Vec::new();
        let mut x = 72.0;
        for palabra in palabras {
            for l in palabra.chars() {
                letras.push((l, x));
                x += 6.0;
            }
            x += 3.0;
        }
        texto(&fuente_con_anchos(&[1500; 26]), &letras_ubicadas(&letras))
    };
    assert_eq!(linea(&["THIS", "SELLER", "SHALL"]), "THIS SELLER SHALL");
    assert_eq!(linea(&["THE", "SELLER", "SHALL"]), "THE SELLER SHALL");
    assert_eq!(linea(&["A", "SELLER", "SHALL"]), "A SELLER SHALL");
    assert_eq!(linea(&["I", "AM", "THE", "SELLER"]), "I AM THE SELLER");
}

/// Palabras con cada letra ubicada aparte en su ancho real de Helvetica y un espacio entre
/// palabras, con una fuente que declara 1500 milesimas para todas las letras.
fn helvetica_con_anchos_iguales(palabras: &[&str]) -> String {
    let mut letras = Vec::new();
    let mut x = 72.0;
    for palabra in palabras {
        for l in palabra.chars() {
            letras.push((l, x));
            x += ANCHOS_HELVETICA[(l as u8 - b'A') as usize] as f64 / 100.0;
        }
        x += 2.78;
    }
    texto(&fuente_con_anchos(&[1500; 26]), &letras_ubicadas(&letras))
}

#[test]
fn anchos_iguales_y_letras_reales_distintas_no_parten_palabras() {
    // Las primeras letras miden lo mismo (S, A, V, E, P: 6,67 puntos), y despues viene una ancha
    // (W, M): sin todo el tramo a la vista pareceria un espacio. No se parte ninguna palabra.
    for palabras in [&["SAVE", "WAREHOUSE"][..], &["PASSAGE", "MADE"], &["SEA", "WAVES", "MAKE", "WAVES"]] {
        let t = helvetica_con_anchos_iguales(palabras);
        assert_eq!(t.replace(' ', ""), palabras.concat(), "{:?}", palabras);
        for p in palabras {
            assert!(t.contains(p), "{:?} partida en {:?}", p, t);
        }
    }
}

#[test]
fn miles_con_espacio_fino_no_son_espacios() {
    // Tres huecos iguales de 0,12 em entre cifras, con las demas uniones apretadas: agrupan miles.
    assert_eq!(texto(HELVETICA, "BT /F1 10 Tf 72 700 Td [(1)-120(234)-120(567)-120(890)]TJ ET"), "1234567890");
    // En una linea justificada apretada, las palabras se separan y la cifra queda entera.
    let c = "BT /F1 10 Tf 72 700 Td [(The)-130(total)-130(was)-130(1)-130(234)-130(567)(.)]TJ ET";
    assert_eq!(texto(HELVETICA, c), "The total was 1234567.");
}

#[test]
fn linea_justificada_muy_larga() {
    // Un solo `TJ` con muchas palabras apretadas: los espacios se agregan de una vez.
    let n = 100_000;
    let c = format!("BT /F1 10 Tf 72 700 Td [{}(fin)]TJ ET", "(abcd)-120".repeat(n));
    let t = texto(HELVETICA, &c);
    assert_eq!(t.len(), n * 5 + 3);
    assert!(t.starts_with("abcd abcd ") && t.ends_with("abcd fin"));
}
