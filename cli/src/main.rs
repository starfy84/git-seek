use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use git2::Repository;
use trustfall_git_adapter::GitAdapter;
use clap::{ArgGroup, Parser};
use serde_json::{Map, Value};
use comfy_table::{Table, presets::UTF8_FULL};

fn convert_trustfall_value_to_json(value: &trustfall::FieldValue) -> Value {
    match value {
        trustfall::FieldValue::Null => Value::Null,
        trustfall::FieldValue::Int64(n) => Value::Number((*n).into()),
        trustfall::FieldValue::Uint64(n) => Value::Number((*n).into()),
        trustfall::FieldValue::Float64(f) => {
            serde_json::Number::from_f64(*f)
                .map(Value::Number)
                .unwrap_or(Value::Null)
        },
        trustfall::FieldValue::String(s) => Value::String(s.to_string()),
        trustfall::FieldValue::Boolean(b) => Value::Bool(*b),
        trustfall::FieldValue::List(items) => {
            Value::Array(items.iter().map(convert_trustfall_value_to_json).collect())
        },
        _ => Value::String(format!("{:?}", value)), // fallback for other types
    }
}

fn format_trustfall_value_for_table(value: &trustfall::FieldValue) -> String {
    match value {
        trustfall::FieldValue::Null => "null".to_string(),
        trustfall::FieldValue::Int64(n) => n.to_string(),
        trustfall::FieldValue::Uint64(n) => n.to_string(),
        trustfall::FieldValue::Float64(f) => f.to_string(),
        trustfall::FieldValue::String(s) => s.to_string(),
        trustfall::FieldValue::Boolean(b) => b.to_string(),
        trustfall::FieldValue::List(items) => {
            format!("[{}]", items.iter()
                .map(format_trustfall_value_for_table)
                .collect::<Vec<_>>()
                .join(", "))
        },
        _ => format!("{:?}", value), // fallback for other types
    }
}

fn convert_result_row_to_json(row: &BTreeMap<std::sync::Arc<str>, trustfall::FieldValue>) -> Value {
    let mut map = Map::new();
    for (key, value) in row {
        map.insert(key.to_string(), convert_trustfall_value_to_json(value));
    }
    Value::Object(map)
}

#[derive(Parser, Debug)]
#[command(
    name = "gitql",
    about = "Run Trustfall queries against a Git repository",
    group(
        ArgGroup::new("query_source")
            .args(["query", "file"])
            .required(false) // because we want to allow stdin fallback
    )
)]
struct Args {
    /// Inline Trustfall query
    #[arg(short, long)]
    pub query: Option<String>,

    /// Path to query file
    #[arg(short, long)]
    pub file: Option<PathBuf>,

    /// Optional variables: --var name=value
    #[arg(long = "var")]
    pub vars: Vec<String>,

