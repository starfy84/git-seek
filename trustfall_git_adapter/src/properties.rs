use trustfall::{FieldValue, provider::{AsVertex, ContextIterator, ContextOutcomeIterator, resolve_property_with}};
use trustfall_core::accessor_property;

use crate::vertex::Vertex;

pub(super) fn resolve_repository_property<'a, V: AsVertex<Vertex<'a>> + 'a>(
    contexts: ContextIterator<'a, V>,
    property_name: &str,
) -> ContextOutcomeIterator<'a, V, FieldValue> {
    match property_name {
        "name" => {
            resolve_property_with(contexts, accessor_property!(as_repository, name))
        },
        _ => unreachable!("resolve_repository_property {property_name}"),
    }
}

pub(super) fn resolve_branch_property<'a, V: AsVertex<Vertex<'a>> + 'a>(
    contexts: ContextIterator<'a, V>,
    property_name: &str,
) -> ContextOutcomeIterator<'a, V, FieldValue> {
    match property_name {
        "name" => {
            resolve_property_with(contexts, accessor_property!(as_branch, inner, {
                match inner.name() {
                    Ok(Some(name)) => name.into(),
                    _ => FieldValue::Null,
                }
            }))
        },
        _ => unreachable!("resolve_branch_property {property_name}"),
    }
}

pub(super) fn resolve_commit_property<'a, V: AsVertex<Vertex<'a>> + 'a>(
    contexts: ContextIterator<'a, V>,
    property_name: &str,
) -> ContextOutcomeIterator<'a, V, FieldValue> {
    match property_name {
        "hash" => {
            resolve_property_with(contexts, accessor_property!(as_commit, inner, {
                inner.id().to_string().into()
            }))
        },
        "message" => {
            resolve_property_with(contexts, accessor_property!(as_commit, inner, {
                inner.message().into()
            }))
        }
        _ => unreachable!("resolve_commit_property {property_name}"),
    }
}