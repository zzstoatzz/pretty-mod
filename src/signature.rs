use crate::config::{colorize, DisplayConfig};
use crate::module_info::{FunctionSignature, ModuleInfo};
use crate::import_resolver::ImportChainResolver;
use pyo3::prelude::*;
use ruff_python_ast::{Expr, ParameterWithDefault, Parameters};

// ===== AST Parameter Parsing =====

/// Extract signature information from AST parameters
pub fn format_parameters(params: &Parameters) -> String {
    let mut parts = Vec::new();

    // Handle positional-only parameters
    if !params.posonlyargs.is_empty() {
        for param in &params.posonlyargs {
            parts.push(format_parameter(param));
        }
        parts.push("/".to_string());
    }

    // Handle regular positional parameters
    for param in &params.args {
        parts.push(format_parameter(param));
    }

    // Handle *args
    if let Some(vararg) = &params.vararg {
        parts.push(format!("*{}", vararg.name.as_str()));
    } else if !params.kwonlyargs.is_empty() {
        // If we have keyword-only args but no *args, add a bare *
        parts.push("*".to_string());
    }

    // Handle keyword-only parameters
    for param in &params.kwonlyargs {
        parts.push(format_parameter(param));
    }

    // Handle **kwargs
    if let Some(kwarg) = &params.kwarg {
        parts.push(format!("**{}", kwarg.name.as_str()));
    }

    parts.join(", ")
}

fn format_parameter(param: &ParameterWithDefault) -> String {
    let mut result = param.parameter.name.as_str().to_string();

    // Add type annotation if present
    if let Some(annotation) = &param.parameter.annotation {
        result.push_str(": ");
        result.push_str(&format_annotation(annotation));
    }

    // Add default value if present
    if let Some(default) = &param.default {
        result.push('=');
        result.push_str(&format_default(default));
    }

    result
}

pub fn format_annotation(expr: &Expr) -> String {
    match expr {
        Expr::Name(name) => name.id.as_str().to_string(),
        Expr::Attribute(attr) => {
            format!("{}.{}", format_annotation(&attr.value), attr.attr.as_str())
        }
        Expr::Subscript(sub) => {
            format!(
                "{}[{}]",
                format_annotation(&sub.value),
                format_annotation(&sub.slice)
            )
        }
        Expr::Tuple(tuple) => {
            let items: Vec<String> = tuple.elts.iter().map(format_annotation).collect();
            items.join(", ")
        }
        Expr::List(list) => {
            let items: Vec<String> = list.elts.iter().map(format_annotation).collect();
            format!("[{}]", items.join(", "))
        }
        Expr::BinOp(binop) => {
            // Handle union types (e.g., str | None)
            format!(
                "{} | {}",
                format_annotation(&binop.left),
                format_annotation(&binop.right)
            )
        }
        Expr::NoneLiteral(_) => "None".to_string(),
        Expr::EllipsisLiteral(_) => "...".to_string(),
        Expr::StringLiteral(str_lit) => {
            // For Literal['string'] types
            if let Some(single) = str_lit.as_single_part_string() {
                format!("'{}'", single.as_str())
            } else {
                "'...'".to_string()
            }
        }
        Expr::BooleanLiteral(bool_lit) => if bool_lit.value { "True" } else { "False" }.to_string(),
        _ => "...".to_string(), // Fallback for truly complex expressions
    }
}

fn format_default(expr: &Expr) -> String {
    // Format default values
    match expr {
        Expr::NoneLiteral(_) => "None".to_string(),
        Expr::BooleanLiteral(bool_lit) => if bool_lit.value { "True" } else { "False" }.to_string(),
        Expr::NumberLiteral(num_lit) => match &num_lit.value {
            ruff_python_ast::Number::Int(i) => i.to_string(),
            ruff_python_ast::Number::Float(f) => f.to_string(),
            ruff_python_ast::Number::Complex { real, imag } => format!("{real}+{imag}j"),
        },
        Expr::StringLiteral(str_lit) => {
            if let Some(single) = str_lit.as_single_part_string() {
                format!("\"{}\"", single.as_str())
            } else {
                "\"...\"".to_string()
            }
        }
        Expr::Name(name) => name.id.as_str().to_string(),
        Expr::List(_) => "[]".to_string(),
        Expr::Dict(_) => "{}".to_string(),
        Expr::Tuple(tuple) if tuple.elts.is_empty() => "()".to_string(),
        _ => "...".to_string(), // Complex defaults shown as ellipsis
    }
}

