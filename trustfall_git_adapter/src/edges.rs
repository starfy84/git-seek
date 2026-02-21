use trustfall::provider::{
    AsVertex, ContextIterator, ContextOutcomeIterator, VertexIterator, resolve_neighbors_with,
};

use crate::{GitAdapter, types, vertex::Vertex};

pub(super) fn resolve_repository_edge<'a, V: AsVertex<Vertex<'a>> + 'a>(
    adapter: &'a GitAdapter<'a>,
    contexts: ContextIterator<'a, V>,
    edge_name: &str,
    parameters: &trustfall_core::ir::EdgeParameters,
) -> ContextOutcomeIterator<'a, V, VertexIterator<'a, Vertex<'a>>> {
    match edge_name {
        "commits" => {
            let limit = parameters.get("limit").map(|v| v.as_usize()).flatten();

            resolve_neighbors_with(contexts, move |_| {
                match adapter.git2_repo.revwalk().map(|mut revwalk| {
                    revwalk.push_head().expect("Could not push HEAD");

                    revwalk
                        .filter_map(|rev| {
                            rev.ok().and_then(|oid| {
                                adapter
                                    .git2_repo
                                    .find_commit(oid)
                                    .ok()
                                    .map(|commit| Vertex::Commit(types::Commit::new(commit)))
                            })
                        })
                        .take(limit.unwrap_or(usize::MAX))
                }) {
                    Ok(commits) => Box::new(commits),
                    Err(_) => Box::new(std::iter::empty()),
                }
            })
        }
        "branches" => resolve_neighbors_with(contexts, |_| {
            let filter = git2::BranchType::Local;
            match adapter.git2_repo.branches(Some(filter)) {
                Ok(branches) => {
                    let branch_vertices = branches.filter_map(|branch_result| {
                        branch_result
                            .ok()
                            .map(|(branch, _)| Vertex::Branch(types::Branch::new(branch)))
                    });

                    Box::new(branch_vertices)
                }
                Err(_) => Box::new(std::iter::empty()),
            }
        }),
        "tags" => resolve_neighbors_with(contexts, |_| {
            match adapter.git2_repo.tag_names(None) {
                Ok(tag_names) => {
                    let tags: Vec<_> = tag_names
                        .iter()
                        .flatten()
                        .filter_map(|name| {
                            let refname = format!("refs/tags/{}", name);
                            let reference = adapter.git2_repo.find_reference(&refname).ok()?;

                            // Try to peel to a tag object (annotated tag)
                            let (target_oid, message, tagger_name, tagger_email) =
                                if let Ok(tag_obj) = reference.peel_to_tag() {
                                    let msg = tag_obj.message().map(|m| m.to_string());
                                    let t_name = tag_obj
                                        .tagger()
                                        .and_then(|t| t.name().map(|n| n.to_string()));
                                    let t_email = tag_obj
                                        .tagger()
                                        .and_then(|t| t.email().map(|e| e.to_string()));
                                    let oid = tag_obj
                                        .target()
                                        .ok()
                                        .and_then(|obj| obj.into_commit().ok())
                                        .map(|c| c.id())?;
                                    (oid, msg, t_name, t_email)
                                } else {
                                    // Lightweight tag — points directly to a commit
                                    let oid = reference.peel_to_commit().ok()?.id();
                                    (oid, None, None, None)
                                };

                            Some(Vertex::Tag(types::Tag::new(
                                name.to_string(),
                                target_oid,
                                message,
                                tagger_name,
                                tagger_email,
                            )))
                        })
                        .collect();

                    Box::new(tags.into_iter()) as VertexIterator<'a, Vertex>
                }
                Err(_) => Box::new(std::iter::empty()),
            }
        }),
        _ => unreachable!("resolve_repository_edge {edge_name}"),
    }
}

pub(super) fn resolve_branch_edge<'a, V: AsVertex<Vertex<'a>> + 'a>(
    adapter: &'a GitAdapter<'a>,
    contexts: ContextIterator<'a, V>,
    edge_name: &str,
) -> ContextOutcomeIterator<'a, V, VertexIterator<'a, Vertex<'a>>> {
    match edge_name {
        "commit" => resolve_neighbors_with(contexts, |vertex| {
            let branch = vertex.as_branch().expect("vertex was not a Branch");

            match branch.inner().name() {
                Ok(Some(name)) => adapter
                    .git2_repo
                    .find_branch(name, git2::BranchType::Local)
                    .ok()
                    .and_then(|git2_branch| git2_branch.get().target())
                    .and_then(|oid| adapter.git2_repo.find_commit(oid).ok())
                    .map(|commit| {
                        Box::new(std::iter::once(Vertex::Commit(types::Commit::new(commit))))
                            as VertexIterator<'a, Vertex>
                    })
                    .unwrap_or_else(|| Box::new(std::iter::empty()) as VertexIterator<'a, Vertex>),
                _ => Box::new(std::iter::empty()),
            }
        }),
        _ => unreachable!("resolve_branch_edge {edge_name}"),
    }
}

pub(super) fn resolve_tag_edge<'a, V: AsVertex<Vertex<'a>> + 'a>(
    adapter: &'a GitAdapter<'a>,
    contexts: ContextIterator<'a, V>,
    edge_name: &str,
) -> ContextOutcomeIterator<'a, V, VertexIterator<'a, Vertex<'a>>> {
    match edge_name {
        "commit" => resolve_neighbors_with(contexts, |vertex| {
            let tag = vertex.as_tag().expect("vertex was not a Tag");
            let oid = tag.target_oid();

            match adapter.git2_repo.find_commit(oid) {
                Ok(commit) => Box::new(std::iter::once(Vertex::Commit(types::Commit::new(commit))))
                    as VertexIterator<'a, Vertex>,
                Err(_) => Box::new(std::iter::empty()),
            }
        }),
        _ => unreachable!("resolve_tag_edge {edge_name}"),
    }
}
