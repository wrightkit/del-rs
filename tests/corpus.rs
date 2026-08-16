//! Corpus harness: walks `tests/corpus/**/*.{del,ostw,workshop}` and checks
//! each fixture's declared outcome against the frontend pipeline.
//!
//! Header directives (leading comment block):
//! - `// source: <url>` — required (provenance)
//! - `// license: <id>` — required
//! - `// expect: ok | parse-error | semantic-error | hir-error | unknown`
//!
//! `projects/` fixtures are exercised by dedicated project tests, not the
//! generic walker.

use del_rs::diagnostics::{Diagnostic, Phase};
use del_rs::project::{load_project, ProjectOptions};
use del_rs::syntax::parse_source;
use del_rs::{FileId, SourceMap};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Expect {
    Ok,
    ParseError,
    SemanticError,
    HirError,
    Unknown,
}

fn parse_expect(line: &str) -> Option<Expect> {
    let line = line.trim_start();
    let line = line.strip_prefix("//")?.trim_start();
    let (key, value) = line.split_once(':')?;
    if key.trim() != "expect" {
        return None;
    }
    match value.trim() {
        "ok" => Some(Expect::Ok),
        "parse-error" => Some(Expect::ParseError),
        "semantic-error" => Some(Expect::SemanticError),
        "hir-error" => Some(Expect::HirError),
        "unknown" => Some(Expect::Unknown),
        other => panic!("corpus fixture has invalid expect value: {other}"),
    }
}

fn header_directives(text: &str) -> (Option<Expect>, bool, bool) {
    let mut expect = None;
    let mut has_source = false;
    let mut has_license = false;
    for line in text.lines().take(8) {
        let t = line.trim_start();
        if !t.starts_with("//") {
            break;
        }
        let t = t.trim_start_matches('/').trim_start();
        if let Some((k, v)) = t.split_once(':') {
            match k.trim() {
                "expect" => expect = parse_expect(line),
                "source" => has_source = true,
                "license" => has_license = true,
                _ => {}
            }
        }
    }
    (expect, has_source, has_license)
}

fn errors_at(diags: &[Diagnostic], phase: Phase) -> usize {
    diags.iter().filter(|d| d.phase == phase && d.is_error()).count()
}

fn any_error(diags: &[Diagnostic]) -> bool {
    diags.iter().any(|d| d.is_error())
}

struct CaseResult {
    path: String,
    expect: Expect,
    outcome: &'static str,
}

fn run_case(path: &Path, text: &str, expect: Expect) -> CaseResult {
    let mut sources = SourceMap::new();
    let id = sources.add_file(path.to_path_buf(), text.to_string());
    let out = parse_source(id, text);
    let parse_errors = out.diagnostics.iter().filter(|d| d.is_error()).count();

    let outcome: &'static str = match expect {
        Expect::Ok => {
            if parse_errors == 0 {
                "PASS"
            } else {
                "FAIL"
            }
        }
        Expect::ParseError => {
            if parse_errors > 0 {
                "PASS"
            } else {
                "FAIL"
            }
        }
        Expect::SemanticError | Expect::HirError => {
            // Full-stage checks land with issues #4/#6; for now the parse
            // stage must be clean for these fixtures.
            if parse_errors == 0 {
                "PENDING"
            } else {
                "FAIL"
            }
        }
        Expect::Unknown => "PENDING",
    };
    CaseResult {
        path: path.display().to_string(),
        expect,
        outcome,
    }
}

#[test]
fn corpus_parse_harness() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus");
    let mut cases = Vec::new();
    let mut total = 0usize;
    for category in ["parser", "semantic", "highlevel"] {
        let dir = root.join(category);
        if !dir.exists() {
            continue;
        }
        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| matches!(p.extension().and_then(|e| e.to_str()), Some("del" | "ostw" | "workshop")))
            .collect();
        files.sort();
        for f in files {
            total += 1;
            let text = std::fs::read_to_string(&f).unwrap();
            let (expect, has_source, has_license) = header_directives(&text);
            let expect = expect.unwrap_or_else(|| {
                panic!("fixture {} is missing a // expect: header", f.display())
            });
            assert!(has_source, "fixture {} is missing // source:", f.display());
            assert!(has_license, "fixture {} is missing // license:", f.display());
            cases.push(run_case(&f, &text, expect));
        }
    }
    let passed = cases.iter().filter(|c| c.outcome == "PASS").count();
    let failed = cases.iter().filter(|c| c.outcome == "FAIL").count();
    let pending = cases.iter().filter(|c| c.outcome == "PENDING").count();
    eprintln!("corpus harness: {total} fixtures | pass {passed} | fail {failed} | pending {pending}");
    if failed > 0 {
        for c in cases.iter().filter(|c| c.outcome == "FAIL") {
            eprintln!("  FAIL {:?} {}", c.expect, c.path);
        }
        panic!("{failed} corpus fixtures failed the declared expectation");
    }
    if passed == 0 {
        panic!("corpus harness passed nothing");
    }
}

#[test]
fn project_fixtures_load() {
    // The projects/ fixtures are exercised as projects: entry loads imports.
    for (name, entry) in [
        ("modules", "PathfindEditor.del"),
        ("pathfinding", "Pathfinding.del"),
    ] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/corpus/projects")
            .join(name);
        let project = load_project(ProjectOptions {
            root: root.clone(),
            entry: Some(PathBuf::from(entry)),
            config: None,
        });
        let errors: Vec<String> = project
            .diagnostics
            .iter()
            .filter(|d| d.is_error())
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect();
        assert!(
            errors.is_empty(),
            "project {name}: {} errors:\n{}",
            errors.len(),
            errors.join("\n")
        );
        assert!(project.files.len() >= 2, "project {name}: expected imports to load, got files {:?}", project.files.len());
        eprintln!("project {name}: {} files loaded, {} imports", project.files.len(), project.imports.len());
    }
}