// ===== Signature Discovery & Display =====

/// Split parameters string respecting nested brackets
fn split_parameters(params: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut in_quotes = false;
    let mut prev_char = '\0';
    
    for ch in params.chars() {
        match ch {
            '\'' | '"' if prev_char != '\\' => in_quotes = !in_quotes,
            '[' | '(' | '{' if !in_quotes => depth += 1,
            ']' | ')' | '}' if !in_quotes => depth -= 1,
            ',' if depth == 0 && !in_quotes => {
                // Found a top-level comma
                result.push(current.trim().to_string());
                current.clear();
                prev_char = ch;
                continue;
            }
            _ => {}
        }
        current.push(ch);
        prev_char = ch;
    }
    
    // Don't forget the last parameter
    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }
    
    result
}

/// Recursively search for a signature in module info
fn find_signature_recursive<'a>(
    module_info: &'a ModuleInfo,
    name: &str,
) -> Option<&'a FunctionSignature> {
    // Check current module
    if let Some(sig) = module_info.signatures.get(name) {
        return Some(sig);
    }

    // Check submodules
    for submod in module_info.submodules.values() {
        if let Some(sig) = find_signature_recursive(submod, name) {
            return Some(sig);
        }
    }

    None
}

/// Format a signature for display
pub fn format_signature_display(sig: &FunctionSignature) -> String {
    let config = DisplayConfig::get();
    let mut result = format!(
        "{} {}\n",
        colorize(
            &config.signature_icon,
            &config.color_scheme.signature_color,
            config
        ),
        colorize(&sig.name, &config.color_scheme.signature_color, config)
    );
    result.push_str(&format!(
        "{} Parameters:\n",
        colorize(&config.tree_branch, &config.color_scheme.tree_color, config)
    ));

    // Format parameters
    if sig.parameters.is_empty() {
        result.push_str(&format!(
            "{} (no parameters)",
            colorize(&config.tree_last, &config.color_scheme.tree_color, config)
        ));
    } else {
        // Split parameters and format each one
        let params = split_parameters(&sig.parameters);
        for (i, param) in params.iter().enumerate() {
            let is_last = i == params.len() - 1 && sig.return_type.is_none();
            let prefix = if is_last {
                &config.tree_last
            } else {
                &config.tree_branch
            };
            result.push_str(&format!(
                "{} {}\n",
                colorize(prefix, &config.color_scheme.tree_color, config),
                colorize(param, &config.color_scheme.param_color, config)
            ));
        }
    }

    // Add return type if present
    if let Some(return_type) = &sig.return_type {
        result.push_str(&format!(
            "{} Returns:\n",
            colorize(&config.tree_last, &config.color_scheme.tree_color, config)
        ));
        result.push_str(&format!(
            "    {} {}",
            colorize(&config.tree_last, &config.color_scheme.tree_color, config),
            colorize(return_type, &config.color_scheme.type_color, config)
        ));
    }

    result
}

/// Result of signature discovery
pub struct SignatureResult {
    pub signature: Option<FunctionSignature>,
    #[allow(dead_code)]
    pub formatted_output: String,
}

