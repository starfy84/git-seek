# git-seek 🔍

A Rust workspace for querying Git repositories using Trustfall's GraphQL-like syntax.

## Project Structure

This workspace contains two crates:

- **`git-seek`** (`cli/`) - Command-line interface for querying Git repositories
- **`trustfall_git_adapter`** (`trustfall_git_adapter/`) - Trustfall adapter library for Git repositories

## Quick Start

### Install from crates.io

```bash
cargo install git-seek
```

### Build from source

```bash
git clone git@github.com:starfy84/git-seek.git
cd git-seek
cargo build --release
```

## Basic Usage

### Preset Queries

Run common queries without writing Trustfall:

```bash
# List available presets
git-seek preset list

# Show recent commits (default: last 10)
git-seek preset run recent-commits

# Show last 5 commits in table format
git-seek preset run recent-commits --param limit=5 --format table

# List all branches
git-seek preset run branches

# List all tags
git-seek preset run tags

# Find commits by a specific author
git-seek preset run commits-by-author --param author="Alice"

# Search commit messages with regex
git-seek preset run search-commits --param pattern="fix.*bug"
```

### Custom Queries

Write your own Trustfall queries for full control:

```bash
# Query repository name
git-seek --query '{repository {name @output}}'

# Query branches with JSON output
git-seek --query '{repository {branches {name @output}}}' --format json

# Load query from file
git-seek --file query.graphql --format table

# Use variables in queries
git-seek --query '{repository {name @output}}' --var repo_name=my-repo
```

## Output Formats

- `table` - Human-readable table (great for terminal use)
- `json` - JSON format (perfect for scripting)
- `raw` - Raw debug output

## Example Queries

**Repository information:**
```trustfall
{
  repository {
    name @output
  }
}
```

**Branch listing:**
```trustfall
{
  repository {
    branches {
      name @output
    }
  }
}
```

**Commit history:**
```trustfall
{
  repository {
    commits {
      hash @output
      message @output
    }
  }
}
```

**Recent commits (limited):**
```trustfall
{
  repository {
    commits(limit: 10) {
      hash @output
      message @output
      author @output
      date @output
    }
  }
}
```

**Commit author and committer details:**
```trustfall
{
  repository {
    commits(limit: 5) {
      hash @output
      author @output
      author_email @output
      committer @output
      committer_email @output
    }
  }
}
```

**Branch with latest commit:**
```trustfall
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
```

**Tag listing:**
```trustfall
{
  repository {
    tags {
      name @output
      message @output
      tagger_name @output
    }
  }
}
```

**Tags with their commits:**
```trustfall
{
  repository {
    tags {
      name @output
      commit {
        hash @output
        message @output
      }
    }
  }
}
```

## Schema

The Trustfall schema defines the structure for querying Git repositories:

```graphql
type Repository {
    name: String!
    commits(limit: Int): [Commit!]!
    branches: [Branch!]!
    tags: [Tag!]!
}

type Commit {
    hash: String!
    message: String
    author: String
    author_email: String
    committer: String
    committer_email: String
    date: String
}

type Branch {
    name: String!
    commit: Commit!
}

type Tag {
    name: String!
    message: String
    tagger_name: String
    tagger_email: String
    commit: Commit!
}
```

## Library Usage

The `trustfall_git_adapter` crate can be used as a library in your own Rust projects:

```rust
use git2::Repository;
use trustfall_git_adapter::GitAdapter;
use std::sync::Arc;

let repo = Repository::open(".")?;
let adapter = GitAdapter::new(&repo);

let query = r#"{
  repository {
    name @output
  }
}"#;

let results = trustfall::execute_query(
    adapter.schema(),
    Arc::new(&adapter),
    query,
    std::collections::BTreeMap::new()
)?;
```

## Development

```bash
# Build the entire workspace
cargo build

# Run all tests
cargo test

# Build individual crates
cargo build -p git-seek
cargo build -p trustfall_git_adapter

# Run CLI tests
cargo test -p git-seek

# Run adapter tests
cargo test -p trustfall_git_adapter
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

BSD-3-Clause
