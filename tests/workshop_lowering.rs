//! Core DEL HIR -> canonical Workshop WIR lowering evidence for #30.

use del_rs::hir;
use del_rs::project::{load_project, ProjectOptions};
use del_rs::semantic::check_project;
use del_rs::semantic::provider::CatalogProvider;
use del_rs::workshop::lower_to_wir;
use std::path::PathBuf;

fn lower(text: &str) -> (workshop_rs::wir::Program, Vec<del_rs::Diagnostic>) {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "del-rs-workshop-lowering-{}-{n}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("main.del"), text).unwrap();
    let project = load_project(ProjectOptions {
        root,
        entry: Some(PathBuf::from("main.del")),
        config: None,
    });
    let provider = CatalogProvider::new().expect("canonical catalog provider");
    let semantic = check_project(&project, &provider);
    let mut diagnostics = semantic.diagnostics.clone();
    let (hir, hir_diags) = hir::lower::lower(&semantic);
    diagnostics.extend(hir_diags);
    let (program, lowering_diags) = lower_to_wir(&hir, &semantic.project.sources);
    diagnostics.extend(lowering_diags);
    (program, diagnostics)
}

#[test]
fn core_rule_lowering_preserves_canonical_ids_and_provenance() {
    let (program, diagnostics) = lower(
        r#"
globalvar Number score = 1;
rule: "damage" Event.OnDamageDealt if (score > 0) {
    score += 2;
}
"#,
    );
    assert!(
        diagnostics.iter().all(|diagnostic| !diagnostic.is_error()),
        "{diagnostics:?}"
    );
    program.validate().expect("structurally valid WIR");
    assert_eq!(program.global_variables.len(), 1);
    assert_eq!(
        program
            .global_variables
            .get(workshop_rs::wir::GlobalVarId::from_index(0))
            .unwrap()
            .index,
        0
    );
    assert_eq!(program.rules.len(), 2);
    let rule = program
        .rules
        .get(workshop_rs::wir::RuleId::from_index(1))
        .unwrap();
    assert!(matches!(
        rule.event,
        workshop_rs::wir::Event::Player {
            kind: workshop_rs::wir::PlayerEventKind::DealtDamage,
            ..
        }
    ));
    assert_eq!(rule.conditions.len(), 1);
    assert_eq!(rule.actions.len(), 1);
    assert!(rule.span.is_some());
    let name_span = rule.name_span.expect("rule name provenance");
    assert_eq!(name_span.file.index(), 0);
    assert_eq!(name_span.start.line, 3);
    assert_eq!(name_span.start.col, 8);
    assert_eq!(name_span.end.line, 3);
    assert_eq!(name_span.end.col, 14);
    assert_ne!(name_span, rule.span.unwrap());
    assert!(program.dump().contains("PlayerDealtDamage"));
}

#[test]
fn unsupported_rule_local_storage_is_structured_and_not_silently_dropped() {
    let (program, diagnostics) = lower(
        r#"
rule: "unsupported" Event.OngoingGlobal {
    define local = 1;
}
"#,
    );
    assert!(program.rules.is_empty());
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "HI018"));
}
