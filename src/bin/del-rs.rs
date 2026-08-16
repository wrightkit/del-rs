//! del-rs CLI: parse / check / inspect / hir / matrix.
//!
//! Exit codes (stable contract): 0 success, 1 errors found, 2 usage error,
//! 3 internal error, 4 I/O error.

use del_rs::matrix;
use del_rs::syntax::parse_source;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_help();
        return ExitCode::SUCCESS;
    }
    if args[0] == "--version" || args[0] == "-V" {
        println!("del-rs {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    let result = match args[0].as_str() {
        "parse" => cmd_parse(&args[1..]),
        "matrix" => cmd_matrix(&args[1..]),
        other => {
            eprintln!("del-rs: unknown command '{other}'");
            print_help();
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(code) => ExitCode::from(code),
        Err(msg) => {
            eprintln!("del-rs: {msg}");
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    println!(
        "del-rs — Workshop-independent OSTW/DeltinScript frontend\n\
\n\
Usage: del-rs <command> [args]\n\
\n\
Commands:\n\
  parse <file> [--json]     Lex + parse a file; print diagnostics and an AST summary\n\
  matrix [--check] [--json] Print or validate the embedded compatibility matrix\n\
  --version, --help\n\
\n\
Exit codes: 0 success, 1 errors found, 2 usage error, 3 internal error, 4 I/O error"
    );
}

fn cmd_parse(args: &[String]) -> Result<u8, String> {
    let mut json = false;
    let mut file: Option<PathBuf> = None;
    for a in args {
        match a.as_str() {
            "--json" => json = true,
            _ if file.is_none() => file = Some(PathBuf::from(a)),
            _ => return Err(format!("unknown argument '{a}'")),
        }
    }
    let file = file.ok_or("parse requires a file argument")?;
    let text = std::fs::read_to_string(&file).map_err(|e| format!("cannot read {}: {e}", file.display()))?;
    let mut sources = del_rs::SourceMap::new();
    let id = sources.add_file(file.clone(), text);
    let out = parse_source(id, sources.text(id));
    let errors: usize = out.diagnostics.iter().filter(|d| d.is_error()).count();
    if json {
        let doc = serde_json::json!({
            "command": "parse",
            "phase": "parse",
            "file": file.display().to_string(),
            "diagnostics": out.diagnostics,
            "summary": {
                "items": out.ast.items.len(),
                "tokens": out.tokens.len(),
                "errors": errors,
            },
        });
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
    } else {
        for d in &out.diagnostics {
            let lc = sources.line_col(d.primary, d.primary.start);
            eprintln!("[{}] {}:{}:{}: {}", d.code, d.primary.file.0, lc.line, lc.col, d.message);
        }
        println!(
            "parsed {}: {} items, {} tokens, {} diagnostics ({} errors)",
            file.display(),
            out.ast.items.len(),
            out.tokens.len(),
            out.diagnostics.len(),
            errors
        );
    }
    Ok(if errors > 0 { 1 } else { 0 })
}

fn cmd_matrix(args: &[String]) -> Result<u8, String> {
    let mut check = false;
    let mut json = false;
    for a in args {
        match a.as_str() {
            "--check" => check = true,
            "--json" => json = true,
            _ => return Err(format!("unknown argument '{a}'")),
        }
    }
    match matrix::load_and_validate() {
        Ok(m) => {
            if json {
                let counts: serde_json::Value = matrix::state_counts(&m)
                    .into_iter()
                    .map(|(s, n)| (format!("{s:?}"), n))
                    .collect();
                let doc = serde_json::json!({
                    "command": "matrix",
                    "phase": "matrix",
                    "valid": true,
                    "entries": m.entries.len(),
                    "states": counts,
                });
                println!("{}", serde_json::to_string_pretty(&doc).unwrap());
            } else if check {
                println!("support matrix valid: {} entries", m.entries.len());
            } else {
                let counts = matrix::state_counts(&m);
                println!("del-rs support matrix (upstream pin: {})", m.meta.upstream_pin);
                for (s, n) in counts {
                    println!("  {s:?}: {n}");
                }
            }
            Ok(0)
        }
        Err(problems) => {
            if json {
                let doc = serde_json::json!({
                    "command": "matrix",
                    "phase": "matrix",
                    "valid": false,
                    "problems": problems,
                });
                println!("{}", serde_json::to_string_pretty(&doc).unwrap());
            } else {
                for p in &problems {
                    eprintln!("matrix problem: {p}");
                }
            }
            Ok(1)
        }
    }
}
