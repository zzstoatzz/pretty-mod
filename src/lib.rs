mod explorer;
mod module_info;
mod package_downloader;
mod signature;
mod tree_formatter;

use crate::explorer::ModuleTreeExplorer;
use crate::signature::display_signature as display_signature_impl;
use crate::tree_formatter::format_tree_display;
use pyo3::prelude::*;

/// Display a module tree
#[pyfunction]
#[pyo3(signature = (root_module_path, max_depth = 2, quiet = false))]
fn display_tree(py: Python, root_module_path: &str, max_depth: usize, quiet: bool) -> PyResult<()> {
    // Extract base package name from version specifiers
    let base_name = root_module_path
        .split(&['[', '>', '<', '=', '!'][..])
        .next()
        .unwrap_or(root_module_path)
        .trim();
    
    // First check if module can be imported
    let import_result = py.import(base_name);
    
    match import_result {
        Ok(_) => {
            // Module exists, do normal exploration
            let explorer = ModuleTreeExplorer::new(base_name.to_string(), max_depth);
            match explorer.explore(py) {
                Ok(tree) => {
                    // Display tree using the wrapped format
                    let tree_str = format_tree_display(py, &tree, base_name)?;
                    py.import("builtins")?
                        .getattr("print")?
                        .call1((tree_str,))?;
                    Ok(())
                }
                Err(e) => Err(e)
            }
        }
        Err(e) => {
            // Check if it's a module not found error
            let err_str = e.to_string();
            if err_str.contains("No module named") || err_str.contains("ModuleNotFoundError") {
                // Try downloading the package
                if !quiet {
                    let sys = py.import("sys")?;
                    let stderr = sys.getattr("stderr")?;
                    stderr.call_method1("write", (format!("Module '{}' not found locally. Attempting to download from PyPI...\n", root_module_path),))?;
                    stderr.call_method0("flush")?;
                }
                
                // Download and extract the package
                let mut downloader = crate::package_downloader::PackageDownloader::new(root_module_path.to_string());
                let package_path = downloader.download_and_extract()?;
                
                // Add to sys.path temporarily with RAII cleanup
                let sys = py.import("sys")?;
                let sys_path = sys.getattr("path")?;
                
                // Determine the right directory to add to sys.path
                let parent_dir = if package_path.ends_with(base_name) || 
                                   package_path.ends_with(&base_name.replace('-', "_")) {
                    package_path.parent().unwrap()
                } else {
                    &package_path
                };
                
                let parent_dir_str = parent_dir.to_str().unwrap();
                sys_path.call_method1("insert", (0, parent_dir_str))?;
                
                // Create a guard that will remove the path when dropped
                struct PathGuard<'py> {
                    sys_path: &'py pyo3::Bound<'py, pyo3::PyAny>,
                    path: &'py str,
                }
                
                impl<'py> Drop for PathGuard<'py> {
                    fn drop(&mut self) {
                        // Best effort removal - don't panic in drop
                        let _ = self.sys_path.call_method1("remove", (self.path,));
                    }
                }
                
                let _guard = PathGuard { sys_path: &sys_path, path: parent_dir_str };
                
                // Try exploration again
                let explorer = ModuleTreeExplorer::new(base_name.to_string(), max_depth);
                match explorer.explore(py) {
                    Ok(tree) => {
                        let tree_str = format_tree_display(py, &tree, base_name)?;
                        py.import("builtins")?
                            .getattr("print")?
                            .call1((tree_str,))?;
                        Ok(())
                    }
                    Err(e) => Err(e)
                }
            } else {
                Err(e)
            }
        }
    }
}

/// Display a function signature
#[pyfunction]
fn display_signature(py: Python, import_path: &str) -> PyResult<String> {
    display_signature_impl(py, import_path)
}

/// Import an object from a module path
#[pyfunction]
pub fn import_object(py: Python, import_path: &str) -> PyResult<PyObject> {
    // Support both colon and dot syntax
    if import_path.contains(':') {
        // Colon syntax: module:object
        let parts: Vec<&str> = import_path.split(':').collect();
        if parts.len() != 2 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Import path must be in format 'module:object' or 'module.object'",
            ));
        }
        let (module_name, object_name) = (parts[0], parts[1]);
        let module = py.import(module_name)?;
        module.getattr(object_name)?.extract()
    } else if import_path.contains('.') {
        // Dot syntax: try to find where module ends and attribute begins
        let parts: Vec<&str> = import_path.split('.').collect();

        // Try importing progressively longer module paths
        for i in (1..parts.len()).rev() {
            let module_path = parts[..i].join(".");
            match py.import(&module_path) {
                Ok(module) => {
                    // Found the module, now get the remaining attributes
                    let mut obj: PyObject = module.into();
                    for attr in &parts[i..] {
                        obj = obj
                            .bind(py)
                            .getattr(attr)
                            .map_err(|_| {
                                PyErr::new::<pyo3::exceptions::PyImportError, _>(format!(
                                    "cannot import name '{}' from '{}'",
                                    attr,
                                    parts[..i].join(".")
                                ))
                            })?
                            .into();
                    }
                    return Ok(obj);
                }
                Err(_) => continue,
            }
        }

        // If no valid module found, it might be a top-level module
        py.import(import_path).map(|m| m.into())
    } else {
        // No dots or colons, assume it's a module name
        py.import(import_path).map(|m| m.into())
    }
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
