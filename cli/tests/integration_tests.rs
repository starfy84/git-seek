use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_repo() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    let repo = git2::Repository::init(&repo_path).unwrap();

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
    )
    .unwrap();

    (temp_dir, repo_path)
}

fn git_seek() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("git-seek"))
}

// --- Backward compatibility ---

#[test]
fn test_query_mode_still_works() {
    let (_temp, path) = create_test_repo();
    git_seek()
        .current_dir(&path)
        .args(["--query", "{repository {name @output}}", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("name"));
}

// --- Preset list ---

#[test]
fn test_preset_list() {
    let (_temp, path) = create_test_repo();
    git_seek()
        .current_dir(&path)
        .args(["preset", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("recent-commits"))
        .stdout(predicate::str::contains("branches"))
        .stdout(predicate::str::contains("tags"))
        .stdout(predicate::str::contains("commits-by-author"))
        .stdout(predicate::str::contains("search-commits"));
}

// --- Preset run ---

#[test]
fn test_preset_recent_commits() {
    let (_temp, path) = create_test_repo();
    git_seek()
        .current_dir(&path)
        .args(["preset", "run", "recent-commits"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initial commit"));
}

#[test]
fn test_preset_recent_commits_with_limit() {
    let (_temp, path) = create_test_repo();
    git_seek()
        .current_dir(&path)
        .args(["preset", "run", "recent-commits", "--param", "limit=1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initial commit"));
}

#[test]
fn test_preset_branches() {
    let (_temp, path) = create_test_repo();
    git_seek()
        .current_dir(&path)
        .args(["preset", "run", "branches"])
        .assert()
        .success();
}

#[test]
fn test_preset_tags_empty() {
    let (_temp, path) = create_test_repo();
    git_seek()
        .current_dir(&path)
        .args(["preset", "run", "tags"])
        .assert()
        .success();
}

#[test]
fn test_preset_commits_by_author() {
    let (_temp, path) = create_test_repo();
    git_seek()
        .current_dir(&path)
        .args([
            "preset",
            "run",
            "commits-by-author",
            "--param",
            "author=Test User",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initial commit"));
}

#[test]
fn test_preset_commits_by_author_missing_param() {
    let (_temp, path) = create_test_repo();
    git_seek()
        .current_dir(&path)
        .args(["preset", "run", "commits-by-author"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Missing required parameter"));
}

#[test]
fn test_preset_search_commits() {
    let (_temp, path) = create_test_repo();
    git_seek()
        .current_dir(&path)
        .args([
            "preset",
            "run",
            "search-commits",
            "--param",
            "pattern=Initial",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initial commit"));
}

#[test]
fn test_preset_unknown() {
    let (_temp, path) = create_test_repo();
    git_seek()
        .current_dir(&path)
        .args(["preset", "run", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown preset"));
}

#[test]
fn test_preset_run_with_format_json() {
    let (_temp, path) = create_test_repo();
    git_seek()
        .current_dir(&path)
        .args(["preset", "run", "recent-commits", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("["));
}
