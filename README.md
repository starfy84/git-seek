# git-seek 🔍

Query Git repositories using Trustfall's GraphQL-like syntax.

## Installation

```bash
cargo build --release
cargo install --path cli
```

## Usage

```bash
# Query repository name
git-seek --query '{repository {name @output}}'

# Query branches (JSON format)
git-seek --query '{repository {branches {name @output}}}' --format json

# Load query from file
git-seek --file query.trustfall --format table
```

## Output Formats

- `table` - Human-readable table
- `json` - JSON format
- `raw` - Raw text output

## Query Examples

**Repository info:**
```trustfall
{
  repository {
    name @output
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
      author_name @output
      message @output
    }
  }
}
```

## Schema

```graphql
type Repository {
  name: String!
  commits: [Commit!]!
  branches: [Branch!]!
}

type Commit {
  hash: String!
  author_name: String!
  author_email: String!
  message: String!
  timestamp: String!
}

type Branch {
  name: String!
  target: String!
}
```

## Development

```bash
cargo build
cargo test
```
