use clap::Parser;
use git_seek::Cli;
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

/// Parse CLI args and run against the given repo path.
/// Returns Ok(()) on success, Err on failure.
fn run_cli(args: &[&str], repo_path: &std::path::Path) -> anyhow::Result<()> {
    let cli = Cli::try_parse_from(args)?;
    git_seek::run_with_repo(cli, repo_path)
}

// --- Backward compatibility ---

#[test]
fn test_query_mode_still_works() {
    let (_temp, path) = create_test_repo();
    run_cli(
        &[
            "git-seek",
            "--query",
            "{repository {name @output}}",
            "--format",
            "json",
        ],
        &path,
    )
    .unwrap();
}

// --- Preset list ---

#[test]
fn test_preset_list() {
    let (_temp, path) = create_test_repo();
    run_cli(&["git-seek", "preset", "list"], &path).unwrap();
}

// --- Preset run ---

#[test]
fn test_preset_recent_commits() {
    let (_temp, path) = create_test_repo();
    run_cli(&["git-seek", "preset", "run", "recent-commits"], &path).unwrap();
}

#[test]
fn test_preset_recent_commits_with_limit() {
    let (_temp, path) = create_test_repo();
    run_cli(
        &[
            "git-seek",
            "preset",
            "run",
            "recent-commits",
            "--param",
            "limit=1",
        ],
        &path,
    )
    .unwrap();
}

#[test]
fn test_preset_branches() {
    let (_temp, path) = create_test_repo();
    run_cli(&["git-seek", "preset", "run", "branches"], &path).unwrap();
}

#[test]
fn test_preset_tags_empty() {
    let (_temp, path) = create_test_repo();
    run_cli(&["git-seek", "preset", "run", "tags"], &path).unwrap();
}

#[test]
fn test_preset_commits_by_author() {
    let (_temp, path) = create_test_repo();
    run_cli(
        &[
            "git-seek",
            "preset",
            "run",
            "commits-by-author",
            "--param",
            "author=Test User",
        ],
        &path,
    )
    .unwrap();
}

#[test]
fn test_preset_commits_by_author_missing_param() {
    let (_temp, path) = create_test_repo();
    let result = run_cli(&["git-seek", "preset", "run", "commits-by-author"], &path);
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Missing required parameter"),
        "Expected 'Missing required parameter', got: {err}"
    );
}

#[test]
fn test_preset_search_commits() {
    let (_temp, path) = create_test_repo();
    run_cli(
        &[
            "git-seek",
            "preset",
            "run",
            "search-commits",
            "--param",
            "pattern=Initial",
        ],
        &path,
    )
    .unwrap();
}

#[test]
fn test_preset_unknown() {
    let (_temp, path) = create_test_repo();
    let result = run_cli(&["git-seek", "preset", "run", "nonexistent"], &path);
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Unknown preset"),
        "Expected 'Unknown preset', got: {err}"
    );
}

#[test]
fn test_preset_run_with_format_json() {
    let (_temp, path) = create_test_repo();
    run_cli(
        &[
            "git-seek",
            "preset",
            "run",
            "recent-commits",
            "--format",
            "json",
        ],
        &path,
    )
    .unwrap();
}
