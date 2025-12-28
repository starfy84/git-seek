# trustfall_git_adapter

A Trustfall adapter for querying Git repositories using GraphQL-like syntax.

## Overview

This crate provides a Trustfall adapter that allows you to query Git repositories using Trustfall's GraphQL-like query language. It acts as a bridge between Trustfall queries and Git repository data via the `git2` crate.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
trustfall_git_adapter = "0.1.0"
git2 = "0.20"
trustfall = "0.8"
```

## Usage

### Basic Setup

```rust
use git2::Repository;
use trustfall_git_adapter::GitAdapter;
use std::sync::Arc;
use std::collections::BTreeMap;

// Open a Git repository
let repo = Repository::open(".")?;

// Create the adapter
let adapter = GitAdapter::new(&repo);

// Execute a query
let query = r#"{
  repository {
    name @output
    branches {
      name @output
    }
  }
}"#;

let variables = BTreeMap::new();
let results = trustfall::execute_query(
    adapter.schema(),
    Arc::new(&adapter),
    query,
    variables
)?;

// Process results
for result in results {
    println!("{:?}", result);
}
```

### Query Examples

**Repository name:**
```rust
let query = r#"{
  repository {
    name @output
  }
}"#;
```

**All branches:**
```rust
let query = r#"{
  repository {
    branches {
      name @output
    }
  }
}"#;
```

**Commits with messages:**
```rust
let query = r#"{
  repository {
    commits {
      hash @output
      message @output
    }
  }
}"#;
```

**Branches with their latest commits:**
```rust
let query = r#"{
  repository {
    branches {
      name @output
      commit {
        hash @output
        message @output
      }
    }
  }
}"#;
```

## Schema

The adapter implements the following GraphQL schema:

```graphql
schema {
  query: RootSchemaQuery
}

type RootSchemaQuery {
    repository: Repository
}

type Repository {
    name: String!
    commits: [Commit!]!
    branches: [Branch!]!
}

type Commit {
    hash: String!
    message: String
    author: String
}

type Branch {
    name: String!
    commit: Commit!
}
```

### Supported Operations

- **Repository queries**: Access repository name and metadata
- **Branch enumeration**: List all branches in the repository
- **Commit traversal**: Iterate through commit history
- **Branch-to-commit relationships**: Access the latest commit for each branch

## Architecture

The adapter is built using Trustfall's derive macros and implements:

- **Vertices**: `Repository`, `Commit`, `Branch`
- **Edges**: Navigation between related Git objects
- **Properties**: Data extraction from Git objects

### Key Components

- `GitAdapter` - Main adapter struct implementing the Trustfall adapter trait
- `Vertex` - Enum representing different Git objects (Repository, Commit, Branch)
- Edge resolution for navigating between Git objects
- Property resolution for extracting data from Git objects

## Performance Considerations

- The adapter loads Git data on-demand during query execution
- Large repositories with many commits/branches may take time to process
- Consider using filters and limits in your Trustfall queries for better performance

## Error Handling

The adapter handles common Git errors:
- Repository not found or inaccessible
- Invalid Git objects or references
- I/O errors when reading Git data

Errors are propagated through Trustfall's error handling mechanism.

## Development

### Running Tests

```bash
cargo test -p trustfall_git_adapter
```

### Testing with Different Repositories

```bash
# Test in a specific repository
cd /path/to/your/git/repo
cargo test -p trustfall_git_adapter
```

### Adding New Query Capabilities

To extend the schema:

1. Update `schema.graphql`
2. Add new vertex types to `vertex.rs`
3. Implement edge resolution in `edges.rs`
4. Add property resolution in `properties.rs`
5. Update tests in `tests/`

## Dependencies

- `git2` - Git repository access
- `trustfall` - Query execution engine
- `trustfall_core` - Core Trustfall functionality
- `trustfall_derive` - Derive macros for adapter implementation

## Examples

See the `git-seek` CLI tool for a complete example of how to use this adapter in a real application.

## License

BSD-3-Clause