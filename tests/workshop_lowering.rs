//! Core DEL HIR -> canonical Workshop WIR lowering evidence for #30.

use del_rs::hir;
use del_rs::project::{load_project, ProjectOptions};
use del_rs::semantic::check_project;
use del_rs::semantic::provider::CatalogProvider;
use del_rs::workshop::{lower_project_to_wir, lower_to_wir};
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
    let (program, lowering_diags) = lower_project_to_wir(&semantic);
    diagnostics.extend(lowering_diags);
    (program, diagnostics)
}

fn lower_files(files: &[(&str, &str)]) -> (workshop_rs::wir::Program, Vec<del_rs::Diagnostic>) {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(10_000);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "del-rs-workshop-lowering-files-{}-{n}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    for (name, text) in files {
        std::fs::write(root.join(name), text).unwrap();
    }
    let project = load_project(ProjectOptions {
        root,
        entry: Some(PathBuf::from("main.del")),
        config: None,
    });
    let provider = CatalogProvider::new().expect("canonical catalog provider");
    let semantic = check_project(&project, &provider);
    let mut diagnostics = semantic.diagnostics.clone();
    let (program, lowering_diags) = lower_project_to_wir(&semantic);
    diagnostics.extend(lowering_diags);
    (program, diagnostics)
}

#[test]
fn hir_is_backend_neutral_and_hir_only_external_lowering_fails_closed() {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1000);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "del-rs-workshop-lowering-hir-only-{}-{n}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("main.del"),
        r#"rule: "damage" Event.OnDamageDealt { }
"#,
    )
    .unwrap();
    let project = load_project(ProjectOptions {
        root,
        entry: Some(PathBuf::from("main.del")),
        config: None,
    });
    let provider = CatalogProvider::new().expect("canonical catalog provider");
    let semantic = check_project(&project, &provider);
    let (hir, hir_diags) = hir::lower::lower(&semantic);
    assert!(
        hir_diags.iter().all(|diagnostic| !diagnostic.is_error()),
        "{hir_diags:?}"
    );

    let external = hir
        .exprs
        .iter()
        .find_map(|expr| match &expr.kind {
            hir::HirExprKind::External { name, namespace } => {
                Some((expr.span, name.as_str(), namespace.as_slice()))
            }
            _ => None,
        })
        .expect("HIR external reference");
    assert_eq!(external.1, "OnDamageDealt");
    assert_eq!(external.2, ["Event"]);
    assert!(!format!("{hir:?}").contains("ExternalBinding"));

    let (program, diagnostics) = lower_to_wir(&hir, &semantic.project.sources);
    assert!(program.rules.is_empty());
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "HI018" && diagnostic.primary == external.0));
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
    assert!(program
        .rules
        .get(workshop_rs::wir::RuleId::from_index(0))
        .and_then(|rule| rule.span)
        .is_some());
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

