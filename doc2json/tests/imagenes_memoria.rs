//! Memoria que piden los documentos fabricados para agotarla con imagenes. Un asignador que cuenta
//! el maximo de bytes vivos (y el total pedido, que mide el trabajo) mide cada caso; los casos
//! corren uno tras otro (un solo `#[test]`) para que otras pruebas no se mezclen en la cuenta.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

struct Contador;

static VIVOS: AtomicUsize = AtomicUsize::new(0);
static MAXIMO: AtomicUsize = AtomicUsize::new(0);
static TOTAL: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Contador {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = System.alloc(l);
        if !p.is_null() {
            TOTAL.fetch_add(l.size(), Relaxed);
            let v = VIVOS.fetch_add(l.size(), Relaxed) + l.size();
            MAXIMO.fetch_max(v, Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        System.dealloc(p, l);
        VIVOS.fetch_sub(l.size(), Relaxed);
    }
}

#[global_allocator]
static ASIGNADOR: Contador = Contador;

/// Maximo de bytes vivos por encima de los de antes, mientras corre `f`.
fn pico<T>(f: impl FnOnce() -> T) -> (T, usize) {
    let base = VIVOS.load(Relaxed);
    MAXIMO.store(base, Relaxed);
    let r = f();
    (r, MAXIMO.load(Relaxed) - base)
}

/// Bytes pedidos en total (aunque se liberen) mientras corre `f`: cuanto se copio o decodifico.
fn pedido<T>(f: impl FnOnce() -> T) -> (T, usize) {
    let base = TOTAL.load(Relaxed);
    let r = f();
    (r, TOTAL.load(Relaxed) - base)
}

fn pdf(objects: &[Vec<u8>]) -> Vec<u8> {
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
    for o in offsets {
        out.extend_from_slice(format!("{:010} 00000 n \n", o).as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", objects.len() + 1, xref).as_bytes());
    out
}

fn stream(dict: &str, data: &[u8]) -> Vec<u8> {
    let mut out = format!("<< {} /Length {} >>\nstream\n", dict, data.len()).into_bytes();
    out.extend_from_slice(data);
    out.extend_from_slice(b"\nendstream");
    out
}

/// PDF de una pagina que dibuja una vez cada una de las `n` imagenes (objetos desde el 5), con los
/// objetos `extra` a continuacion (desde el `5 + n`).
fn muchas(n: usize, image: impl Fn(usize) -> Vec<u8>, extra: Vec<Vec<u8>>) -> Vec<u8> {
    let names: String = (0..n).map(|i| format!("/I{} {} 0 R ", i, i + 5)).collect();
    let content: String = (0..n).map(|i| format!("/I{} Do ", i)).collect();
    let mut objs = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 612 792] >>".to_vec(),
        format!("<< /Type /Page /Parent 2 0 R /Contents 4 0 R /Resources << /XObject << {} >> >> >>", names).into_bytes(),
        stream("", content.as_bytes()),
    ];
    objs.extend((0..n).map(image));
    objs.extend(extra);
    pdf(&objs)
}

/// ZIP sin compresion.
fn zip(parts: &[(String, Vec<u8>)]) -> Vec<u8> {
    let le16 = |v: usize| (v as u16).to_le_bytes();
    let le32 = |v: usize| (v as u32).to_le_bytes();
    let (mut z, mut central) = (Vec::new(), Vec::new());
    for (name, data) in parts {
        let n = name.as_bytes();
        let at = z.len();
        z.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        z.extend_from_slice(&le32(data.len()));
        z.extend_from_slice(&le32(data.len()));
        z.extend_from_slice(&le16(n.len()));
        z.extend_from_slice(&[0, 0]);
        z.extend_from_slice(n);
        z.extend_from_slice(data);
        central.extend_from_slice(&[0x50, 0x4B, 0x01, 0x02, 20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        central.extend_from_slice(&le32(data.len()));
        central.extend_from_slice(&le32(data.len()));
        central.extend_from_slice(&le16(n.len()));
        central.extend_from_slice(&[0; 12]);
        central.extend_from_slice(&le32(at));
        central.extend_from_slice(n);
    }
    let start = z.len();
    z.extend_from_slice(&central);
    z.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06, 0, 0, 0, 0]);
    z.extend_from_slice(&le16(parts.len()));
    z.extend_from_slice(&le16(parts.len()));
    z.extend_from_slice(&le32(central.len()));
    z.extend_from_slice(&le32(start));
    z.extend_from_slice(&[0, 0]);
    z
}

/// .docx con `n` imagenes distintas de `size` bytes cada una.
fn docx(n: usize, size: usize) -> Vec<u8> {
    const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    let drawings: String = (0..n)
        .map(|i| format!(r#"<w:p><w:r><w:drawing><wp:inline><wp:extent cx="254000" cy="254000"/><wp:docPr id="{0}" name="I"/><a:graphic><a:graphicData><pic:pic><pic:blipFill><a:blip r:embed="rId{0}"/></pic:blipFill></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#, i + 1))
        .collect();
    let body = format!(r#"<w:document xmlns:w="w" xmlns:wp="wp" xmlns:a="a" xmlns:pic="pic" xmlns:r="r"><w:body>{}</w:body></w:document>"#, drawings);
    let rels: String = (0..n).map(|i| format!(r#"<Relationship Id="rId{0}" Type="{1}/image" Target="media/i{0}.png"/>"#, i + 1, REL)).collect();
    let mut parts = vec![
        ("_rels/.rels".to_string(), format!(r#"<Relationships xmlns="r"><Relationship Id="rId1" Type="{}/officeDocument" Target="word/document.xml"/></Relationships>"#, REL).into_bytes()),
        ("word/document.xml".to_string(), body.into_bytes()),
        ("word/_rels/document.xml.rels".to_string(), format!(r#"<Relationships xmlns="r">{}</Relationships>"#, rels).into_bytes()),
    ];
    parts.extend((0..n).map(|i| (format!("word/media/i{}.png", i + 1), vec![i as u8; size])));
    zip(&parts)
}

fn pagina(content: &str, image: Vec<u8>) -> Vec<u8> {
    pdf(&[
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 612 792] >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /Contents 4 0 R /Resources << /XObject << /Im1 5 0 R >> >> >>".to_vec(),
        stream("", content.as_bytes()),
        image,
    ])
}

const MB: usize = 1 << 20;

#[test]
fn imagenes_fabricadas_no_agotan_la_memoria() {
    // 1. Diccionario de imagen con una cadena de 1 MB, dibujado 400 veces: cada `Do` no debe
    //    copiar el diccionario (antes, 400 MB).
    let grande = format!("/Relleno <{}>", "41".repeat(MB));
    let dict = format!("/Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8 {}", grande);
    let bytes = pagina(&"/Im1 Do ".repeat(400), stream(&dict, &[0]));
    let (d, usado) = pico(|| doc2json::read(&bytes).unwrap());
    assert_eq!(d.images[0].places.len(), 400);
    assert!(usado < 40 * MB, "dibujar 400 veces la misma imagen uso {} MB", usado / MB);

    // 2. 5.000 x 5.000 en RGB (75 MB) declarados con 150 bytes: no se reserva por el tamano declarado.
    let bytes = pagina("/Im1 Do", stream("/Subtype /Image /Width 5000 /Height 5000 /ColorSpace /DeviceRGB /BitsPerComponent 8", &[0; 150]));
    let (d, usado) = pico(|| doc2json::read_with_images(&bytes).unwrap());
    assert!(d.images[0].data.is_none());
    assert!(usado < 4 * MB, "una imagen de 150 bytes uso {} MB", usado / MB);

    // 3. Lo mismo con CMYK (se convierte a RGB) y con una mascara suave de pocos bytes.
    let bytes = pagina("/Im1 Do", stream("/Subtype /Image /Width 5000 /Height 5000 /ColorSpace /DeviceCMYK /BitsPerComponent 8", &[0; 150]));
    let (_, usado) = pico(|| doc2json::read_with_images(&bytes).unwrap());
    assert!(usado < 4 * MB, "CMYK de 150 bytes uso {} MB", usado / MB);

    // 4. La imagen como diccionario directo en los recursos, dibujada 100 veces. No es una imagen
    //    valida (le falta el flujo): no se registra, y ningun `Do` guarda una copia (antes, 100 MB).
    let bytes = pdf(&[
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 612 792] >>".to_vec(),
        format!("<< /Type /Page /Parent 2 0 R /Contents 4 0 R /Resources << /XObject << /Im1 << {} >> >> >> >>", dict).into_bytes(),
        stream("", "/Im1 Do ".repeat(100).as_bytes()),
    ]);
    let (d, usado) = pico(|| doc2json::read(&bytes).unwrap());
    assert!(usado < 40 * MB, "dibujar 100 veces una imagen directa uso {} MB", usado / MB);
    assert!(d.images.is_empty());
    assert_eq!(d.pages[0].images, 100);

    // 5. 200 imagenes con la misma paleta, cuya tabla es un flujo de 1 MB: la tabla se lee una sola
    //    vez al extraer, y ninguna al solo listar (antes, 200 MB en cada caso). Se compara con las
    //    mismas imagenes con una tabla de 3 bytes, para descontar lo que cuesta armar cada PNG.
    let paleta = |tabla: &[u8]| {
        muchas(
            200,
            |_| stream("/Subtype /Image /Width 1 /Height 1 /ColorSpace [/Indexed /DeviceRGB 0 205 0 R] /BitsPerComponent 8", &[0]),
            vec![stream("", tabla)],
        )
    };
    let (chica, grande) = (paleta(&[7; 3]), paleta(&vec![7; MB]));
    let (_, base) = pedido(|| doc2json::read(&chica).unwrap());
    let (d, total) = pedido(|| doc2json::read(&grande).unwrap());
    assert_eq!(d.images.len(), 200);
    assert!(total < base + 10 * MB, "listar 200 imagenes con una paleta compartida pidio {} MB", total / MB);
    let (_, base) = pedido(|| doc2json::read_with_images(&chica).unwrap());
    let (d, total) = pedido(|| doc2json::read_with_images(&grande).unwrap());
    assert!(d.images.iter().all(|i| i.data.is_some()), "{:?}", d.images[0].note);
    assert!(total < base + 10 * MB, "extraer 200 imagenes con una paleta compartida pidio {} MB (sin ella, {} MB)", total / MB, base / MB);

    // 6. 200 imagenes con la misma mascara suave, de 1 MB: se decodifica una sola vez (antes,
    //    200 MB). Se compara con la misma mascara de 1 byte.
    let mascara = |datos: &[u8]| {
        muchas(
            200,
            |_| stream("/Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8 /SMask 205 0 R", &[0]),
            vec![stream("/Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8", datos)],
        )
    };
    let (_, base) = pedido(|| doc2json::read_with_images(&mascara(&[128])).unwrap());
    let bytes = mascara(&vec![128; MB]);
    let (d, total) = pedido(|| doc2json::read_with_images(&bytes).unwrap());
    assert!(d.images.iter().all(|i| i.data.is_some()), "{:?}", d.images[0].note);
    assert!(total < base + 10 * MB, "extraer 200 imagenes con una mascara compartida pidio {} MB (con una de 1 byte, {} MB)", total / MB, base / MB);

    // 7. El tope cuenta los pixeles declarados antes de decodificar: con 1,5 MB, de 10 imagenes de
    //    1.000 x 1.000 (1 MB cada una, PNG de pocos KB) se decodifica una. Antes se decodificaban
    //    las 10, y todas cabian porque solo se contaba el PNG.
    let bytes = muchas(10, |i| stream("/Subtype /Image /Width 1000 /Height 1000 /ColorSpace /DeviceGray /BitsPerComponent 8", &vec![i as u8; MB]), vec![]);
    let (d, total) = pedido(|| doc2json::read_with_image_budget(&bytes, 3 * MB / 2).unwrap());
    assert!(total < 8 * MB, "decodificar 1 de 10 imagenes de 1 MB pidio {} MB", total / MB);
    assert_eq!(d.images.iter().filter(|i| i.data.is_some()).count(), 1);

    // 8. Lo mismo en Word, con el tamano descomprimido que declara el ZIP: de 10 imagenes de 1 MB
    //    con un tope de 1,5 MB se descomprime una (antes, las 10).
    let bytes = docx(10, MB);
    let (d, total) = pedido(|| doc2json::read_with_image_budget(&bytes, 3 * MB / 2).unwrap());
    assert!(total < 4 * MB, "leer 1 de 10 imagenes de 1 MB pidio {} MB", total / MB);
    assert_eq!(d.images.iter().filter(|i| i.data.is_some()).count(), 1);

    // 9. Fax de 1 x 100 millones: cabe en el tope de pixeles, pero cada fila ocupa un byte. Antes
    //    se reservaban 100 MB en el decodificador y otros 100 MB para las filas de la imagen.
    let fax = |w: u32, h: u32, data: &[u8]| {
        let dict = format!("/Subtype /Image /Width {} /Height {} /ColorSpace /DeviceGray /BitsPerComponent 1 /Filter /CCITTFaxDecode /DecodeParms << /K -1 /Columns {} >>", w, h, w);
        pagina("/Im1 Do", stream(&dict, data))
    };
    let (d, usado) = pico(|| doc2json::read_with_images(&fax(1, 100_000_000, &[0xFF, 0xFF])).unwrap());
    assert!(d.images[0].note.as_deref().unwrap_or("").contains("demasiado grande"), "{:?}", d.images[0].note);
    assert!(usado < 4 * MB, "fax de 1 x 100 millones uso {} MB", usado / MB);

    // 10. Fax de 8 x 10 millones con dos bytes: dan 16 filas. Antes se reservaban las 10 millones
    //     de filas (dos veces) y se armaba un PNG de ese alto; ahora se entrega tal cual, en TIFF.
    let (d, usado) = pico(|| doc2json::read_with_images(&fax(8, 10_000_000, &[0xFF, 0xFF])).unwrap());
    assert_eq!(d.images[0].format, "tiff");
    assert!(d.images[0].note.as_deref().unwrap_or("").contains("16 de 10000000"), "{:?}", d.images[0].note);
    assert!(usado < 4 * MB, "fax de 8 x 10 millones con 2 bytes uso {} MB", usado / MB);
}
