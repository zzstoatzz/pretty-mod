use pyo3::prelude::*;

/// Display a function signature
pub fn display_signature(py: Python, import_path: &str, quiet: bool) -> PyResult<String> {
    // Try to import the object with auto-download support
    let func = match crate::utils::import_object_with_download(py, import_path, quiet) {
        Ok(obj) => obj,
        Err(e) => {
            return Ok(format!("Error: Could not import {}: {}", import_path, e));
        }
    };

    // Check if callable
    let builtins = py.import("builtins")?;
    let is_callable = builtins
        .getattr("callable")?
        .call1((&func,))?
        .extract::<bool>()?;
    if !is_callable {
        return Ok(format!(
            "Error: Imported object {} is not callable",
            import_path
        ));
    }

    // Get the function name
    let func_name = if let Ok(name) = func.bind(py).getattr("__name__") {
        name.extract::<String>()
            .unwrap_or_else(|_| "unknown".to_string())
    } else {
        // Extract name from import path
        import_path
            .split(&[':', '.'][..])
            .last()
            .unwrap_or("unknown")
            .to_string()
    };

    let inspect = py.import("inspect")?;
    match inspect.getattr("signature")?.call1((&func,)) {
        Ok(sig) => {
            // Build the formatted output
            let mut result = format!("📎 {}\n", func_name);
            result.push_str("├── Parameters:\n");

            // Get parameters from signature
            let params_obj = sig.getattr("parameters")?;
            let params_values = params_obj.call_method0("values")?;
            let builtins = py.import("builtins")?;
            let params_list: Vec<PyObject> = builtins
                .getattr("list")?
                .call1((params_values,))?
                .extract()?;

            if params_list.is_empty() {
                result.push_str("└── (no parameters)");
            } else {
                let mut has_seen_keyword_only_separator = false;

                for (i, param) in params_list.iter().enumerate() {
                    let is_last = i == params_list.len() - 1;
                    let param_bound = param.bind(py);

                    // Get parameter properties
                    let name: String = param_bound.getattr("name")?.extract()?;
                    let kind = param_bound.getattr("kind")?;
                    let default = param_bound.getattr("default")?;
                    let annotation = param_bound.getattr("annotation")?;

                    // Get kind name
                    let kind_name: String = kind.getattr("name")?.extract()?;

                    // Handle positional-only separator
                    if kind_name == "POSITIONAL_ONLY" && i < params_list.len() - 1 {
                        // Check if next param is not POSITIONAL_ONLY
                        let next_param = params_list[i + 1].bind(py);
                        let next_kind = next_param.getattr("kind")?;
                        let next_kind_name: String = next_kind.getattr("name")?.extract()?;
                        if next_kind_name != "POSITIONAL_ONLY" {
                            result.push_str("├── /\n");
                        }
                    }

                    // Handle keyword-only separator
                    if !has_seen_keyword_only_separator && kind_name == "KEYWORD_ONLY" {
                        result.push_str("├── *\n");
                        has_seen_keyword_only_separator = true;
                    }

                    // Format the parameter
                    let mut param_str = String::new();

                    // Handle special parameters
                    if kind_name == "VAR_POSITIONAL" {
                        param_str.push('*');
                    } else if kind_name == "VAR_KEYWORD" {
                        param_str.push_str("**");
                    }

                    param_str.push_str(&name);

                    // Add type annotation if present
                    let empty = inspect.getattr("_empty")?;
                    if !annotation.is(&empty) {
                        let annotation_str = annotation.to_string();
                        // Only filter out verbose class representations
                        if !annotation_str.starts_with("<class '") {
                            param_str.push_str(&format!(": {}", annotation_str));
                        }
                    }

                    // Add default value if present
                    if !default.is(&empty) {
                        param_str.push('=');
                        let default_str = default.to_string();
                        if default_str.len() > 20 {
                            param_str.push_str("...");
                        } else {
                            param_str.push_str(&default_str);
                        }
                    }

                    let prefix = if is_last
                        && !sig
                            .getattr("return_annotation")
                            .map(|r| !r.is(&empty))
                            .unwrap_or(false)
                    {
                        "└── "
                    } else {
                        "├── "
                    };
                    result.push_str(&format!("{}{}\n", prefix, param_str));
                }
            }

            // Check for return annotation
            if let Ok(return_annotation) = sig.getattr("return_annotation") {
                let empty = inspect.getattr("_empty")?;
                if !return_annotation.is(&empty) {
                    result.push_str("└── Returns:\n");
                    result.push_str(&format!("    └── {}", return_annotation));
                }
            }

            Ok(result)
        }
        Err(_) => Ok(format!("📎 {} (signature unavailable)", func_name)),
    }
}