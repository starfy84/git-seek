use git2::Repository;
use std::sync::Arc;
use tempfile::TempDir;
use trustfall_git_adapter::GitAdapter;

fn create_test_repo() -> (TempDir, Repository) {
    let temp_dir = TempDir::new().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();

    {
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();
    }

    let signature = git2::Signature::now("Test User", "test@example.com").unwrap();
    let tree_id = {
        let mut index = repo.index().unwrap();
        index.write_tree().unwrap()
    };

    {
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )
        .unwrap();
    }

    (temp_dir, repo)
}

#[test]
fn test_adapter_creation() {
    let (_temp_dir, repo) = create_test_repo();
    let adapter = GitAdapter::new(&repo);
    let _schema = adapter.schema();

    // Just verify the adapter can be created and has a schema
}

#[test]
fn test_query_repository_name() {
    let (_temp_dir, repo) = create_test_repo();
    let adapter = GitAdapter::new(&repo);

    let query = r#"
    {
        repository {
            name @output
        }
    }
    "#;

    let variables: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    let results: Vec<_> =
        trustfall::execute_query(adapter.schema(), Arc::new(&adapter), query, variables)
            .unwrap()
            .collect();

    assert_eq!(results.len(), 1);
    assert!(results[0].contains_key("name"));
}

#[test]
fn test_query_repository_branches() {
    let (_temp_dir, repo) = create_test_repo();
    let adapter = GitAdapter::new(&repo);

    let query = r#"
    {
        repository {
            branches {
                name @output
            }
        }
    }
    "#;

    let variables: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    let results: Vec<_> =
        trustfall::execute_query(adapter.schema(), Arc::new(&adapter), query, variables)
            .unwrap()
            .collect();

    // Should have at least the main/master branch
    assert!(!results.is_empty());

    let branch_names: Vec<_> = results
        .iter()
        .filter_map(|row| {
            if let Some(trustfall::FieldValue::String(name)) = row.get("name") {
                Some(name.as_ref())
            } else {
                None
            }
        })
        .collect();

    assert!(branch_names.contains(&"main") || branch_names.contains(&"master"));
}
