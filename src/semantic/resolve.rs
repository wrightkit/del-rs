//! Name resolution results shared between the checker and the API.

use crate::semantic::provider::ExternalBinding;
use crate::semantic::symbols::SymbolId;
use crate::semantic::types::Type;

#[derive(Clone, Debug)]
pub enum Resolution {
    /// Bound to a declared symbol.
    Symbol(SymbolId),
    /// A primitive type name used as a value (e.g. `EnumTest.A` static path).
    PrimitiveType(Type),
    /// `root` (project scope marker).
    Root,
    /// `this`.
    This,
    /// Provider-resolved (Known).
    External(ExternalBinding),
    /// Provider NotFound — unresolved-but-legal.
    UnresolvedExternal,
    /// Language-owned builtin member (array members, `.Key`, `.Invoke`).
    BuiltinMember(BuiltinMember),
    /// Playervar access through a player expression.
    PlayervarAccess(SymbolId),
    /// Genuine error (SM code emitted).
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuiltinMember {
    ArrayLength,
    ArrayIndexOf,
    ArrayMap,
    ArrayFilteredArray,
    ArrayRandom,
    ArrayFirst,
    ArrayModAppend,
    ArrayModRemoveByIndex,
    ArrayAppend,
    ArrayLast,
    ArrayContains,
    ArraySortedArray,
    ArrayIsTrueForAll,
    ArrayIsTrueForAny,
    Key,
    Invoke,
}
