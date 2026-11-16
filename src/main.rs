//! Linea de comandos: `mpp2xlsx archivo.mpp [salida.xlsx] [--json salida.json] [--json-full salida.json]`

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut input: Option<PathBuf> = None;
    let mut xlsx_out: Option<PathBuf> = None;
    let mut json_out: Option<(PathBuf, bool)> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" | "--json-full" => {
                match args.get(i + 1) {
                    Some(p) => json_out = Some((PathBuf::from(p), args[i] == "--json-full")),
                    None => return usage(),
                }
                i += 1;
            }
            "-h" | "--help" => return usage(),
            a if input.is_none() => input = Some(PathBuf::from(a)),
            a if xlsx_out.is_none() => xlsx_out = Some(PathBuf::from(a)),
            _ => return usage(),
        }
        i += 1;
    }
    let input = match input {
        Some(p) => p,
        None => return usage(),
    };
    let bytes = match std::fs::read(&input) {
        Ok(b) => b,
        Err(e) => return fail(&format!("No se pudo leer {}: {}", input.display(), e)),
    };
    let project = match mpp2xlsx::read_mpp(&bytes) {
        Ok(p) => p,
        Err(e) => return fail(&e),
    };
    say(&format!(
        "Leido: {} tareas, {} recursos, {} asignaciones, {} relaciones.",
        project.tasks.len(),
        project.resources.iter().filter(|r| r.name.is_some()).count(),
        project.assignments.len(),
        project.relations.len()
    ));

    // Sin destino explicito, el Excel queda junto al .mpp con el mismo nombre.
    let only_json = json_out.is_some() && xlsx_out.is_none();
    if !only_json {
        let out = xlsx_out.unwrap_or_else(|| input.with_extension("xlsx"));
        if let Err(e) = write(&out, &mpp2xlsx::xlsx::workbook(&project)) {
            return fail(&e);
        }
        say(&format!("Excel: {}", out.display()));
    }
    if let Some((path, full)) = json_out {
        if let Err(e) = write(&path, mpp2xlsx::json::project_json(&project, full).as_bytes()) {
            return fail(&e);
        }
        say(&format!("JSON:  {}", path.display()));
    }
    ExitCode::SUCCESS
}

/// Imprime sin abortar si la salida esta cerrada (por ejemplo, al encadenar con `head`).
fn say(msg: &str) {
    let _ = writeln!(std::io::stdout(), "{}", msg);
}

fn write(path: &Path, data: &[u8]) -> Result<(), String> {
    std::fs::write(path, data).map_err(|e| format!("No se pudo escribir {}: {}", path.display(), e))
}

fn usage() -> ExitCode {
    eprintln!("Uso: mpp2xlsx <archivo.mpp> [salida.xlsx] [--json salida.json] [--json-full salida.json]");
    ExitCode::from(2)
}

fn fail(msg: &str) -> ExitCode {
    eprintln!("Error: {}", msg);
    ExitCode::FAILURE
}
