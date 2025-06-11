mod explorer;
mod module_info;
mod package_downloader;
mod signature;
mod tree_formatter;
mod utils;

use crate::explorer::ModuleTreeExplorer;
use crate::signature::display_signature as display_signature_impl;
use crate::tree_formatter::format_tree_display;
use crate::utils::{extract_base_package, try_download_and_import, import_object_impl};
use pyo3::prelude::*;

/// Display a module tree
#[pyfunction]
#[pyo3(signature = (root_module_path, max_depth = 2, quiet = false))]
fn display_tree(py: Python, root_module_path: &str, max_depth: usize, quiet: bool) -> PyResult<()> {
    // Check for invalid package name (contains colon)
    if root_module_path.contains(':') {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Invalid module path '{}': use 'pretty-mod sig' for exploring specific objects", root_module_path)
        ));
    }
    
    // Parse package@version syntax
    let (module_name, _version) = utils::parse_package_spec(root_module_path);
    
    // Remove any PEP 508 version specifiers
    let module_name = module_name
        .split(&['[', '>', '<', '=', '!'][..])
        .next()
        .unwrap_or(module_name)
        .trim();
    
    // First check if module can be imported
    let import_result = py.import(module_name);
    
    match import_result {
        Ok(_) => {
            // Module exists, do normal exploration
            let explorer = ModuleTreeExplorer::new(module_name.to_string(), max_depth);
            match explorer.explore(py) {
                Ok(tree) => {
                    // Display tree using the wrapped format
                    let tree_str = format_tree_display(py, &tree, module_name)?;
                    println!("{}", tree_str);
                    Ok(())
                }
                Err(e) => Err(e)
            }
        }
        Err(e) => {
            // Check if it's a module not found error
            let err_str = e.to_string();
            if err_str.contains("No module named") || err_str.contains("ModuleNotFoundError") {
                // Extract the base package name and preserve version spec if present
                let base_package = extract_base_package(module_name);
                let download_spec = if let (_, Some(version)) = utils::parse_package_spec(root_module_path) {
                    format!("{}@{}", base_package, version)
                } else {
                    base_package.to_string()
                };
                
                // Try downloading and importing the base package
                try_download_and_import(py, &download_spec, quiet, || {
                    // Try exploration again with the full module path
                    let explorer = ModuleTreeExplorer::new(module_name.to_string(), max_depth);
                    match explorer.explore(py) {
                        Ok(tree) => {
                            let tree_str = format_tree_display(py, &tree, module_name)?;
                            println!("{}", tree_str);
                            Ok(())
                        }
                        Err(e) => Err(e)
                    }
                })
            } else {
                Err(e)
            }
        }
    }
}

/// Display a function signature
#[pyfunction]
#[pyo3(signature = (import_path, quiet = false))]
fn display_signature(py: Python, import_path: &str, quiet: bool) -> PyResult<String> {
    display_signature_impl(py, import_path, quiet)
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
