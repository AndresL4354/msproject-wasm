//! Linea de comandos:
//! `doc2json <archivo.docx|archivo.pdf> [salida.json] [--full] [--max-mb=N] [--max-paginas=N] [--max-caracteres=N]`

use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut full = false;
    let mut limits = doc2json::budget::Limits::default();
    for a in std::env::args().skip(1) {
        // Limites de recursos: --max-mb, --max-paginas, --max-caracteres (enteros positivos).
        if let Some((flag, value)) = a.split_once('=').filter(|(f, _)| f.starts_with("--max-")) {
            let n = match value.parse::<usize>() {
                Ok(n) if n > 0 => n,
                _ => return usage(),
            };
            match flag {
                "--max-mb" => limits.max_decompressed = n.saturating_mul(1 << 20),
                "--max-paginas" => limits.max_pages = n,
                "--max-caracteres" => limits.max_chars = n,
                _ => return usage(),
            }
            continue;
        }
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
    let doc = match doc2json::read_with_limits(&bytes, limits) {
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
    eprintln!("Uso: doc2json <archivo.docx|archivo.pdf> [salida.json] [--full] [--max-mb=N] [--max-paginas=N] [--max-caracteres=N]");
    eprintln!("  --max-mb=N          datos descomprimidos en total, en MB (por defecto 128)");
    eprintln!("  --max-paginas=N     paginas de PDF que se leen (por defecto 20000)");
    eprintln!("  --max-caracteres=N  caracteres de texto (por defecto 50000000)");
    eprintln!("  Al alcanzar un limite se entrega lo leido hasta ahi, con un aviso.");
    ExitCode::from(2)
}

fn fail(msg: &str) -> ExitCode {
    eprintln!("Error: {}", msg);
    ExitCode::FAILURE
}
