//! Linea de comandos: `doc2json <archivo.docx|archivo.pdf> [salida.json] [--full]`

use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut full = false;
    for a in std::env::args().skip(1) {
        match a.as_str() {
            "--full" => full = true,
            "-h" | "--help" => return usage(),
            _ if input.is_none() => input = Some(PathBuf::from(a)),
            _ if output.is_none() => output = Some(PathBuf::from(a)),
            _ => return usage(),
        }
    }
    let input = match input {
        Some(p) => p,
        None => return usage(),
    };
    let bytes = match std::fs::read(&input) {
        Ok(b) => b,
        Err(e) => return fail(&format!("No se pudo leer {}: {}", input.display(), e)),
    };
    let doc = match doc2json::read(&bytes) {
        Ok(d) => d,
        Err(e) => return fail(&e),
    };
    // Sin destino explicito, el JSON queda junto al documento con el mismo nombre.
    let out = output.unwrap_or_else(|| input.with_extension("json"));
    if let Err(e) = std::fs::write(&out, doc2json::doc::to_json(&doc, full)) {
        return fail(&format!("No se pudo escribir {}: {}", out.display(), e));
    }
    let pages = doc.page_count.map_or(String::new(), |n| format!("{} paginas, ", n));
    say(&format!("Leido ({}): {}{} bloques de contenido.", doc.kind, pages, doc.content.len()));
    for w in &doc.warnings {
        say(&format!("Aviso: {}", w));
    }
    say(&format!("JSON: {}", out.display()));
    ExitCode::SUCCESS
}

/// Imprime sin abortar si la salida esta cerrada.
fn say(msg: &str) {
    let _ = writeln!(std::io::stdout(), "{}", msg);
}

fn usage() -> ExitCode {
    eprintln!("Uso: doc2json <archivo.docx|archivo.pdf> [salida.json] [--full]");
    ExitCode::from(2)
}

fn fail(msg: &str) -> ExitCode {
    eprintln!("Error: {}", msg);
    ExitCode::FAILURE
}
