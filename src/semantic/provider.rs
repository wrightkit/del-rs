//! Workshop-facing name boundary: the single seam through which Workshop
//! catalog names enter the frontend (architecture §12). `del-rs` owns the
//! trait and the permissive default; `workshop-rs` implements a real provider
//! at integration time (#8). No catalog data lives here.

use crate::span::{FileId, Span};

/// Position a query name is used in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExternalPosition {
    Type,
    Value,
    Action,
    Event,
    Pattern,
}

/// A query for a Workshop-facing name.
#[derive(Clone, Debug)]
pub struct NameQuery {
    /// `[]` for bare names; `["Color"]` for `Color.SkyBlue`.
    pub namespace: Vec<String>,
    pub name: String,
    pub position: ExternalPosition,
    /// Argument count at the call site (0 for non-calls).
    pub arity: usize,
    pub span: Span,
}

/// The provider's verdict.
#[derive(Clone, Debug)]
pub enum ExternalResolution {
    Known(ExternalBinding),
    /// Unresolved-but-legal (default for `NoopProvider`).
    NotFound,
    /// The provider says this is definitively wrong.
    DefiniteError(String),
}

#[derive(Clone, Debug)]
pub enum ExternalBinding {
    Value(ExternalValueInfo),
    Action(ExternalActionInfo),
    Event(ExternalEventInfo),
    Type(ExternalTypeInfo),
    /// Qualified members exist (e.g. `Color.` prefix).
    Namespace,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExternalCategory {
    Number,
    String,
    Bool,
    Vector,
    Entity,
    Color,
    EnumLike,
    Constant,
    AnyLike,
}

#[derive(Clone, Debug)]
pub struct ExternalValueInfo {
    pub ty: Option<ExternalCategory>,
    pub signature: Option<ArgSignature>,
}

#[derive(Clone, Debug)]
pub struct ExternalActionInfo {
    pub params: Option<Vec<ExternalParam>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EventContext {
    Global,
    Player,
}

#[derive(Clone, Debug)]
pub struct ExternalEventInfo {
    pub context: Option<EventContext>,
}

#[derive(Clone, Debug)]
pub struct ExternalTypeInfo {
    pub category: ExternalCategory,
    pub constant: bool,
}

#[derive(Clone, Debug)]
pub struct ArgSignature {
    pub params: Vec<ExternalParam>,
}

#[derive(Clone, Debug)]
pub struct ExternalParam {
    pub name: String,
    pub optional: bool,
}

/// Permissive default: everything is `NotFound` (unresolved-but-legal).
pub struct NoopProvider;

impl NoopProvider {
    pub fn new() -> NoopProvider {
        NoopProvider
    }
}

impl Default for NoopProvider {
    fn default() -> Self {
        Self::new()
    }
}

pub trait WorkshopProvider: Send + Sync {
    fn resolve(&self, query: &NameQuery) -> ExternalResolution;
}

impl WorkshopProvider for NoopProvider {
    fn resolve(&self, _query: &NameQuery) -> ExternalResolution {
        ExternalResolution::NotFound
    }
}

/// File-agnostic placeholder context for provider calls (providers that need
/// program context can be given it at construction; the trait stays narrow).
pub struct ResolutionContext<'a> {
    pub file: FileId,
    pub provider_calls: &'a mut Vec<()>,
}
