# git-seek CLI

A command-line tool for querying Git repositories using Trustfall's GraphQL-like syntax.

## Installation

### From crates.io

```bash
cargo install git-seek
```

### From source

```bash
git clone https://github.com/starfy84/git-seek
cd git-seek
cargo build --release -p git-seek
```

## Usage

### Basic Queries

```bash
# Query repository name
git-seek --query '{repository {name @output}}'

# Query all branches
git-seek --query '{repository {branches {name @output}}}'

# Query commits with their messages
git-seek --query '{repository {commits {hash @output message @output}}}'
```

### Input Methods

You can provide queries in three ways:

1. **Inline query** (using `--query` or `-q`):
   ```bash
   git-seek --query '{repository {name @output}}'
   ```

2. **From file** (using `--file` or `-f`):
   ```bash
   git-seek --file my-query.trustfall
   ```

3. **Via stdin** (pipe input):
   ```bash
   echo '{repository {name @output}}' | git-seek
   ```

### Variables

Use variables in your queries with `--var`:

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