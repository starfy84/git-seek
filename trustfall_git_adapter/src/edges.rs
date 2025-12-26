use trustfall::provider::{AsVertex, ContextIterator, ContextOutcomeIterator, VertexIterator, resolve_neighbors_with};

use crate::{GitAdapter, vertex::Vertex, types};

pub(super) fn resolve_repository_edge<'a, V: AsVertex<Vertex> + 'a>(
    adapter: &'a GitAdapter<'a>,
    contexts: ContextIterator<'a, V>,
    edge_name: &str,
) -> ContextOutcomeIterator<'a, V, VertexIterator<'a, Vertex>> {
    match edge_name {
        "commits" => resolve_neighbors_with(contexts, |_| {
            match adapter.git2_repo.revwalk().map(|mut revwalk| {
                revwalk.push_head().expect("Could not push HEAD");

                revwalk.filter_map(|rev| {
                    rev.ok().and_then(|oid| {
                        adapter.git2_repo.find_commit(oid).ok().map(|commit| {
                            Vertex::Commit(types::Commit::new(oid.to_string(), commit.message().map(|m| m.to_owned())))
                        })
                    })
                })
            }) {
                Ok(commits) => Box::new(commits),
                Err(_) => Box::new(std::iter::empty()),
            }
        }),
        "branches" => resolve_neighbors_with(contexts, |_| {
            let filter = git2::BranchType::Local;
            match adapter.git2_repo.branches(Some(filter)) {
                Ok(branches) => {
                    let branch_vertices = branches.filter_map(|branch_result| {
                        branch_result.ok().and_then(|(branch, _)| {
                            branch.name().ok().flatten().map(|name| {
                                Vertex::Branch(types::Branch::new(name.to_string()))
                            })
                        })
                    });

                    Box::new(branch_vertices)
                },
                Err(_) => Box::new(std::iter::empty()),
            }
        }),
        _ => unreachable!("resolve_repository_edge {edge_name}"),
    }
}

pub(super) fn resolve_branch_edge<'a, V: AsVertex<Vertex> + 'a>(
    adapter: &'a GitAdapter<'a>,
    contexts: ContextIterator<'a, V>,
    edge_name: &str,
) -> ContextOutcomeIterator<'a, V, VertexIterator<'a, Vertex>> {
    match edge_name {
        "commit" => resolve_neighbors_with(contexts, |vertex| {
            let branch = vertex.as_branch().expect("vertex was not a Branch");

            adapter.git2_repo.find_branch(branch.name(), git2::BranchType::Local).ok()
                .and_then(|git2_branch| git2_branch.get().target())
                .and_then(|oid| adapter.git2_repo.find_commit(oid).ok())
                .map(|commit| {
                    Box::new(std::iter::once(Vertex::Commit(types::Commit::new(
                        commit.id().to_string(),
                        commit.message().map(|m| m.to_owned()),
                    )))) as VertexIterator<'a, Vertex>
                })
                .unwrap_or_else(|| Box::new(std::iter::empty()) as VertexIterator<'a, Vertex>)
        }),
        _ => unreachable!("resolve_branch_edge {edge_name}"),
    }
}