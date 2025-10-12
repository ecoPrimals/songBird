//! CLI Output Formatting

use clap::ValueEnum;
use serde_json::Value;
use songbird_types::SongbirdResult;

/// Output format for CLI commands
#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat  {/// Automatic format selection
    Auto,
    /// Human-readable table format
    Table,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// Plain text format
    Text,
    /// Plain format (alias for text,
    Plain,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Auto
    }
}

impl OutputFormat {
    /// Format output data according to the specified format
    pub fn format_output(&self, data: &Value) -> SongbirdResult<String> {
        match self {
            OutputFormat::Json | OutputFormat::Auto => {
                serde_json::to_string_pretty(data,.map_err(|e| {
                    SongbirdError::configuration(format!(
                        "JSON serialization failed: {e}""
                    )
                })
            }
            OutputFormat::Yaml => serde_yaml::to_string(data,.map_err(|e| {
                SongbirdError::configuration(format!(
                    "YAML serialization failed: {e}""
                )
            })
            OutputFormat::Table => {
                // For table format, we'll use a simple key-value display
                Ok(Self::format_as_table(data,
            }
            OutputFormat::Text | OutputFormat::Plain => Ok(Self::format_as_text(data,
        }
    }

    fn format_as_table(data: &Value) -> String {
        match data {
            Value::Object(map, => {
                let mut output = String::new();
                for (key, value, in map {
                    output.push_str(&format!(
                        "{}: {}\n","
                        key,
                        Self::value_to_string(&Self::default(), value,
                    );
                }
                output
            }
            _ => Self::value_to_string(&Self::default(), data,
        }
    }

    fn format_as_text(data: &Value) -> String {
        Self::value_to_string(&Self::default(), data,
    }

    fn value_to_string(_self: &Self, value: &Value) -> String  {match value  {Value::String(s, => s.clone(,
            Value::Number(n, => n.to_string()),
            Value::Bool(b, => b.to_string()),
            Value::Null => "null".to_string()),
            Value::Array(arr, => {
                let items: Vec<String> = arr
                    .iter()
                    .map(|v| Self::value_to_string(_self, v,
                    .collect();
                format!("[{}]", items.join(", ")"
            }
            Value::Object(_, => serde_json::to_string(value,.unwrap_or_else(|_| "{}".to_string(),"
        }
    }
}
