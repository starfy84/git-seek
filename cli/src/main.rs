mod presets;

use clap::Parser;
use comfy_table::{Table, presets::UTF8_FULL};
use git2::Repository;
use serde_json::{Map, Value};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use trustfall_git_adapter::GitAdapter;

fn convert_trustfall_value_to_json(value: &trustfall::FieldValue) -> Value {
    match value {
        trustfall::FieldValue::Null => Value::Null,
        trustfall::FieldValue::Int64(n) => Value::Number((*n).into()),
        trustfall::FieldValue::Uint64(n) => Value::Number((*n).into()),
        trustfall::FieldValue::Float64(f) => serde_json::Number::from_f64(*f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        trustfall::FieldValue::String(s) => Value::String(s.to_string()),
        trustfall::FieldValue::Boolean(b) => Value::Bool(*b),
        trustfall::FieldValue::List(items) => {
            Value::Array(items.iter().map(convert_trustfall_value_to_json).collect())
        }
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
            format!(
                "[{}]",
                items
                    .iter()
                    .map(format_trustfall_value_for_table)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
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
    name = "git-seek",
    about = "Run Trustfall queries against a Git repository"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Inline Trustfall query
    #[arg(short, long, group = "query_source")]
    pub query: Option<String>,

    /// Path to query file
    #[arg(short, long, group = "query_source")]
    pub file: Option<PathBuf>,

    /// Optional variables: --var name=value
    #[arg(long = "var")]
    pub vars: Vec<String>,

    /// Output format
    #[arg(long, value_enum, default_value = "raw")]
    pub format: OutputFormat,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Run a preset query
    Preset {
        #[command(subcommand)]
        action: PresetAction,
    },
}

#[derive(clap::Subcommand, Debug)]
enum PresetAction {
    /// List all available presets
    List,
    /// Run a specific preset
    Run {
        /// Name of the preset to run
        name: String,

        /// Preset parameters: --param name=value
        #[arg(long = "param")]
        params: Vec<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "raw")]
        format: OutputFormat,
    },
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
enum OutputFormat {
    Table,
    Json,
    Raw,
}

use std::io::{self, IsTerminal, Read};

fn load_query(query: &Option<String>, file: &Option<PathBuf>) -> anyhow::Result<String> {
    if let Some(q) = query {
        return Ok(q.clone());
    }
    if let Some(path) = file {
        return Ok(std::fs::read_to_string(path)?);
    }
    let mut input = String::new();
    if io::stdin().is_terminal() {
        anyhow::bail!("No query provided. Use --query, --file, or pipe via stdin.");
    }
    io::stdin().read_to_string(&mut input)?;
    Ok(input)
}

/// Coerce a string CLI variable into a typed Trustfall FieldValue.
/// Tries integer, then float, then falls back to string.
fn coerce_variable(value: &str) -> trustfall::FieldValue {
    if let Ok(n) = value.parse::<i64>() {
        trustfall::FieldValue::Int64(n)
    } else if let Ok(f) = value.parse::<f64>() {
        trustfall::FieldValue::Float64(f)
    } else {
        trustfall::FieldValue::String(value.into())
    }
}

fn execute_and_output(
    adapter: &GitAdapter<'_>,
    query: &str,
    variables: BTreeMap<&str, &str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let typed_variables: BTreeMap<&str, trustfall::FieldValue> = variables
        .into_iter()
        .map(|(k, v)| (k, coerce_variable(v)))
        .collect();
    let result =
        trustfall::execute_query(adapter.schema(), Arc::new(adapter), query, typed_variables)?;

    match format {
        OutputFormat::Json => {
            let results: Vec<Value> = result.map(|row| convert_result_row_to_json(&row)).collect();
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        OutputFormat::Table => {
            let rows: Vec<_> = result.collect();
            if rows.is_empty() {
                return Ok(());
            }
            let columns: Vec<String> = rows[0].keys().map(|k| k.to_string()).collect();
            let mut table = Table::new();
            table.load_preset(UTF8_FULL).set_header(&columns);
            for row in &rows {
                let row_values = columns.iter().map(|col| match row.get(col.as_str()) {
                    Some(value) => format_trustfall_value_for_table(value),
                    None => String::new(),
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

fn run_preset(adapter: &GitAdapter<'_>, action: PresetAction) -> anyhow::Result<()> {
    match action {
        PresetAction::List => {
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_header(vec!["Name", "Description", "Parameters"]);

            for preset in presets::all_presets() {
                let params_str = if preset.params.is_empty() {
                    "(none)".to_string()
                } else {
                    preset
                        .params
                        .iter()
                        .map(|p| {
                            if let Some(default) = p.default {
                                format!("--{}: {} (default: {})", p.name, p.description, default)
                            } else {
                                format!("--{}: {} (required)", p.name, p.description)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                table.add_row(vec![preset.name, preset.description, &params_str]);
            }
            println!("{table}");
            Ok(())
        }
        PresetAction::Run {
            name,
            params,
            format,
        } => {
            let preset = presets::find_preset(&name).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown preset: '{}'. Run 'git-seek preset list' to see available presets.",
                    name
                )
            })?;

            let mut user_params = BTreeMap::new();
            for p in &params {
                match p.split_once("=") {
                    Some((k, v)) => {
                        user_params.insert(k, v);
                    }
                    None => {
                        anyhow::bail!(
                            "Invalid parameter format '{}'. Expected '--param name=value'.",
                            p
                        );
                    }
                }
            }

            let mut variables = BTreeMap::new();
            let mut inline_replacements = Vec::new();
            for param in preset.params {
                let resolved_value = if let Some(value) = user_params.get(param.name) {
                    Some(*value)
                } else if let Some(default) = param.default {
                    Some(default)
                } else if param.required {
                    anyhow::bail!(
                        "Missing required parameter '--param {}=<value>' for preset '{}'",
                        param.name,
                        preset.name
                    );
                } else {
                    None
                };

                if let Some(value) = resolved_value {
                    if param.inline {
                        if value.parse::<i64>().is_err() {
                            anyhow::bail!(
                                "Parameter '{}' must be an integer, got '{}'",
                                param.name,
                                value
                            );
                        }
                        inline_replacements.push((param.name, value));
                    } else {
                        variables.insert(param.name, value);
                    }
                }
            }

            // Inline params are substituted directly into the query string because
            // Trustfall does not support variables in edge arguments (only in @filter).
            // Safe here because preset queries are controlled constants.
            let query = if inline_replacements.is_empty() {
                preset.query.to_string()
            } else {
                let mut q = preset.query.to_string();
                for (name, value) in &inline_replacements {
                    q = q.replace(&format!("${}", name), value);
                }
                q
            };

            execute_and_output(adapter, &query, variables, &format)
        }
    }
}

fn main() -> anyhow::Result<()> {
    let repo = Repository::open_from_env()?;
    let adapter = GitAdapter::new(&repo);
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Preset { action }) => run_preset(&adapter, action),
        None => {
            let variables = cli
                .vars
                .iter()
                .filter_map(|var_entry| var_entry.split_once("="))
                .collect::<BTreeMap<_, _>>();

            let query = load_query(&cli.query, &cli.file)?;
            execute_and_output(&adapter, &query, variables, &cli.format)
        }
    }
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
        let value = trustfall::FieldValue::List(
            vec![
                trustfall::FieldValue::String("a".into()),
                trustfall::FieldValue::Int64(1),
                trustfall::FieldValue::Boolean(true),
            ]
            .into(),
        );
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
        let value = trustfall::FieldValue::List(
            vec![
                trustfall::FieldValue::String("a".into()),
                trustfall::FieldValue::Int64(1),
            ]
            .into(),
        );
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
        row.insert(
            Arc::from("name"),
            trustfall::FieldValue::String("test-repo".into()),
        );
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
    fn test_load_query_inline() {
        let query = Some("test query".to_string());
        let result = load_query(&query, &None).unwrap();
        assert_eq!(result, "test query");
    }

    #[test]
    fn test_load_query_file() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "file query content").unwrap();
        let result = load_query(&None, &Some(temp_file.path().to_path_buf())).unwrap();
        assert_eq!(result, "file query content\n");
    }

    #[test]
    fn test_load_query_file_not_found() {
        assert!(load_query(&None, &Some(PathBuf::from("/nonexistent/file.txt"))).is_err());
    }

    #[test]
    fn test_load_query_priority_inline_over_file() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "file content").unwrap();
        let query = Some("inline query".to_string());
        let result = load_query(&query, &Some(temp_file.path().to_path_buf())).unwrap();
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

    #[test]
    fn test_coerce_variable_integer() {
        assert_eq!(coerce_variable("42"), trustfall::FieldValue::Int64(42));
    }

    #[test]
    fn test_coerce_variable_negative_integer() {
        assert_eq!(coerce_variable("-7"), trustfall::FieldValue::Int64(-7));
    }

    #[test]
    fn test_coerce_variable_float() {
        assert_eq!(
            coerce_variable("3.14"),
            trustfall::FieldValue::Float64(3.14)
        );
    }

    #[test]
    fn test_coerce_variable_string() {
        assert_eq!(
            coerce_variable("hello"),
            trustfall::FieldValue::String("hello".into())
        );
    }
}