#[test]
fn core_control_flow_and_player_storage_lower_to_canonical_wir() {
    let (program, diagnostics) = lower(
        r#"
globalvar Number index = 0;
globalvar Number[] values = [1, 2];
playervar Number playerIndex;
rule: "flow" Event.OngoingGlobal {
    for (index = 0; index < 3; index = index + 1) {
        if (index == 1) { index += 2; }
    }
    for (index = 0; index < 2; index++) { }
    switch (index) {
        case 1: index += 1; break;
        case 2: index += 2;
        default: index = 0;
    }
    values = [3, 4];
}
rule: "player" Event.OngoingPlayer {
    playerIndex = 1;
}
"#,
    );
    assert!(
        diagnostics.iter().all(|diagnostic| !diagnostic.is_error()),
        "{diagnostics:?}"
    );
    program.validate().expect("structurally valid WIR");
    assert_eq!(program.global_variables.len(), 2);
    assert_eq!(program.player_variables.len(), 1);
    let dump = program.dump();
    assert!(dump.contains("while"), "{dump}");
    assert!(dump.contains("if"), "{dump}");
    assert!(dump.contains("modifyGlobalVariable index"), "{dump}");
    assert!(dump.contains("setPlayerVariable"), "{dump}");
    assert!(dump.contains("setGlobalVariable values"), "{dump}");
    let flow = program
        .rules
        .iter()
        .find(|rule| rule.name == "flow")
        .expect("flow rule");
    assert_eq!(flow.actions.len(), 6);
    assert!(matches!(
        program.actions.get(flow.actions[0]),
        Some(workshop_rs::wir::Action::SetGlobalVariable { .. })
    ));
    let workshop_rs::wir::Action::While { body, .. } =
        program.actions.get(flow.actions[1]).unwrap()
    else {
        panic!("classic for must lower to init plus while")
    };
    assert_eq!(
        body.len(),
        2,
        "while body must retain body and classic step"
    );
    assert!(matches!(
        program.actions.get(flow.actions[2]),
        Some(workshop_rs::wir::Action::SetGlobalVariable { .. })
    ));
    assert!(matches!(
        program.actions.get(flow.actions[3]),
        Some(workshop_rs::wir::Action::While { .. })
    ));
    let workshop_rs::wir::Action::If {
        branches,
        else_body,
        ..
    } = program.actions.get(flow.actions[4]).unwrap()
    else {
        panic!("switch must lower to canonical if branches")
    };
    assert_eq!(branches.len(), 2);
    assert_eq!(branches[0].body.len(), 1);
    assert_eq!(
        branches[1].body.len(),
        2,
        "case 2 must fall through to default"
    );
    assert_eq!(else_body.as_ref().map(Vec::len), Some(1));
}

#[test]
fn named_workshop_arguments_reorder_and_materialize_catalog_defaults() {
    let (program, diagnostics) = lower(
        r#"
rule: "message" Event.OngoingGlobal {
    SmallMessage(Header: "Hello");
}
"#,
    );
    assert!(
        diagnostics.iter().all(|diagnostic| !diagnostic.is_error()),
        "{diagnostics:?}"
    );
    let dump = program.dump();
    assert!(dump.contains("call smallMessage"), "{dump}");
    assert!(dump.contains("allPlayers(Team.ALL)"), "{dump}");
    assert!(dump.contains("\"Hello\""), "{dump}");
}

#[test]
fn missing_required_named_workshop_argument_fails_closed() {
    let (program, diagnostics) = lower(
        r#"
rule: "message" Event.OngoingGlobal {
    SmallMessage(VisibleTo: AllPlayers(Team.All));
}
"#,
    );
    assert!(program.rules.is_empty());
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "HI018" && diagnostic.message.contains("required")
        }),
        "{diagnostics:?}"
    );
}

#[test]
fn foreach_is_a_del_owned_runtime_gap_and_fails_closed() {
    let (program, diagnostics) = lower(
        r#"
globalvar Number[] values = [1];
rule: "foreach" Event.OngoingGlobal {
    foreach (Number value in values) { }
}
"#,
    );
    assert!(program.rules.is_empty());
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "HI018"
                && diagnostic.message.contains("DEL-owned")
                && diagnostic.message.contains("#31")
        }),
        "{diagnostics:?}"
    );
}

#[test]
fn stable_switch_scrutinee_is_lowered_without_materialization() {
    let (program, diagnostics) = lower(
        r#"
globalvar Number value = 1;
rule: "stable-switch" Event.OngoingGlobal {
    switch (value) {
        case 1: value = 2; break;
        default: value = 0;
    }
}
"#,
    );
    assert!(
        diagnostics.iter().all(|diagnostic| !diagnostic.is_error()),
        "{diagnostics:?}"
    );
    let rule = program
        .rules
        .iter()
        .find(|rule| rule.name == "stable-switch")
        .expect("stable switch rule");
    assert!(matches!(
        program.actions.get(rule.actions[0]),
        Some(workshop_rs::wir::Action::If { branches, .. }) if branches.len() == 1
    ));
}

