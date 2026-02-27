# git-seek CLI

A command-line tool for querying Git repositories using Trustfall's GraphQL-like syntax.

## Installation

### From crates.io

```bash
cargo install git-seek
```

### From source

```bash
git clone git@github.com:starfy84/git-seek.git
cd git-seek
cargo build --release -p git-seek
```

## Usage

### Preset Queries

Run common queries without writing Trustfall:

```bash
# List all available presets
git-seek preset list

# Show recent commits (default: last 10)
git-seek preset run recent-commits

# Show last 5 commits in table format
git-seek preset run recent-commits --param limit=5 --format table

# List all branches with their latest commit
git-seek preset run branches

# List all tags
git-seek preset run tags

# Find commits by a specific author
git-seek preset run commits-by-author --param author="Alice"

# Search commit messages with regex
git-seek preset run search-commits --param pattern="fix.*bug"
```

#### Available Presets

| Preset | Description | Parameters |
|--------|-------------|------------|
| `recent-commits` | Show recent commits | `--param limit=N` (default: 10) |
| `branches` | List all branches with latest commit | (none) |
| `tags` | List all tags with their commit | (none) |
| `commits-by-author` | Commits by a specific author | `--param author=NAME` (required) |
| `search-commits` | Search commit messages by regex | `--param pattern=REGEX` (required) |

### Custom Queries

Write your own Trustfall queries for full control:

```bash
# Query repository name
git-seek --query '{repository {name @output}}'

# Query all branches
git-seek --query '{repository {branches {name @output}}}'

# Query commits with their messages
git-seek --query '{repository {commits {hash @output message @output}}}'
```

### Input Methods

You can provide custom queries in three ways:

1. **Inline query** (using `--query` or `-q`):
   ```bash
   git-seek --query '{repository {name @output}}'
   ```

2. **From file** (using `--file` or `-f`):
   ```bash
   git-seek --file my-query.graphql
   ```

3. **Via stdin** (pipe input):
   ```bash
   echo '{repository {name @output}}' | git-seek
   ```

### Variables

Use variables in your custom queries with `--var`:

```bash
git-seek --query '{repository {name @output}}' --var repo_name=my-repo
```

### Output Formats

Control the output format with `--format`:

- `raw` (default) - Raw debug output
- `json` - Pretty-printed JSON
- `table` - Human-readable table

```bash
# JSON output
git-seek --query '{repository {branches {name @output}}}' --format json

# Table output
git-seek --query '{repository {commits {hash @output message @output}}}' --format table
```

## Examples

### Repository Information

```bash
git-seek --query '{
  repository {
    name @output
  }
}' --format json
```

### Branch Listing with Latest Commits

```bash
git-seek --query '{
  repository {
    branches {
      name @output
      commit {
        hash @output
        message @output
      }
    }
  }
}' --format table
```

### Recent Commits

```bash
git-seek --query '{
  repository {
    commits {
      hash @output
      message @output
    }
  }
}' --format table
```

### Limited Commit History

```bash
# Get only the last 5 commits
git-seek --query '{
  repository {
    commits(limit: 5) {
      hash @output
      message @output
      author @output
      date @output
    }
  }
}' --format table
```

### Tag Listing

```bash
git-seek --query '{
  repository {
    tags {
      name @output
      message @output
      tagger_name @output
    }
  }
}' --format table
```

### Tags with Commits

```bash
git-seek --query '{
  repository {
    tags {
      name @output
      commit {
        hash @output
        message @output
      }
    }
  }
}' --format table
```

### Author and Committer Details

```bash
# Show author and committer info for recent commits
git-seek --query '{
  repository {
    commits(limit: 5) {
      hash @output
      author @output
      author_email @output
      committer @output
      committer_email @output
    }
  }
}' --format table
```

## Error Handling

The tool provides helpful error messages:

- If no query is provided and stdin is a terminal, it will prompt for input method
- Invalid Trustfall syntax will show parsing errors
- Git repository errors (e.g., not in a Git repo) are clearly reported

## Development

```bash
# Run tests
cargo test -p git-seek

# Build in debug mode
cargo build -p git-seek

# Run with debug output
RUST_LOG=debug git-seek --query '{repository {name @output}}'
```

## Dependencies

- `trustfall_git_adapter` - The Git adapter library
- `clap` - Command-line argument parsing
- `git2` - Git repository access
- `serde_json` - JSON serialization
- `comfy-table` - Table formatting
- `anyhow` - Error handling

## License

BSD-3-Clause