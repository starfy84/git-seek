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

fn create_test_repo_with_multiple_commits() -> (TempDir, Repository) {
    let temp_dir = TempDir::new().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();

    {
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();
    }

    let signature = git2::Signature::now("Test User", "test@example.com").unwrap();
    let author_signature = git2::Signature::now("Author User", "author@example.com").unwrap();

    // Create initial commit
    let first_commit = {
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            Some("HEAD"),
            &author_signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )
        .unwrap()
    };

    // Create a second commit
    {
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        let first_commit_obj = repo.find_commit(first_commit).unwrap();
        repo.commit(
            Some("HEAD"),
            &author_signature,
            &signature,
            "Second commit with more details",
            &tree,
            &[&first_commit_obj],
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

#[test]
fn test_query_repository_commits() {
    let (_temp_dir, repo) = create_test_repo_with_multiple_commits();
    let adapter = GitAdapter::new(&repo);

    let query = r#"
    {
        repository {
            commits {
                hash @output
            }
        }
    }
    "#;

    let variables: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    let results: Vec<_> =
        trustfall::execute_query(adapter.schema(), Arc::new(&adapter), query, variables)
            .unwrap()
            .collect();

    // Should have at least 2 commits (initial + second)
    assert!(results.len() >= 2);

    // Verify all results have hash field
    for result in &results {
        assert!(result.contains_key("hash"));
        if let Some(trustfall::FieldValue::String(hash)) = result.get("hash") {
            // Git hashes should be 40 characters
            assert_eq!(hash.len(), 40);
            // Should only contain hexadecimal characters
            assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        } else {
            panic!("Hash field should be a string");
        }
    }
}

#[test]
fn test_query_commit_properties() {
    let (_temp_dir, repo) = create_test_repo_with_multiple_commits();
    let adapter = GitAdapter::new(&repo);

    let query = r#"
    {
        repository {
            commits {
                hash @output
                message @output
                author @output
            }
        }
    }
    "#;

    let variables: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    let results: Vec<_> =
        trustfall::execute_query(adapter.schema(), Arc::new(&adapter), query, variables)
            .unwrap()
            .collect();

    assert!(!results.is_empty());

    for result in &results {
        // Verify hash field
        assert!(result.contains_key("hash"));
        if let Some(trustfall::FieldValue::String(hash)) = result.get("hash") {
            assert_eq!(hash.len(), 40);
            assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        } else {
            panic!("Hash field should be a string");
        }

        // Verify message field
        assert!(result.contains_key("message"));
        if let Some(trustfall::FieldValue::String(message)) = result.get("message") {
            assert!(!message.is_empty());
        } else {
            panic!("Message field should be a string");
        }

        // Verify author field
        assert!(result.contains_key("author"));
        if let Some(trustfall::FieldValue::String(author)) = result.get("author") {
            assert!(!author.is_empty());
        } else {
            panic!("Author field should be a string");
        }
    }
}

#[test]
fn test_query_specific_commit_content() {
    let (_temp_dir, repo) = create_test_repo_with_multiple_commits();
    let adapter = GitAdapter::new(&repo);

    let query = r#"
    {
        repository {
            commits {
                message @output
                author @output
            }
        }
    }
    "#;

    let variables: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    let results: Vec<_> =
        trustfall::execute_query(adapter.schema(), Arc::new(&adapter), query, variables)
            .unwrap()
            .collect();

    assert!(results.len() >= 2);

    // Find our specific commit messages
    let messages: Vec<String> = results
        .iter()
        .filter_map(|row| {
            if let Some(trustfall::FieldValue::String(msg)) = row.get("message") {
                Some(msg.to_string())
            } else {
                None
            }
        })
        .collect();

    assert!(messages.contains(&"Initial commit".to_string()));
    assert!(messages.contains(&"Second commit with more details".to_string()));

    // Verify author information
    let authors: Vec<String> = results
        .iter()
        .filter_map(|row| {
            if let Some(trustfall::FieldValue::String(author)) = row.get("author") {
                Some(author.to_string())
            } else {
                None
            }
        })
        .collect();

    assert!(authors.contains(&"Author User".to_string()));
}

#[test]
fn test_query_branch_commit_relationship() {
    let (_temp_dir, repo) = create_test_repo_with_multiple_commits();
    let adapter = GitAdapter::new(&repo);

    let query = r#"
    {
        repository {
            branches {
                name @output
                commit {
                    hash @output
                    message @output
                }
            }
        }
    }
    "#;

    let variables: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    let results: Vec<_> =
        trustfall::execute_query(adapter.schema(), Arc::new(&adapter), query, variables)
            .unwrap()
            .collect();

    assert!(!results.is_empty());

    for result in &results {
        // Verify branch has a name
        assert!(result.contains_key("name"));

        // Verify the commit relationship
        assert!(result.contains_key("hash"));
        assert!(result.contains_key("message"));

        if let Some(trustfall::FieldValue::String(hash)) = result.get("hash") {
            assert_eq!(hash.len(), 40);
            assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        }

        if let Some(trustfall::FieldValue::String(message)) = result.get("message") {
            // Should point to the latest commit (second commit)
            assert_eq!(message.as_ref(), "Second commit with more details");
        }
    }
}

#[test]
fn test_query_commits_with_limit() {
    let (_temp_dir, repo) = create_test_repo_with_multiple_commits();
    let adapter = GitAdapter::new(&repo);

    // Test with limit of 1
    let query = r#"
    {
        repository {
            commits(limit: 1) {
                hash @output
                message @output
            }
        }
    }
    "#;

    let variables: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    let results: Vec<_> =
        trustfall::execute_query(adapter.schema(), Arc::new(&adapter), query, variables)
            .unwrap()
            .collect();

    // Should have exactly 1 commit due to limit
    assert_eq!(results.len(), 1);

    // Verify the result has the expected fields
    let result = &results[0];
    assert!(result.contains_key("hash"));
    assert!(result.contains_key("message"));

    if let Some(trustfall::FieldValue::String(hash)) = result.get("hash") {
        assert_eq!(hash.len(), 40);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    } else {
        panic!("Hash field should be a string");
    }

    if let Some(trustfall::FieldValue::String(message)) = result.get("message") {
        // Should be the latest commit (HEAD)
        assert_eq!(message.as_ref(), "Second commit with more details");
    } else {
        panic!("Message field should be a string");
    }
}

#[test]
fn test_query_commit_date() {
    let (_temp_dir, repo) = create_test_repo_with_multiple_commits();
    let adapter = GitAdapter::new(&repo);

    let query = r#"
    {
        repository {
            commits {
                hash @output
                date @output
            }
        }
    }
    "#;

    let variables: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    let results: Vec<_> =
        trustfall::execute_query(adapter.schema(), Arc::new(&adapter), query, variables)
            .unwrap()
            .collect();

    assert!(results.len() >= 2);

    for result in &results {
        assert!(result.contains_key("date"));
        if let Some(trustfall::FieldValue::String(date)) = result.get("date") {
            // Should be a non-empty string
            assert!(!date.is_empty());
            // Should contain 'T' separator (ISO 8601 format)
            assert!(date.contains('T'));
            // Should be a reasonable length for an ISO 8601 date string
            assert!(date.len() >= 19);
        } else {
            panic!("Date field should be a string");
        }
    }
}

#[test]
fn test_query_commit_author_and_committer_fields() {
    let (_temp_dir, repo) = create_test_repo_with_multiple_commits();
    let adapter = GitAdapter::new(&repo);

    let query = r#"
    {
        repository {
            commits {
                author @output
                author_email @output
                committer @output
                committer_email @output
            }
        }
    }
    "#;

    let variables: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    let results: Vec<_> =
        trustfall::execute_query(adapter.schema(), Arc::new(&adapter), query, variables)
            .unwrap()
            .collect();

    assert!(results.len() >= 2);

    for result in &results {
        if let Some(trustfall::FieldValue::String(author)) = result.get("author") {
            assert_eq!(author.as_ref(), "Author User");
        } else {
            panic!("author field should be a string");
        }

        if let Some(trustfall::FieldValue::String(email)) = result.get("author_email") {
            assert_eq!(email.as_ref(), "author@example.com");
        } else {
            panic!("author_email field should be a string");
        }

        if let Some(trustfall::FieldValue::String(committer)) = result.get("committer") {
            assert_eq!(committer.as_ref(), "Test User");
        } else {
            panic!("committer field should be a string");
        }

        if let Some(trustfall::FieldValue::String(email)) = result.get("committer_email") {
            assert_eq!(email.as_ref(), "test@example.com");
        } else {
            panic!("committer_email field should be a string");
        }
    }
}

#[test]
fn test_query_commits_with_different_limits() {
    let (_temp_dir, repo) = create_test_repo_with_multiple_commits();
    let adapter = GitAdapter::new(&repo);

    // Test with no limit (should get all commits)
    let query_no_limit = r#"
    {
        repository {
            commits {
                hash @output
            }
        }
    }
    "#;

    let variables: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    let results_no_limit: Vec<_> = trustfall::execute_query(
        adapter.schema(),
        Arc::new(&adapter),
        query_no_limit,
        variables.clone(),
    )
    .unwrap()
    .collect();

    // Test with limit of 1
    let query_limit_1 = r#"
    {
        repository {
            commits(limit: 1) {
                hash @output
            }
        }
    }
    "#;

    let results_limit_1: Vec<_> = trustfall::execute_query(
        adapter.schema(),
        Arc::new(&adapter),
        query_limit_1,
        variables.clone(),
    )
    .unwrap()
    .collect();

    // Test with limit of 5 (should return all available commits since we only have 2)
    let query_limit_5 = r#"
    {
        repository {
            commits(limit: 5) {
                hash @output
            }
        }
    }
    "#;

    let results_limit_5: Vec<_> = trustfall::execute_query(
        adapter.schema(),
        Arc::new(&adapter),
        query_limit_5,
        variables,
    )
    .unwrap()
    .collect();

    // Assertions
    assert!(
        results_no_limit.len() >= 2,
        "Should have at least 2 commits without limit"
    );
    assert_eq!(
        results_limit_1.len(),
        1,
        "Should have exactly 1 commit with limit 1"
    );
    assert_eq!(
        results_limit_5.len(),
        results_no_limit.len(),
        "Limit 5 should return all available commits"
    );

    // Verify that limit 1 returns the same first commit as no limit
    if let (Some(first_no_limit), Some(first_limit_1)) =
        (results_no_limit.first(), results_limit_1.first())
    {
        assert_eq!(
            first_no_limit.get("hash"),
            first_limit_1.get("hash"),
            "Limited query should return the same first commit"
        );
    }
}
