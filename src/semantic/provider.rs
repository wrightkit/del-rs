//! Workshop-facing name boundary: the single seam through which Workshop
//! catalog names enter the frontend (architecture §12). `del-rs` owns the
//! trait, the permissive default, and the catalog-backed source-language
//! adapter; canonical catalog data remains in `workshop-rs`.

use crate::span::{FileId, Span};
use workshop_rs::catalog::{Catalog, CatalogEntry, Kind, Locale};
use workshop_rs::WorkshopError;

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
    /// Canonical Workshop identity, when the provider resolved one.
    pub canonical_id: String,
    pub ty: Option<ExternalCategory>,
    pub signature: Option<ArgSignature>,
}

#[derive(Clone, Debug)]
pub struct ExternalActionInfo {
    /// Canonical Workshop action identity.
    pub canonical_id: String,
    pub params: Option<Vec<ExternalParam>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EventContext {
    Global,
    Player,
}

#[derive(Clone, Debug)]
pub struct ExternalEventInfo {
    /// Canonical Workshop event identity.
    pub canonical_id: String,
    pub context: Option<EventContext>,
}

#[derive(Clone, Debug)]
pub struct ExternalTypeInfo {
    /// Canonical Workshop type/domain identity.
    pub canonical_id: String,
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

/// A catalog-backed provider for the Workshop integration boundary.
///
/// The provider owns only the source-language adapter. Canonical Workshop
/// identities, aliases, parameter metadata, enum domains, and provenance are
/// read from `workshop-rs`; no catalog data is duplicated here.
pub struct CatalogProvider {
    catalog: Catalog,
    locale: Locale,
}

impl CatalogProvider {
    /// Load the built-in catalog from the released `workshop-rs` dependency.
    pub fn new() -> Result<Self, WorkshopError> {
        Self::from_catalog(Catalog::builtin()?, Locale::new("en-US"))
    }

    /// Build a provider from a caller-supplied canonical catalog and locale.
    pub fn from_catalog(catalog: Catalog, locale: Locale) -> Result<Self, WorkshopError> {
        if !catalog.supports(&locale) {
            return Err(WorkshopError::Unsupported {
                message: format!(
                    "provider locale '{}' is not declared by the catalog",
                    locale
                ),
                span: None,
            });
        }
        Ok(Self { catalog, locale })
    }

    /// The canonical catalog identity consumed by this provider.
    pub fn catalog_identity(&self) -> workshop_rs::catalog::CatalogIdentity {
        self.catalog.identity()
    }

    /// The provider's catalog and locale, for integration/lowering clients.
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub fn locale(&self) -> &Locale {
        &self.locale
    }

    fn resolve_entry(&self, kind: Kind, name: &str) -> Option<&CatalogEntry> {
        self.catalog
            .entry(kind, name)
            .or_else(|| self.catalog.resolve(kind, &self.locale, name))
            .or_else(|| {
                let normalized = lowercase_first(name);
                self.catalog.entry(kind, &normalized)
            })
    }

    fn resolve_event(&self, name: &str) -> Option<(&CatalogEntry, String)> {
        // The spelling bridge is DEL's source contract; the identity remains
        // owned by the canonical catalog.
        let canonical = match name {
            "OngoingGlobal" => "global",
            "OngoingPlayer" => "eachPlayer",
            "Subroutine" => "subroutine",
            _ => name,
        };
        self.catalog
            .entry(Kind::Event, canonical)
            .map(|entry| (entry, canonical.to_string()))
            .or_else(|| {
                self.catalog
                    .resolve(Kind::Event, &self.locale, name)
                    .map(|entry| (entry, entry.id.clone()))
            })
    }

