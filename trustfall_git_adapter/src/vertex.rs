use trustfall_derive::TrustfallEnumVertex;

use crate::types;

#[derive(Debug, Clone, TrustfallEnumVertex)]
pub enum Vertex {
    Repository(types::Repository),
    Commit(types::Commit),
    Branch(types::Branch),
}