    /// Output format
    #[arg(long, value_enum, default_value = "raw")]
    pub format: OutputFormat,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
enum OutputFormat {
    Table,
    Json,
    Raw,
}

use std::io::{self, Read, IsTerminal};

impl Args {
    pub fn load_query(&self) -> anyhow::Result<String> {
        if let Some(q) = &self.query {
            return Ok(q.clone());
        }

        if let Some(path) = &self.file {
            return Ok(std::fs::read_to_string(path)?);
        }

        // Fallback to stdin
        let mut input = String::new();
        if io::stdin().is_terminal() {
            anyhow::bail!("No query provided. Use --query, --file, or pipe via stdin.");
        }

        io::stdin().read_to_string(&mut input)?;
        Ok(input)
    }
}

fn main() -> anyhow::Result<()> {
    let repo = Repository::open_from_env()?;
    let adapter = GitAdapter::new(&repo);

    let args = Args::parse();

    let variables = args.vars.iter().filter_map(|var_entry| {
        var_entry.split_once("=")
    }).collect::<BTreeMap<_, _>>();

    let query = args.load_query()?;

    let result = trustfall::execute_query(adapter.schema(), Arc::new(&adapter), query.as_str(), variables)?;

    match args.format {
        OutputFormat::Json => {
            let results: Vec<Value> = result
                .map(|row| convert_result_row_to_json(&row))
                .collect();
            println!("{}", serde_json::to_string_pretty(&results)?);
        },
        OutputFormat::Table => {
            let rows: Vec<_> = result.collect();
            if rows.is_empty() {
                return Ok(());
            }
            
            let columns: Vec<String> = rows[0].keys().map(|k| k.to_string()).collect();
            
            let mut table = Table::new();
            table.load_preset(UTF8_FULL)
                 .set_header(&columns);
            
            for row in &rows {
                let row_values = columns.iter().map(|col| {
                    match row.get(col.as_str()) {
                        Some(value) => format_trustfall_value_for_table(value),
                        None => String::new(),
                    }
                });
                table.add_row(row_values);
            }
            
            println!("{table}");
        }
        OutputFormat::Raw => {
            for row in result {
                println!("{:?}", row);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn test_convert_trustfall_value_to_json_string() {
        let value = trustfall::FieldValue::String("test".into());
        let result = convert_trustfall_value_to_json(&value);
        assert_eq!(result, json!("test"));
    }

    #[test]
    fn test_convert_trustfall_value_to_json_int64() {
        let value = trustfall::FieldValue::Int64(42);
        let result = convert_trustfall_value_to_json(&value);
        assert_eq!(result, json!(42));
    }

    #[test]
    fn test_convert_trustfall_value_to_json_uint64() {
        let value = trustfall::FieldValue::Uint64(42);
        let result = convert_trustfall_value_to_json(&value);
        assert_eq!(result, json!(42));
    }

    #[test]
    fn test_convert_trustfall_value_to_json_float64() {
        let value = trustfall::FieldValue::Float64(3.14);
        let result = convert_trustfall_value_to_json(&value);
        assert_eq!(result, json!(3.14));
    }

    #[test]
    fn test_convert_trustfall_value_to_json_boolean() {
        let value = trustfall::FieldValue::Boolean(true);
        let result = convert_trustfall_value_to_json(&value);
        assert_eq!(result, json!(true));

        let value = trustfall::FieldValue::Boolean(false);
        let result = convert_trustfall_value_to_json(&value);
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_convert_trustfall_value_to_json_null() {
        let value = trustfall::FieldValue::Null;
        let result = convert_trustfall_value_to_json(&value);
        assert_eq!(result, json!(null));
    }

    #[test]
    fn test_convert_trustfall_value_to_json_list() {
        let value = trustfall::FieldValue::List(vec![
            trustfall::FieldValue::String("a".into()),
            trustfall::FieldValue::Int64(1),
            trustfall::FieldValue::Boolean(true),
        ].into());
        let result = convert_trustfall_value_to_json(&value);
        assert_eq!(result, json!(["a", 1, true]));
    }

    #[test]
    fn test_format_trustfall_value_for_table_string() {
        let value = trustfall::FieldValue::String("test".into());
        let result = format_trustfall_value_for_table(&value);
        assert_eq!(result, "test");
    }

    #[test]
    fn test_format_trustfall_value_for_table_numbers() {
        let value = trustfall::FieldValue::Int64(-42);
        assert_eq!(format_trustfall_value_for_table(&value), "-42");

        let value = trustfall::FieldValue::Uint64(42);
        assert_eq!(format_trustfall_value_for_table(&value), "42");

        let value = trustfall::FieldValue::Float64(3.14);
        assert_eq!(format_trustfall_value_for_table(&value), "3.14");
    }

    #[test]
    fn test_format_trustfall_value_for_table_boolean() {
        let value = trustfall::FieldValue::Boolean(true);
        assert_eq!(format_trustfall_value_for_table(&value), "true");

        let value = trustfall::FieldValue::Boolean(false);
        assert_eq!(format_trustfall_value_for_table(&value), "false");
    }

    #[test]
    fn test_format_trustfall_value_for_table_null() {
        let value = trustfall::FieldValue::Null;
        assert_eq!(format_trustfall_value_for_table(&value), "null");
    }

    #[test]
    fn test_format_trustfall_value_for_table_list() {
        let value = trustfall::FieldValue::List(vec![
            trustfall::FieldValue::String("a".into()),
            trustfall::FieldValue::Int64(1),
        ].into());
        let result = format_trustfall_value_for_table(&value);
        assert_eq!(result, "[a, 1]");
    }

    #[test]
    fn test_format_trustfall_value_for_table_empty_list() {
        let value = trustfall::FieldValue::List(vec![].into());
        let result = format_trustfall_value_for_table(&value);
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_convert_result_row_to_json() {
        let mut row = BTreeMap::new();
        row.insert(Arc::from("name"), trustfall::FieldValue::String("test-repo".into()));
        row.insert(Arc::from("count"), trustfall::FieldValue::Int64(42));
        row.insert(Arc::from("active"), trustfall::FieldValue::Boolean(true));

        let result = convert_result_row_to_json(&row);
        let expected = json!({
            "name": "test-repo",
            "count": 42,
            "active": true
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_convert_result_row_to_json_empty() {
        let row = BTreeMap::new();
        let result = convert_result_row_to_json(&row);
        assert_eq!(result, json!({}));
    }

    #[test]
    fn test_args_load_query_inline() {
        let args = Args {
            query: Some("test query".to_string()),
            file: None,
            vars: vec![],
            format: OutputFormat::Raw,
        };
        let result = args.load_query().unwrap();
        assert_eq!(result, "test query");
    }

    #[test]
    fn test_args_load_query_file() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "file query content").unwrap();

        let args = Args {
            query: None,
            file: Some(temp_file.path().to_path_buf()),
            vars: vec![],
            format: OutputFormat::Raw,
        };
        let result = args.load_query().unwrap();
        assert_eq!(result, "file query content\n");
    }

    #[test]
    fn test_args_load_query_file_not_found() {
        let args = Args {
            query: None,
            file: Some(PathBuf::from("/nonexistent/file.txt")),
            vars: vec![],
            format: OutputFormat::Raw,
        };
        assert!(args.load_query().is_err());
    }

    #[test]
    fn test_args_load_query_priority_inline_over_file() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "file content").unwrap();

        let args = Args {
            query: Some("inline query".to_string()),
            file: Some(temp_file.path().to_path_buf()),
            vars: vec![],
            format: OutputFormat::Raw,
        };
        let result = args.load_query().unwrap();
        assert_eq!(result, "inline query");
    }

    #[test]
    fn test_output_format_values() {
        use clap::ValueEnum;
        let formats = OutputFormat::value_variants();
        assert_eq!(formats.len(), 3);
        assert!(formats.contains(&OutputFormat::Table));
        assert!(formats.contains(&OutputFormat::Json));
        assert!(formats.contains(&OutputFormat::Raw));
    }
}
