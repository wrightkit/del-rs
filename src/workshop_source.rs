//! Source provenance bridge for the DEL -> Workshop integration boundary.
//!
//! This module converts DEL source identity and byte spans into the canonical
//! `workshop-rs` source model. It does not lower HIR or define Workshop
//! semantics; those responsibilities remain in the later lowering layer and
//! in `workshop-rs`, respectively.

use crate::span::{SourceMap, Span};
use workshop_rs::ids::Id;
use workshop_rs::source::{Position, SourceFile, Span as WorkshopSpan};

/// Deterministic mapping from DEL source files to Workshop source files.
#[derive(Debug, Clone)]
pub struct WorkshopSourceMap {
    files: Vec<SourceFile>,
}

impl WorkshopSourceMap {
    /// Copy the DEL source registry in its stable file order.
    pub fn from_source_map(sources: &SourceMap) -> Self {
        Self {
            files: sources
                .files()
                .map(|source| SourceFile::new(source.name.display().to_string()))
                .collect(),
        }
    }

    /// Workshop source files in the same order as the DEL source registry.
    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }

    /// Convert a DEL byte span into a validated 1-based Workshop span.
    pub fn span(&self, sources: &SourceMap, span: Span) -> Option<WorkshopSpan> {
        let file = sources.get(span.file);
        if span.end < span.start || span.end > file.text.len() as u32 {
            return None;
        }
        let file_id = Id::from_index(span.file.0 as usize);
        if file_id.index() >= self.files.len() {
            return None;
        }
        Some(WorkshopSpan::new(
            file_id,
            position(file.line_col(span.start)),
            position(file.line_col(span.end)),
        ))
    }
}

fn position(line_col: crate::span::LineCol) -> Position {
    Position::new(line_col.line, line_col.col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn preserves_file_order_and_exact_one_based_unicode_spans() {
        let mut sources = SourceMap::new();
        let first = sources.add_file(PathBuf::from("main.del"), "éx\nrule".into());
        let second = sources.add_file(PathBuf::from("lib.del"), "ok".into());
        let bridge = WorkshopSourceMap::from_source_map(&sources);

        assert_eq!(bridge.files()[0].path, "main.del");
        assert_eq!(bridge.files()[1].path, "lib.del");

        let converted = bridge
            .span(&sources, Span::new(first, 0, 3))
            .expect("valid source span");
        assert_eq!(converted.file.index(), first.0 as usize);
        assert_eq!(converted.start, Position::new(1, 1));
        assert_eq!(converted.end, Position::new(1, 3));

        let second_span = bridge
            .span(&sources, Span::new(second, 0, 2))
            .expect("second source span");
        assert_eq!(second_span.file.index(), second.0 as usize);
        assert_eq!(second_span.start, Position::new(1, 1));
        assert_eq!(second_span.end, Position::new(1, 3));
    }

    #[test]
    fn rejects_out_of_range_spans() {
        let mut sources = SourceMap::new();
        let file = sources.add_file(PathBuf::from("main.del"), "ok".into());
        let bridge = WorkshopSourceMap::from_source_map(&sources);
        assert!(bridge.span(&sources, Span::new(file, 1, 3)).is_none());
        assert!(bridge.span(&sources, Span::new(file, 2, 1)).is_none());
    }
}
