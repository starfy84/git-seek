use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;
use git2::Repository;

/// Helper to create a test git repository
fn create_test_repo() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();
    
    let repo = Repository::init(&repo_path).unwrap();
    
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Test User").unwrap();
    config.set_str("user.email", "test@example.com").unwrap();
    
    let signature = git2::Signature::now("Test User", "test@example.com").unwrap();
    let tree_id = {
        let mut index = repo.index().unwrap();
        index.write_tree().unwrap()
    };
    let tree = repo.find_tree(tree_id).unwrap();
    
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],
    ).unwrap();
    
    (temp_dir, repo_path)
}

#[test]
fn test_cli_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("git-seek"));
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Run Trustfall queries against a Git repository"))
        .stdout(predicate::str::contains("--query"))
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn test_query_repository_name_json() {
    let (_temp_dir, repo_path) = create_test_repo();
    
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("git-seek"));
    cmd.current_dir(&repo_path)
        .arg("--query")
        .arg("{repository {name @output}}")
        .arg("--format")
        .arg("json");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("name"));
}

#[test]
fn test_query_repository_name_table() {
    let (_temp_dir, repo_path) = create_test_repo();
    
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("git-seek"));
    cmd.current_dir(&repo_path)
        .arg("--query")
        .arg("{repository {name @output}}")
        .arg("--format")
        .arg("table");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("│"));
}

#[test]
fn test_query_repository_name_raw() {
    let (_temp_dir, repo_path) = create_test_repo();
    
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("git-seek"));
    cmd.current_dir(&repo_path)
        .arg("--query")
        .arg("{repository {name @output}}")
        .arg("--format")
        .arg("raw");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("name"));
}

#[test]
fn test_invalid_query_syntax() {
    let (_temp_dir, repo_path) = create_test_repo();
    
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("git-seek"));
    cmd.current_dir(&repo_path)
        .arg("--query")
        .arg("invalid syntax {{{")
        .arg("--format")
        .arg("json");
    
    cmd.assert()
        .failure();
}

#[test]
fn test_not_in_git_repository() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("git-seek"));
    cmd.current_dir(&temp_dir.path())
        .arg("--query")
        .arg("{repository {name @output}}")
        .arg("--format")
        .arg("json");
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("could not find repository"));
}