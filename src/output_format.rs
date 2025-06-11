use crate::module_info::FunctionSignature;
use pyo3::prelude::*;
use std::collections::HashMap;

/// Trait for different output format visitors
pub trait OutputFormatter {
    /// Format a module tree
    fn format_tree(&self, py: Python, tree: &PyObject, module_name: &str) -> PyResult<String>;

    /// Format a function signature
    fn format_signature(&self, signature: &FunctionSignature) -> String;

    /// Format a signature not available message
    fn format_signature_not_available(&self, object_name: &str) -> String;
}

/// Pretty print formatter (current default behavior)
pub struct PrettyPrintFormatter;

impl OutputFormatter for PrettyPrintFormatter {
    fn format_tree(&self, py: Python, tree: &PyObject, module_name: &str) -> PyResult<String> {
        // Use existing tree formatter
        crate::tree_formatter::format_tree_display(py, tree, module_name)
    }

    fn format_signature(&self, signature: &FunctionSignature) -> String {
        // Use existing signature formatter
        crate::signature::format_signature_display(signature)
    }

    fn format_signature_not_available(&self, object_name: &str) -> String {
        let config = crate::config::DisplayConfig::get();
        format!(
            "{} {} (signature not available)",
            crate::config::colorize(
                &config.signature_icon,
                &config.color_scheme.signature_color,
                config
            ),
            crate::config::colorize(object_name, &config.color_scheme.signature_color, config)
        )
    }
}

/// JSON formatter for machine-readable output
pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn format_tree(&self, py: Python, tree: &PyObject, module_name: &str) -> PyResult<String> {
        // Convert PyObject tree to a serializable structure
        let mut result = HashMap::new();
        result.insert(
            "module".to_string(),
            serde_json::Value::String(module_name.to_string()),
        );

        // Convert the tree structure to JSON
        if let Ok(tree_value) = pyobject_to_json_value(py, tree) {
            result.insert("tree".to_string(), tree_value);
        }

        serde_json::to_string_pretty(&result)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn format_signature(&self, signature: &FunctionSignature) -> String {
        // Serialize signature to JSON
        serde_json::to_string_pretty(signature).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_signature_not_available(&self, object_name: &str) -> String {
        let result = serde_json::json!({
            "name": object_name,
            "available": false,
            "reason": "signature not available"
        });
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Convert PyObject to serde_json::Value
fn pyobject_to_json_value(py: Python, obj: &PyObject) -> PyResult<serde_json::Value> {
    // Try to extract as different Python types
    if let Ok(dict) = obj.extract::<HashMap<String, PyObject>>(py) {
        let mut map = serde_json::Map::new();
        for (key, value) in dict {
            if let Ok(json_value) = pyobject_to_json_value(py, &value) {
                map.insert(key, json_value);
            }
        }
        Ok(serde_json::Value::Object(map))
    } else if let Ok(list) = obj.extract::<Vec<PyObject>>(py) {
        let vec: Vec<serde_json::Value> = list
            .iter()
            .filter_map(|item| pyobject_to_json_value(py, item).ok())
            .collect();
        Ok(serde_json::Value::Array(vec))
    } else if let Ok(s) = obj.extract::<String>(py) {
        Ok(serde_json::Value::String(s))
    } else if let Ok(b) = obj.extract::<bool>(py) {
        Ok(serde_json::Value::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>(py) {
        Ok(serde_json::Value::Number(serde_json::Number::from(i)))
    } else if let Ok(f) = obj.extract::<f64>(py) {
        if let Some(num) = serde_json::Number::from_f64(f) {
            Ok(serde_json::Value::Number(num))
        } else {
            Ok(serde_json::Value::Null)
        }
    } else {
        Ok(serde_json::Value::Null)
    }
}

/// Factory function to create formatter based on format string
pub fn create_formatter(format: &str) -> Box<dyn OutputFormatter> {
    match format.to_lowercase().as_str() {
        "json" => Box::new(JsonFormatter),
        _ => Box::new(PrettyPrintFormatter),
    }
}
