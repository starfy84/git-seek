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

```bash
# Query repository name
git-seek --query '{repository {name @output}}'

# Query branches with JSON output
git-seek --query '{repository {branches {name @output}}}' --format json

# Load query from file
git-seek --file query.trustfall --format table

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

## Schema

The Trustfall schema defines the structure for querying Git repositories:

```graphql
type Repository {
    name: String!
    commits: [Commit!]!
    branches: [Branch!]!
}

type Commit {
    hash: String!
    message: String!
}

type Branch {
    name: String!
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
