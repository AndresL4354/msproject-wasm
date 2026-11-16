//! Mide cuanto tarda cada etapa de la conversion:  cargo run --release --example tiempos -- archivo.mpp
use std::time::Instant;

fn main() {
    let path = std::env::args().nth(1).expect("uso: tiempos <archivo.mpp>");
    let bytes = std::fs::read(&path).expect("no se pudo leer el archivo");
    let n = 20;

    let t = Instant::now();
    let mut project = None;
    for _ in 0..n {
        project = Some(mpp2xlsx::read_mpp(&bytes).expect("no se pudo leer el .mpp"));
    }
    let read_ms = t.elapsed().as_secs_f64() * 1000.0 / n as f64;
    let project = project.unwrap();

    let t = Instant::now();
    let mut xlsx_len = 0;
    for _ in 0..n {
        xlsx_len = mpp2xlsx::xlsx::workbook(&project).len();
    }
    let xlsx_ms = t.elapsed().as_secs_f64() * 1000.0 / n as f64;

    let t = Instant::now();
    let mut json_len = 0;
    for _ in 0..n {
        json_len = mpp2xlsx::json::project_json(&project, false).len();
    }
    let json_ms = t.elapsed().as_secs_f64() * 1000.0 / n as f64;

    println!("archivo: {} KB, {} tareas", bytes.len() / 1024, project.tasks.len());
    println!("leer .mpp:      {:6.1} ms", read_ms);
    println!("generar .xlsx:  {:6.1} ms  ({} KB)", xlsx_ms, xlsx_len / 1024);
    println!("generar JSON:   {:6.1} ms  ({} KB)", json_ms, json_len / 1024);
}