#[test]
fn dynamic_switch_scrutinee_fails_closed_without_runtime_materialization() {
    let (program, diagnostics) = lower(
        r#"
globalvar Number value = 0;
rule: "dynamic-switch" Event.OngoingGlobal {
    switch (Add(1, 2)) {
        case 3: value = 1; break;
        default: value = 0;
    }
}
"#,
    );
    assert!(program.rules.is_empty());
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "HI018"
                && diagnostic.message.contains("single-evaluation")
                && diagnostic.message.contains("runtime materialization")
        }),
        "{diagnostics:?}"
    );
}

#[test]
fn variable_and_subroutine_allocation_is_deterministic_and_honors_reservations() {
    let source = r#"
globalvar Number explicit 2;
globalvar { "reserved", 0 };
void First() "First" { }
void Second() "Second" { First(); }
rule: "allocation" Event.OngoingGlobal { Second(); }
"#;
    let (first, first_diagnostics) = lower(source);
    let (second, second_diagnostics) = lower(source);
    assert!(
        first_diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "{first_diagnostics:?}"
    );
    assert!(
        second_diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "{second_diagnostics:?}"
    );
    assert_eq!(first.dump(), second.dump());
    assert_eq!(first.subroutines.len(), 2);
    assert!(first.dump().contains("callSubroutine"));
    assert_eq!(first.global_variables.len(), 2);
    assert_eq!(
        first
            .global_variables
            .get(workshop_rs::wir::GlobalVarId::from_index(0))
            .unwrap()
            .index,
        2
    );
    assert_eq!(
        first
            .global_variables
            .get(workshop_rs::wir::GlobalVarId::from_index(1))
            .unwrap()
            .index,
        0
    );
}

