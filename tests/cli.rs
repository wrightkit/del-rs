//! CLI smoke tests: exit codes and --json schema (architecture §18).

use std::process::Command;

fn del_rs_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_del-rs"))
}

fn sample() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/highlevel/enum-basic.del")
}

#[test]
fn parse_ok_exit_zero() {
    let out = del_rs_bin().arg("parse").arg(sample()).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("parsed"), "stdout: {stdout}");
}

#[test]
fn parse_json_schema() {
    let out = del_rs_bin()
        .args(["parse", "--json"])
        .arg(sample())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["command"], "parse");
    assert!(doc["diagnostics"].is_array());
    assert!(doc["summary"]["items"].is_number());
}

#[test]
fn check_full_pipeline() {
    let out = del_rs_bin().arg("check").arg(sample()).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn hir_command() {
    let out = del_rs_bin().arg("hir").arg(sample()).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("rules"), "stdout: {stdout}");
}

#[test]
fn inspect_query() {
    let out = del_rs_bin()
        .args(["inspect"])
        .arg(sample())
        .arg("5:14")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn matrix_check_exit_zero() {
    let out = del_rs_bin().args(["matrix", "--check"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn compatibility_json_report() {
    let out = del_rs_bin().args(["compatibility", "--json"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["schema"], 1);
    assert!(doc["summary"]["matched"].is_number());
    assert!(doc["summary"]["known_gaps"].is_number());
    assert!(doc["summary"]["unexpected_regressions"].is_number());
}

#[test]
fn usage_error_exit_two() {
    let out = del_rs_bin().arg("bogus-command").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn missing_file_exit_four() {
    let out = del_rs_bin()
        .args(["parse", "/nonexistent/path/nowhere.del"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
}

#[test]
fn errors_exit_one() {
    let bad = std::env::temp_dir().join(format!("del-rs-cli-bad-{}", std::process::id()));
    std::fs::write(&bad, "rule: \"\" {\n    this;\n}\n").unwrap();
    let out = del_rs_bin().arg("check").arg(&bad).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    let _ = std::fs::remove_file(&bad);
}