    fn resolve_enum_member(&self, namespace: &[String], name: &str) -> Option<(String, String)> {
        let domain = namespace.first()?;
        let canonical_name = lowercase_first(name);
        self.catalog
            .enum_domain(domain)
            .and_then(|_| {
                self.catalog
                    .resolve_enum_member(domain, &self.locale, name)
                    .or_else(|| {
                        self.catalog
                            .resolve_enum_member(domain, &self.locale, &canonical_name)
                    })
            })
            .or_else(|| {
                self.catalog
                    .enum_domain(domain)
                    .and_then(|d| {
                        d.members.iter().find(|m| {
                            m.member == name
                                || m.member == canonical_name
                                || m.member.eq_ignore_ascii_case(name)
                        })
                    })
                    .map(|m| (domain.to_string(), m.member.clone()))
            })
    }
}

impl WorkshopProvider for CatalogProvider {
    fn resolve(&self, query: &NameQuery) -> ExternalResolution {
        if query.namespace.first().map(String::as_str) == Some("Event") {
            let Some((entry, canonical_id)) = self.resolve_event(&query.name) else {
                return ExternalResolution::NotFound;
            };
            let context = match canonical_id.as_str() {
                "global" => Some(EventContext::Global),
                "eachPlayer" => Some(EventContext::Player),
                _ => None,
            };
            return ExternalResolution::Known(ExternalBinding::Event(ExternalEventInfo {
                canonical_id: entry.id.clone(),
                context,
            }));
        }

        if !query.namespace.is_empty() {
            if let Some((domain, member)) = self.resolve_enum_member(&query.namespace, &query.name)
            {
                return ExternalResolution::Known(ExternalBinding::Value(ExternalValueInfo {
                    canonical_id: format!("{domain}.{member}"),
                    ty: Some(ExternalCategory::EnumLike),
                    signature: None,
                }));
            }
        }

        let kinds: &[Kind] = match query.position {
            ExternalPosition::Action => &[Kind::Action],
            ExternalPosition::Event => &[Kind::Event],
            ExternalPosition::Type => &[Kind::Enum],
            // The existing DEL semantic contract presents both value and
            // action calls through the value position; preserve that seam by
            // asking the canonical catalog for both kinds in order.
            ExternalPosition::Value | ExternalPosition::Pattern => &[Kind::Value, Kind::Action],
        };
        if let Some((kind, entry)) = kinds.iter().find_map(|kind| {
            self.resolve_entry(*kind, &query.name)
                .map(|entry| (*kind, entry))
        }) {
            if query.arity > entry.params.len() {
                return ExternalResolution::DefiniteError(format!(
                    "Workshop {} '{}' accepts at most {} arguments, got {}",
                    kind.as_str(),
                    entry.id,
                    entry.params.len(),
                    query.arity
                ));
            }
            return match kind {
                Kind::Action => {
                    ExternalResolution::Known(ExternalBinding::Action(ExternalActionInfo {
                        canonical_id: entry.id.clone(),
                        params: Some(parameters(entry)),
                    }))
                }
                Kind::Value => {
                    ExternalResolution::Known(ExternalBinding::Value(ExternalValueInfo {
                        canonical_id: entry.id.clone(),
                        ty: None,
                        signature: Some(ArgSignature {
                            params: parameters(entry),
                        }),
                    }))
                }
                Kind::Enum => ExternalResolution::Known(ExternalBinding::Type(ExternalTypeInfo {
                    canonical_id: entry.id.clone(),
                    category: ExternalCategory::EnumLike,
                    constant: true,
                })),
                Kind::Event => {
                    ExternalResolution::Known(ExternalBinding::Event(ExternalEventInfo {
                        canonical_id: entry.id.clone(),
                        context: None,
                    }))
                }
                _ => ExternalResolution::NotFound,
            };
        }

        if query.namespace.is_empty()
            && matches!(
                query.position,
                ExternalPosition::Value | ExternalPosition::Pattern
            )
        {
            let matches = self.catalog.bare_member_matches(&self.locale, &query.name);
            if matches.len() == 1 {
                let (domain, member) = &matches[0];
                return ExternalResolution::Known(ExternalBinding::Value(ExternalValueInfo {
                    canonical_id: format!("{domain}.{member}"),
                    ty: Some(ExternalCategory::EnumLike),
                    signature: None,
                }));
            }
            if matches.len() > 1 {
                return ExternalResolution::DefiniteError(format!(
                    "ambiguous Workshop enum member '{}' (matches {})",
                    query.name,
                    matches
                        .iter()
                        .map(|(domain, member)| format!("{domain}.{member}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        ExternalResolution::NotFound
    }
}

fn lowercase_first(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_lowercase().collect::<String>() + chars.as_str()
}

fn parameters(entry: &CatalogEntry) -> Vec<ExternalParam> {
    entry
        .params
        .iter()
        .enumerate()
        .map(|(index, name)| ExternalParam {
            name: name.clone(),
            optional: entry
                .param_defaults
                .get(index)
                .and_then(Option::as_ref)
                .is_some(),
        })
        .collect()
}

/// File-agnostic placeholder context for provider calls (providers that need
/// program context can be given it at construction; the trait stays narrow).
pub struct ResolutionContext<'a> {
    pub file: FileId,
    pub provider_calls: &'a mut Vec<()>,
}