#[test]
fn global_rule_scalar_subroutine_parameters_materialize_in_source_order() {
    let (program, diagnostics) = lower(
        r#"
void First(Number amount, String label) "First" {
    amount += 1;
}
void Second(Number amount, String label) "Second" {
    amount++;
}
rule: "params" Event.OngoingGlobal {
    First(label: "one", amount: 2);
    Second(3, label: "two");
}
"#,
    );
    assert!(
        diagnostics.iter().all(|diagnostic| !diagnostic.is_error()),
        "{diagnostics:?}"
    );
    program.validate().expect("structurally valid WIR");
    assert_eq!(program.global_variables.len(), 4);
    assert_eq!(
        program
            .global_variables
            .iter()
            .map(|variable| variable.name.as_str())
            .collect::<Vec<_>>(),
        [
            "__del_param_f0_p0",
            "__del_param_f0_p1",
            "__del_param_f1_p0",
            "__del_param_f1_p1"
        ]
    );
    let rule = program
        .rules
        .iter()
        .find(|rule| rule.name == "params")
        .expect("parameter rule");
    assert_eq!(rule.actions.len(), 6);
    let first_id = program
        .subroutines
        .iter()
        .enumerate()
        .find(|(_, subroutine)| subroutine.name == "First")
        .map(|(index, _)| workshop_rs::wir::SubroutineId::from_index(index))
        .expect("First subroutine identity");
    let second_id = program
        .subroutines
        .iter()
        .enumerate()
        .find(|(_, subroutine)| subroutine.name == "Second")
        .map(|(index, _)| workshop_rs::wir::SubroutineId::from_index(index))
        .expect("Second subroutine identity");
    let expected_sets = [
        (
            0usize,
            1usize,
            workshop_rs::wir::Value::String("one".into()),
        ),
        (
            1,
            0,
            workshop_rs::wir::Value::Number {
                value: 2.0,
                text: "2".into(),
            },
        ),
        (
            3,
            2,
            workshop_rs::wir::Value::Number {
                value: 3.0,
                text: "3".into(),
            },
        ),
        (4, 3, workshop_rs::wir::Value::String("two".into())),
    ];
    for (action_index, slot_index, expected_value) in expected_sets {
        let workshop_rs::wir::Action::SetGlobalVariable {
            variable, value, ..
        } = program.actions.get(rule.actions[action_index]).unwrap()
        else {
            panic!("action {action_index} must materialize a parameter")
        };
        assert_eq!(
            *variable,
            workshop_rs::wir::GlobalVarId::from_index(slot_index)
        );
        match (&program.values.get(*value).unwrap().value, expected_value) {
            (
                workshop_rs::wir::Value::String(actual),
                workshop_rs::wir::Value::String(expected),
            ) => {
                assert_eq!(actual, &expected)
            }
            (
                workshop_rs::wir::Value::Number {
                    value: actual,
                    text: actual_text,
                },
                workshop_rs::wir::Value::Number {
                    value: expected,
                    text: expected_text,
                },
            ) => {
                assert_eq!(*actual, expected);
                assert_eq!(actual_text, &expected_text);
            }
            (actual, expected) => {
                panic!("unexpected materialized value: {actual:?} vs {expected:?}")
            }
        }
    }
    for (action_index, expected_subroutine) in [(2, first_id), (5, second_id)] {
        let workshop_rs::wir::Action::CallSubroutine { subroutine, .. } =
            program.actions.get(rule.actions[action_index]).unwrap()
        else {
            panic!("action {action_index} must call a subroutine")
        };
        assert_eq!(*subroutine, expected_subroutine);
    }
    assert!(program
        .global_variables
        .iter()
        .all(|variable| { variable.span.is_some() && variable.name_span.is_some() }));
    let parameter_spans = [(2, 19, 25), (2, 34, 39), (5, 20, 26), (5, 35, 40)];
    for (variable, (line, start, end)) in program.global_variables.iter().zip(parameter_spans) {
        let span = variable.span.unwrap();
        assert_eq!(
            (span.start.line, span.start.col, span.end.col),
            (line, start, end)
        );
        assert_eq!(variable.name_span, variable.span);
    }
    for (action, (line, start, end, target_line, target_start, target_end)) in [
        (rule.actions[0], (9, 18, 23, 2, 34, 39)),
        (rule.actions[1], (9, 33, 34, 2, 19, 25)),
        (rule.actions[3], (10, 12, 13, 5, 20, 26)),
        (rule.actions[4], (10, 22, 27, 5, 35, 40)),
    ] {
        let workshop_rs::wir::Action::SetGlobalVariable {
            span, target_span, ..
        } = program.actions.get(action).unwrap()
        else {
            panic!("expected parameter materialization action")
        };
        let span = span.unwrap();
        assert_eq!(
            (span.start.line, span.start.col, span.end.col),
            (line, start, end)
        );
        let target_span = target_span.unwrap();
        assert_eq!(
            (
                target_span.start.line,
                target_span.start.col,
                target_span.end.col
            ),
            (target_line, target_start, target_end)
        );
    }
    for (action, (line, start, end)) in [
        (rule.actions[2], (9, 5, 35)),
        (rule.actions[5], (10, 5, 28)),
    ] {
        let workshop_rs::wir::Action::CallSubroutine { span, .. } =
            program.actions.get(action).unwrap()
        else {
            panic!("expected subroutine call action")
        };
        let span = span.unwrap();
        assert_eq!(
            (span.start.line, span.start.col, span.end.col),
            (line, start, end)
        );
    }
    let first = program
        .rules
        .iter()
        .find(|rule| rule.name == "First")
        .expect("first subroutine rule");
    assert!(first.actions.iter().any(|action| matches!(
        program.actions.get(*action),
        Some(workshop_rs::wir::Action::ModifyGlobalVariable { variable, .. })
            if *variable == workshop_rs::wir::GlobalVarId::from_index(0)
    )));
    let second = program
        .rules
        .iter()
        .find(|rule| rule.name == "Second")
        .expect("second subroutine rule");
    assert!(second.actions.iter().any(|action| matches!(
        program.actions.get(*action),
        Some(workshop_rs::wir::Action::ModifyGlobalVariable { variable, .. })
            if *variable == workshop_rs::wir::GlobalVarId::from_index(2)
    )));
}

