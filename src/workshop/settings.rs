//! DEL/OSTW project settings carriers at the canonical Workshop boundary.

use crate::diagnostics::{error, Diagnostic, Phase};
use crate::project::{Project, SettingsImportKind};
use crate::span::Span;
use serde_json::Value;
use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::ids::Id;
use workshop_rs::settings::{Settings, SettingsListElement, SettingsNode};
use workshop_rs::source::{SourceFile, Span as WorkshopSpan};
use workshop_rs::wir;

pub(super) fn append_imports(
    program: &mut wir::Program,
    project: &Project,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if project.settings_imports.is_empty() {
        return;
    }

    let catalog = match Catalog::builtin() {
        Ok(catalog) => catalog,
        Err(cause) => {
            for import in &project.settings_imports {
                diagnostics.push(settings_error(
                    import.span,
                    format!("canonical Workshop catalog could not be loaded: {cause}"),
                ));
            }
            return;
        }
    };
    let locale = Locale::new("en-US");

    for import in &project.settings_imports {
        let source = project.sources.text(import.file);
        let (text, preserve_source_spans) = match import.kind {
            SettingsImportKind::Lobby => (source.to_owned(), true),
            SettingsImportKind::Json => match json_settings_text(source) {
                Ok(text) => (text, false),
                Err(cause) => {
                    diagnostics.push(settings_error(import.span, cause));
                    continue;
                }
            },
        };
        let mut settings = match workshop_rs::parser::parse(&text, &catalog, &locale) {
            Ok(parsed) => match parsed.settings {
                Some(settings) => settings,
                None => {
                    diagnostics.push(settings_error(
                        import.span,
                        "settings carrier did not contain a canonical settings section",
                    ));
                    continue;
                }
            },
            Err(cause) => {
                diagnostics.push(settings_error(
                    import.span,
                    format!("canonical Workshop settings parsing failed: {cause}"),
                ));
                continue;
            }
        };

        if preserve_source_spans {
            rebind_spans(&mut settings, import.file.0 as usize);
        } else {
            clear_spans(&mut settings);
        }
        merge_settings(
            program.settings.get_or_insert_with(empty_settings),
            settings,
        );
    }
}

fn empty_settings() -> Settings {
    Settings {
        span: None,
        children: Vec::new(),
    }
}

fn settings_error(span: Span, message: impl Into<String>) -> Diagnostic {
    error(Phase::Workshop, "WK003", span, message)
}

fn json_settings_text(source: &str) -> Result<String, String> {
    let root: Value = serde_json::from_str(source)
        .map_err(|cause| format!("invalid JSON settings carrier: {cause}"))?;
    let Value::Object(groups) = root else {
        return Err("JSON settings carrier must contain an object".to_string());
    };
    let mut out = String::from("settings {\n");
    for (name, value) in groups {
        render_json_member(&mut out, &canonical_group_name(&name), &value, 1)?;
    }
    out.push('}');
    Ok(out)
}

fn canonical_group_name(name: &str) -> String {
    match name {
        "Modes" => "modes".to_string(),
        "Heroes" => "heroes".to_string(),
        "Lobby" => "lobby".to_string(),
        "Main" => "main".to_string(),
        "Extensions" => "extensions".to_string(),
        "Workshop" => "workshop".to_string(),
        _ => name.to_string(),
    }
}

