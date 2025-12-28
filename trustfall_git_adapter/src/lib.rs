use std::sync::LazyLock;

use trustfall::{
    Schema,
    provider::{Adapter, resolve_coercion_using_schema},
};

use crate::{types::Repository, vertex::Vertex};

mod edges;
mod properties;
mod types;
mod vertex;

static SCHEMA: LazyLock<Schema> =
    LazyLock::new(|| Schema::parse(include_str!("schema.graphql")).expect("schema not valid"));

pub struct GitAdapter<'a> {
    git2_repo: &'a git2::Repository,
}

impl<'a> GitAdapter<'a> {
    pub fn new(git2_repo: &'a git2::Repository) -> Self {
        GitAdapter { git2_repo }
    }

    pub fn schema(&self) -> &Schema {
        &SCHEMA
    }
}

impl<'a> Adapter<'a> for &'a GitAdapter<'a> {
    type Vertex = Vertex<'a>;

    fn resolve_starting_vertices(
        &self,
        edge_name: &std::sync::Arc<str>,
        _parameters: &trustfall_core::ir::EdgeParameters,
        _resolve_info: &trustfall::provider::ResolveInfo,
    ) -> trustfall::provider::VertexIterator<'a, Self::Vertex> {
        match edge_name.as_ref() {
            "repository" => {
                let repo_name = match self.git2_repo.find_remote("origin") {
                    Ok(remote) => remote.url().and_then(|url| {
                        url.trim_end_matches(".git")
                            .rsplit('/')
                            .next()
                            .map(|s| s.to_string())
                    }),
                    Err(_) => None,
                }
                .unwrap_or_else(|| {
                    // Fallback to directory name if no remote origin
                    self.git2_repo
                        .path()
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|name| name.to_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                });

                Box::new(std::iter::once(Vertex::Repository(Repository::new(
                    repo_name,
                ))))
            }
            _ => unreachable!("resolve_starting_vertices {edge_name}"),
        }
    }

    fn resolve_property<V: trustfall::provider::AsVertex<Self::Vertex> + 'a>(
        &self,
        contexts: trustfall::provider::ContextIterator<'a, V>,
        type_name: &std::sync::Arc<str>,
        property_name: &std::sync::Arc<str>,
        _resolve_info: &trustfall::provider::ResolveInfo,
    ) -> trustfall::provider::ContextOutcomeIterator<'a, V, trustfall::FieldValue> {
        match type_name.as_ref() {
            "Repository" => properties::resolve_repository_property(contexts, property_name),
            "Branch" => properties::resolve_branch_property(contexts, property_name),
            "Commit" => properties::resolve_commit_property(contexts, property_name),
            _ => unreachable!("resolve_property {type_name}"),
        }
    }

    fn resolve_neighbors<V: trustfall::provider::AsVertex<Self::Vertex> + 'a>(
        &self,
        contexts: trustfall::provider::ContextIterator<'a, V>,
        type_name: &std::sync::Arc<str>,
        edge_name: &std::sync::Arc<str>,
        _parameters: &trustfall_core::ir::EdgeParameters,
        _resolve_info: &trustfall::provider::ResolveEdgeInfo,
    ) -> trustfall::provider::ContextOutcomeIterator<
        'a,
        V,
        trustfall::provider::VertexIterator<'a, Self::Vertex>,
    > {
        match type_name.as_ref() {
            "Repository" => edges::resolve_repository_edge(self, contexts, edge_name),
            "Branch" => edges::resolve_branch_edge(self, contexts, edge_name),
            _ => unreachable!("resolve_neighbors {type_name}"),
        }
    }

    fn resolve_coercion<V: trustfall::provider::AsVertex<Self::Vertex> + 'a>(
        &self,
        contexts: trustfall::provider::ContextIterator<'a, V>,
        _type_name: &std::sync::Arc<str>,
        coerce_to_type: &std::sync::Arc<str>,
        _resolve_info: &trustfall::provider::ResolveInfo,
    ) -> trustfall::provider::ContextOutcomeIterator<'a, V, bool> {
        resolve_coercion_using_schema(contexts, &SCHEMA, coerce_to_type)
    }
}