#[test]
fn parameter_runtime_rejects_player_recursive_nested_and_nonvoid_calls() {
    let cases = [
        (
            r#"
void Player(Number amount) playervar "Player" { }
rule: "player" Event.OngoingPlayer { Player(1); }
"#,
            "global Workshop rule",
        ),
        (
            r#"
recursive void Loop(Number amount) "Loop" { }
rule: "recursive" Event.OngoingGlobal { Loop(1); }
"#,
            "non-recursive",
        ),
        (
            r#"
void Inner(Number amount) "Inner" { }
void Outer(Number amount) "Outer" { Inner(amount); }
rule: "nested" Event.OngoingGlobal { Outer(1); }
"#,
            "nested subroutine",
        ),
        (
            r#"
Number Return(Number amount) "Return" { return amount; }
rule: "return" Event.OngoingGlobal { Return(1); }
"#,
            "return void",
        ),
        (
            r#"
void Ref(ref Number amount) "Ref" { }
rule: "ref" Event.OngoingGlobal { Ref(1); }
"#,
            "scalar value parameters",
        ),
        (
            r#"
void Array(Number[] amount) "Array" { }
rule: "array" Event.OngoingGlobal { Array([1]); }
"#,
            "scalar value parameters",
        ),
        (
            r#"
void Nested(Number amount) "Nested" { }
rule: "nested" Event.OngoingGlobal { if (true) { Nested(1); } }
"#,
            "direct global-rule actions",
        ),
        (
            r#"
Number Producer() "Producer" { return 1; }
void Target(Number amount) "Target" { }
rule: "side-effect" Event.OngoingGlobal { Target(Producer()); }
"#,
            "side-effect-free values",
        ),
    ];
    for (source, message) in cases {
        let (program, diagnostics) = lower(source);
        assert!(program.rules.is_empty(), "{source}");
        assert!(program.global_variables.is_empty(), "{source}");
        assert!(program.player_variables.is_empty(), "{source}");
        assert!(program.subroutines.is_empty(), "{source}");
        assert!(program.actions.is_empty(), "{source}");
        assert!(program.values.is_empty(), "{source}");
        assert!(program.files.is_empty(), "{source}");
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "HI018" && diagnostic.message.contains(message)
            }),
            "{source}\n{diagnostics:?}"
        );
    }
}

#[test]
fn parameter_slots_are_allocated_after_global_reservations() {
    let (program, diagnostics) = lower(
        r#"
globalvar Number explicit 2;
globalvar { "reserved", 0 };
void First(Number amount) "First" { }
rule: "reserved-params" Event.OngoingGlobal { First(1); }
"#,
    );
    assert!(
        diagnostics.iter().all(|diagnostic| !diagnostic.is_error()),
        "{diagnostics:?}"
    );
    assert_eq!(program.global_variables.len(), 3);
    assert_eq!(
        program
            .global_variables
            .get(workshop_rs::wir::GlobalVarId::from_index(0))
            .unwrap()
            .index,
        2
    );
    assert_eq!(
        program
            .global_variables
            .get(workshop_rs::wir::GlobalVarId::from_index(1))
            .unwrap()
            .index,
        0
    );
    assert_eq!(
        program
            .global_variables
            .get(workshop_rs::wir::GlobalVarId::from_index(2))
            .unwrap()
            .index,
        1
    );
}

#[test]
fn cross_file_lowering_preserves_source_provenance() {
    let (program, diagnostics) = lower_files(&[
        (
            "main.del",
            "import \"lib.del\";\nrule: \"main\" Event.OngoingGlobal { }\n",
        ),
        (
            "lib.del",
            "globalvar Number shared = 1;\nrule: \"library\" Event.OngoingGlobal { shared = 2; }\n",
        ),
    ]);
    assert!(
        diagnostics.iter().all(|diagnostic| !diagnostic.is_error()),
        "{diagnostics:?}"
    );
    let rule = program
        .rules
        .iter()
        .find(|rule| rule.name == "library")
        .expect("imported rule in WIR");
    assert_eq!(rule.span.expect("rule provenance").file.index(), 1);
    let action = program
        .actions
        .get(rule.actions[0])
        .expect("library action");
    assert_eq!(action.span().expect("action provenance").file.index(), 1);
}