fn render_json_member(
    out: &mut String,
    name: &str,
    value: &Value,
    depth: usize,
) -> Result<(), String> {
    let name = if matches!(value, Value::Object(_)) {
        name.to_string()
    } else {
        name.to_ascii_lowercase()
    };
    let indent = "    ".repeat(depth);
    match value {
        Value::Object(children) => {
            out.push_str(&indent);
            out.push_str(&name);
            out.push_str(" {\n");
            for (child_name, child_value) in children {
                render_json_member(out, child_name, child_value, depth + 1)?;
            }
            out.push_str(&indent);
            out.push_str("}\n");
        }
        Value::Array(values) => {
            out.push_str(&indent);
            out.push_str(&name);
            out.push_str(" {\n");
            for value in values {
                let Value::String(value) = value else {
                    return Err(format!("settings list '{name}' must contain strings"));
                };
                if value.is_empty() || value.contains(['\n', '\r', '{', '}', ':', '"']) {
                    return Err(format!(
                        "settings list '{name}' contains an unsupported value"
                    ));
                }
                out.push_str(&"    ".repeat(depth + 1));
                out.push_str(value);
                out.push('\n');
            }
            out.push_str(&indent);
            out.push_str("}\n");
        }
        Value::Bool(value) => {
            out.push_str(&indent);
            out.push_str(&name);
            out.push_str(if *value { ": On\n" } else { ": Off\n" });
        }
        Value::Number(value) if value.is_f64() => {
            out.push_str(&indent);
            out.push_str(&name);
            out.push_str(": ");
            out.push_str(&value.to_string());
            out.push('\n');
        }
        Value::String(value) => {
            out.push_str(&indent);
            out.push_str(&name);
            out.push_str(": ");
            out.push_str(&serde_json::to_string(value).expect("JSON strings serialize"));
            out.push('\n');
        }
        Value::Null => return Err(format!("settings value '{name}' cannot be null")),
        Value::Number(_) => return Err(format!("settings value '{name}' is not finite")),
    }
    Ok(())
}

fn merge_settings(target: &mut Settings, incoming: Settings) {
    merge_children(&mut target.children, incoming.children);
}

fn merge_children(target: &mut Vec<SettingsNode>, incoming: Vec<SettingsNode>) {
    for node in incoming {
        if let Some(existing) = target
            .iter_mut()
            .find(|existing| existing.name() == node.name() && same_container_kind(existing, &node))
        {
            debug_assert!(merge_node(existing, node));
            continue;
        }
        target.push(node);
    }
}

fn same_container_kind(left: &SettingsNode, right: &SettingsNode) -> bool {
    matches!(
        (left, right),
        (SettingsNode::Group { .. }, SettingsNode::Group { .. })
            | (SettingsNode::Workshop { .. }, SettingsNode::Workshop { .. })
    )
}

fn merge_node(existing: &mut SettingsNode, incoming: SettingsNode) -> bool {
    match (existing, incoming) {
        (
            SettingsNode::Group {
                children: existing, ..
            },
            SettingsNode::Group {
                children: incoming, ..
            },
        )
        | (
            SettingsNode::Workshop {
                children: existing, ..
            },
            SettingsNode::Workshop {
                children: incoming, ..
            },
        ) => {
            merge_children(existing, incoming);
            true
        }
        _ => false,
    }
}

fn rebind_spans(settings: &mut Settings, file: usize) {
    settings.span = rebind_span(settings.span, file);
    for node in &mut settings.children {
        rebind_node_spans(node, file);
    }
}

fn rebind_node_spans(node: &mut SettingsNode, file: usize) {
    match node {
        SettingsNode::Workshop { children, span } | SettingsNode::Group { children, span, .. } => {
            *span = rebind_span(*span, file);
            for child in children {
                rebind_node_spans(child, file);
            }
        }
        SettingsNode::List { elements, span, .. } => {
            *span = rebind_span(*span, file);
            for SettingsListElement { span, .. } in elements {
                *span = rebind_span(*span, file);
            }
        }
        SettingsNode::Number { span, .. }
        | SettingsNode::Bool { span, .. }
        | SettingsNode::Flag { span, .. }
        | SettingsNode::String { span, .. }
        | SettingsNode::Raw { span, .. } => *span = rebind_span(*span, file),
    }
}

fn rebind_span(span: Option<WorkshopSpan>, file: usize) -> Option<WorkshopSpan> {
    span.map(|span| WorkshopSpan::new(Id::<SourceFile>::from_index(file), span.start, span.end))
}

fn clear_spans(settings: &mut Settings) {
    settings.span = None;
    for node in &mut settings.children {
        clear_node_spans(node);
    }
}

fn clear_node_spans(node: &mut SettingsNode) {
    match node {
        SettingsNode::Workshop { children, span } | SettingsNode::Group { children, span, .. } => {
            *span = None;
            for child in children {
                clear_node_spans(child);
            }
        }
        SettingsNode::List { elements, span, .. } => {
            *span = None;
            for element in elements {
                element.span = None;
            }
        }
        SettingsNode::Number { span, .. }
        | SettingsNode::Bool { span, .. }
        | SettingsNode::Flag { span, .. }
        | SettingsNode::String { span, .. }
        | SettingsNode::Raw { span, .. } => *span = None,
    }
}
