use trustfall_derive::TrustfallEnumVertex;

use crate::types;

#[derive(Debug, Clone, TrustfallEnumVertex)]
pub enum Vertex<'a> {
    Repository(types::Repository),
    Commit(types::Commit<'a>),
    Branch(types::Branch<'a>),
}