use trustfall::{
    FieldValue,
    provider::{AsVertex, ContextIterator, ContextOutcomeIterator, resolve_property_with},
};
use trustfall_core::accessor_property;

use crate::vertex::Vertex;

pub(super) fn resolve_repository_property<'a, V: AsVertex<Vertex<'a>> + 'a>(
    contexts: ContextIterator<'a, V>,
    property_name: &str,
) -> ContextOutcomeIterator<'a, V, FieldValue> {
    match property_name {
        "name" => resolve_property_with(contexts, accessor_property!(as_repository, name)),
        _ => unreachable!("resolve_repository_property {property_name}"),
    }
}

pub(super) fn resolve_branch_property<'a, V: AsVertex<Vertex<'a>> + 'a>(
    contexts: ContextIterator<'a, V>,
    property_name: &str,
) -> ContextOutcomeIterator<'a, V, FieldValue> {
    match property_name {
        "name" => resolve_property_with(
            contexts,
            accessor_property!(as_branch, inner, {
                match inner.name() {
                    Ok(Some(name)) => name.into(),
                    _ => FieldValue::Null,
                }
            }),
        ),
        _ => unreachable!("resolve_branch_property {property_name}"),
    }
}

pub(super) fn resolve_commit_property<'a, V: AsVertex<Vertex<'a>> + 'a>(
    contexts: ContextIterator<'a, V>,
    property_name: &str,
) -> ContextOutcomeIterator<'a, V, FieldValue> {
    match property_name {
        "hash" => resolve_property_with(
            contexts,
            accessor_property!(as_commit, inner, { inner.id().to_string().into() }),
        ),
        "message" => resolve_property_with(
            contexts,
            accessor_property!(as_commit, inner, { inner.message().into() }),
        ),
        "author" => resolve_property_with(
            contexts,
            accessor_property!(as_commit, inner, { inner.author().name().into() }),
        ),
        "author_email" => resolve_property_with(
            contexts,
            accessor_property!(as_commit, inner, { inner.author().email().into() }),
        ),
        "committer" => resolve_property_with(
            contexts,
            accessor_property!(as_commit, inner, { inner.committer().name().into() }),
        ),
        "committer_email" => resolve_property_with(
            contexts,
            accessor_property!(as_commit, inner, { inner.committer().email().into() }),
        ),
        "date" => resolve_property_with(
            contexts,
            accessor_property!(as_commit, inner, {
                let time = inner.time();
                let utc_datetime = chrono::DateTime::from_timestamp(time.seconds(), 0).unwrap();
                let local_datetime = utc_datetime.with_timezone(&chrono::Local);
                local_datetime.to_rfc3339().into()
            }),
        ),
        _ => unreachable!("resolve_commit_property {property_name}"),
    }
}

pub(super) fn resolve_tag_property<'a, V: AsVertex<Vertex<'a>> + 'a>(
    contexts: ContextIterator<'a, V>,
    property_name: &str,
) -> ContextOutcomeIterator<'a, V, FieldValue> {
    match property_name {
        "name" => resolve_property_with(contexts, |vertex| {
            let tag = vertex.as_tag().expect("vertex was not a Tag");
            tag.name().into()
        }),
        "message" => resolve_property_with(contexts, |vertex| {
            let tag = vertex.as_tag().expect("vertex was not a Tag");
            tag.message().into()
        }),
        "tagger_name" => resolve_property_with(contexts, |vertex| {
            let tag = vertex.as_tag().expect("vertex was not a Tag");
            tag.tagger_name().into()
        }),
        "tagger_email" => resolve_property_with(contexts, |vertex| {
            let tag = vertex.as_tag().expect("vertex was not a Tag");
            tag.tagger_email().into()
        }),
        _ => unreachable!("resolve_tag_property {property_name}"),
    }
}
