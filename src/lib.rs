mod config;
mod explorer;
mod import_resolver;
mod module_info;
mod output_format;
mod package_downloader;
mod semantic;
mod signature;
mod stdlib;
mod tree_formatter;
mod utils;

use crate::explorer::ModuleTreeExplorer;
use crate::output_format::create_formatter;
use crate::utils::{extract_base_package, try_download_and_import, import_object_impl};
use pyo3::prelude::*;

/// Display a module tree
#[pyfunction]
#[pyo3(signature = (root_module_path, max_depth = 2, quiet = false, format = "pretty"))]
fn display_tree(py: Python, root_module_path: &str, max_depth: usize, quiet: bool, format: &str) -> PyResult<()> {
    let formatter = create_formatter(format);
    // Check for invalid single colon (but allow double colon)
    if root_module_path.contains(':') && !root_module_path.contains("::") {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Invalid module path '{}': use 'pretty-mod sig' for exploring specific objects", root_module_path)
        ));
    }
    
    // Parse the full specification
    let (package_override, module_path, version) = utils::parse_full_spec(root_module_path);
    
    // Remove any PEP 508 version specifiers from module path
    let module_name = module_path
        .split(&['[', '>', '<', '=', '!'][..])
        .next()
        .unwrap_or(module_path)
        .trim();
    
    // Try to explore the module directly first
    let explorer = ModuleTreeExplorer::new(module_name.to_string(), max_depth);
    match explorer.explore(py) {
        Ok(tree) => {
            // Display tree using the formatter
            let tree_str = formatter.format_tree(py, &tree, module_name)?;
            println!("{}", tree_str);
            Ok(())
        }
        Err(e) => {
            // Check if it's a module not found error
            let err_str = e.to_string();
            if err_str.contains("No module named") || err_str.contains("ModuleNotFoundError") {
                // Determine which package to download
                let download_package = if let Some(pkg) = package_override {
                    // Use the explicit package name
                    pkg
                } else {
                    // Extract the base package name from module
                    extract_base_package(module_name)
                };
                
                // Build download spec with version if present
                let download_spec = if let Some(v) = version {
                    format!("{}@{}", download_package, v)
                } else {
                    download_package.to_string()
                };
                
                // Try downloading and importing the package
                match try_download_and_import(py, &download_spec, quiet, || {
                    // Try exploration again with the full module path
                    let explorer = ModuleTreeExplorer::new(module_name.to_string(), max_depth);
                    match explorer.explore(py) {
                        Ok(tree) => {
                            let tree_str = formatter.format_tree(py, &tree, module_name)?;
                            println!("{}", tree_str);
                            Ok(())
                        }
                        Err(e) => Err(e)
                    }
                }) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        let err_str = e.to_string();
                        if err_str.contains("No module named") {
                            let missing = err_str
                                .split("No module named")
                                .nth(1)
                                .unwrap_or("")
                                .trim()
                                .trim_matches('\'')
                                .trim_matches('"')
                                .split('.')
                                .next()
                                .unwrap_or("");
                            
                            if !missing.is_empty() {
                                println!("Cannot explore {}: missing dependency '{}'", module_name, missing);
                                return Ok(());
                            }
                        }
                        println!("Cannot explore {}", module_name);
                        Ok(())
                    }
                }
            } else {
                Err(e)
            }
        }
    }
}

/// Display a function signature
#[pyfunction]
#[pyo3(signature = (import_path, quiet = false, format = "pretty"))]
fn display_signature(py: Python, import_path: &str, quiet: bool, format: &str) -> PyResult<String> {
    use crate::signature::try_ast_signature;
    let formatter = create_formatter(format);
    
    // First try to get signature from AST
    if let Some(result) = try_ast_signature(py, import_path, quiet) {
        if let Some(ref sig) = result.signature {
            return Ok(formatter.format_signature(sig));
        }
    }
    
    // If AST parsing didn't find it, return a simple message
    let object_name = if import_path.contains(':') {
        import_path.split(':').last().unwrap_or(import_path)
    } else {
        import_path.split('.').last().unwrap_or(import_path)
    };
    
    Ok(formatter.format_signature_not_available(object_name))
}

/// Import an object from a module path (public API, no auto-download)
#[pyfunction]
pub fn import_object(py: Python, import_path: &str) -> PyResult<PyObject> {
    import_object_impl(py, import_path)
}


#[pymodule]
#[pyo3(name = "_pretty_mod")]
fn pretty_mod(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ModuleTreeExplorer>()?;
    m.add_function(wrap_pyfunction!(display_tree, m)?)?;
    m.add_function(wrap_pyfunction!(display_signature, m)?)?;
    m.add_function(wrap_pyfunction!(import_object, m)?)?;
    Ok(())
}
