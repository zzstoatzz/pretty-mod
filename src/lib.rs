mod explorer;
mod module_info;
mod package_downloader;
mod signature;
mod tree_formatter;

use crate::explorer::ModuleTreeExplorer;
use crate::package_downloader::{download_package, DownloadedPackage};
use crate::signature::display_signature as display_signature_impl;
use crate::tree_formatter::format_tree_display;
use pyo3::prelude::*;

/// Display a module tree
#[pyfunction]
#[pyo3(signature = (root_module_path, max_depth = 2))]
fn display_tree(py: Python, root_module_path: &str, max_depth: usize) -> PyResult<()> {
    // Create explorer and display via Python formatting
    let explorer = ModuleTreeExplorer::new(root_module_path.to_string(), max_depth);
    let tree = explorer.explore(py)?;

    // Display tree using the wrapped format
    let tree_str = format_tree_display(py, &tree, root_module_path)?;
    py.import("builtins")?
        .getattr("print")?
        .call1((tree_str,))?;
    Ok(())
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
    m.add_class::<DownloadedPackage>()?;
    m.add_function(wrap_pyfunction!(display_tree, m)?)?;
    m.add_function(wrap_pyfunction!(display_signature, m)?)?;
    m.add_function(wrap_pyfunction!(import_object, m)?)?;
    m.add_function(wrap_pyfunction!(download_package, m)?)?;
    Ok(())
}