/// Try to get signature from AST parsing
pub fn try_ast_signature(py: Python, import_path: &str, quiet: bool) -> Option<SignatureResult> {
    // Parse the full specification first
    let (package_override, path_without_package, version) =
        crate::utils::parse_full_spec(import_path);

    // Parse the import path to extract module and object name
    let (module_path, object_name) = if path_without_package.contains(':') {
        let parts: Vec<&str> = path_without_package.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        (parts[0], parts[1])
    } else if let Some(dot_pos) = path_without_package.rfind('.') {
        (
            &path_without_package[..dot_pos],
            &path_without_package[dot_pos + 1..],
        )
    } else {
        return None;
    };

    // Helper function to try exploration and get signature
    let try_get_signature = |py: Python| -> Option<FunctionSignature> {
        // For builtin modules (implemented in C), we can't extract signatures from filesystem
        if crate::stdlib::is_builtin_module(module_path) {
            return None;
        }

        // First try the exact module path
        let explorer = crate::explorer::ModuleTreeExplorer::new(module_path.to_string(), 2);
        if let Ok(module_info) = explorer.explore_module_pure_filesystem(py, module_path) {
            if let Some(sig) = module_info.signatures.get(object_name) {
                return Some(sig.clone());
            }

            // Check if it's in __all__ and search recursively
            if let Some(all_exports) = &module_info.all_exports {
                if all_exports.contains(&object_name.to_string()) {
                    // Use the recursive search function to find it anywhere in the tree
                    if let Some(sig) = find_signature_recursive(&module_info, object_name) {
                        return Some(sig.clone());
                    }
                }
            }
        }

        // If not found in the module, try the base package exploration
        if module_path.contains('.') {
            // Try the root package
            let root_package = module_path.split('.').next().unwrap();
            let explorer = crate::explorer::ModuleTreeExplorer::new(root_package.to_string(), 3);
            if let Ok(root_info) = explorer.explore_module_pure_filesystem(py, root_package) {
                // Search recursively for the object
                if let Some(sig) = find_signature_recursive(&root_info, object_name) {
                    return Some(sig.clone());
                }
            }
        }

        None
    };

    // First try direct filesystem exploration
    if let Some(sig) = try_get_signature(py) {
        return Some(SignatureResult {
            signature: Some(sig.clone()),
            formatted_output: format_signature_display(&sig),
        });
    }

    // If not found directly, try following import chains for known patterns
    let resolver = ImportChainResolver::new();
    if let Some(sig) = resolver.resolve_symbol_signature(py, module_path, object_name) {
        return Some(SignatureResult {
            signature: Some(sig.clone()),
            formatted_output: format_signature_display(&sig),
        });
    }

    // Check if this is a stdlib module - if so, don't try to download
    if crate::stdlib::is_stdlib_module(module_path) {
        return None;
    }

    // If not found and not stdlib, try downloading the package
    let download_package = if let Some(pkg) = package_override {
        pkg
    } else {
        crate::utils::extract_base_package(module_path)
    };

    let download_spec = if let Some(v) = version {
        format!("{}@{}", download_package, v)
    } else {
        download_package.to_string()
    };

    // Try downloading (message is printed by try_download_and_import)
    // Need to capture the result inside the closure while sys.path is modified
    let mut download_result = None;
    if let Ok(()) = crate::utils::try_download_and_import(py, &download_spec, quiet, || {
        download_result = try_get_signature(py);
        Ok(())
    }) {
        if let Some(sig) = download_result {
            return Some(SignatureResult {
                signature: Some(sig.clone()),
                formatted_output: format_signature_display(&sig),
            });
        }
    }

    None
}

/// Display a function signature
#[allow(dead_code)]
pub fn display_signature(py: Python, import_path: &str, quiet: bool) -> PyResult<String> {
    // First try to get signature from AST
    if let Some(result) = try_ast_signature(py, import_path, quiet) {
        return Ok(result.formatted_output);
    }

    // If AST parsing didn't find it, return a simple message
    let config = DisplayConfig::get();
    let object_name = if import_path.contains(':') {
        import_path.split(':').last().unwrap_or(import_path)
    } else {
        import_path.split('.').last().unwrap_or(import_path)
    };

    Ok(format!(
        "{} {} (signature not available)",
        colorize(
            &config.signature_icon,
            &config.color_scheme.signature_color,
            config
        ),
        colorize(object_name, &config.color_scheme.signature_color, config)
    ))
}
