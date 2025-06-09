mod explorer;
mod module_info;

use crate::explorer::ModuleTreeExplorer;
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
    // Try to import the object using import_object which handles both syntaxes
    let func = match import_object(py, import_path) {
        Ok(obj) => obj,
        Err(e) => {
            return Ok(format!("Error: Could not import {}: {}", import_path, e));
        }
    };
    
    // Check if callable
    let builtins = py.import("builtins")?;
    let is_callable = builtins.getattr("callable")?.call1((&func,))?.extract::<bool>()?;
    if !is_callable {
        return Ok(format!("Error: Imported object {} is not callable", import_path));
    }
    
    // Get the function name
    let func_name = if let Ok(name) = func.bind(py).getattr("__name__") {
        name.extract::<String>().unwrap_or_else(|_| "unknown".to_string())
    } else {
        // Extract name from import path
        import_path.split(&[':', '.'][..]).last().unwrap_or("unknown").to_string()
    };
    
    let inspect = py.import("inspect")?;
    match inspect.getattr("signature")?.call1((&func,)) {
        Ok(sig) => {
            // Parse the signature to extract parameters
            let sig_str = sig.to_string();
            
            // Build the formatted output
            let mut result = format!("📎 {}\n", func_name);
            
            // Simple parsing - just show the signature with Parameters: prefix
            if sig_str.contains('(') && sig_str.contains(')') {
                result.push_str("├── Parameters:\n");
                
                // Extract parameters from signature string
                let params_start = sig_str.find('(').unwrap_or(0) + 1;
                let params_end = sig_str.rfind(')').unwrap_or(sig_str.len());
                let params_str = &sig_str[params_start..params_end];
                
                if !params_str.is_empty() {
                    let params: Vec<&str> = params_str.split(", ").collect();
                    for (i, param) in params.iter().enumerate() {
                        if i == params.len() - 1 {
                            result.push_str(&format!("└── {}", param));
                        } else {
                            result.push_str(&format!("├── {}\n", param));
                        }
                    }
                } else {
                    result.push_str("└── (no parameters)");
                }
            } else {
                result.push_str(&format!("└── {}", sig_str));
            }
            
            Ok(result)
        }
        Err(_) => Ok(format!("📎 {} (signature unavailable)", func_name)),
    }
}

/// Import an object from a module path
#[pyfunction]
fn import_object(py: Python, import_path: &str) -> PyResult<PyObject> {
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
                        obj = obj.bind(py).getattr(attr).map_err(|_| {
                            PyErr::new::<pyo3::exceptions::PyImportError, _>(format!(
                                "cannot import name '{}' from '{}'",
                                attr,
                                parts[..i].join(".")
                            ))
                        })?.into();
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

/// Format tree display for wrapped format (with api/submodules structure)
pub(crate) fn format_tree_display(py: Python, tree: &PyObject, module_name: &str) -> PyResult<String> {
    let tree_dict: std::collections::HashMap<String, pyo3::PyObject> = tree.extract(py)?;
    
    let mut result = format!("📦 {}\n", module_name);
    
    // Check if there are submodules
    let has_submodules = tree_dict.get("submodules")
        .and_then(|s| s.extract::<std::collections::HashMap<String, pyo3::PyObject>>(py).ok())
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    
    // Extract the api dict
    if let Some(api) = tree_dict.get("api") {
        let api_dict: std::collections::HashMap<String, pyo3::PyObject> = api.extract(py)?;
        
        let mut items: Vec<String> = Vec::new();
        
        // Add __all__ if present
        if let Some(all_exports) = api_dict.get("all") {
            let exports: Vec<String> = all_exports.extract(py)?;
            if !exports.is_empty() {
                items.push(format!("📜 __all__: {}", exports.join(", ")));
            }
        }
        
        // functions
        if let Some(functions) = api_dict.get("functions") {
            let funcs: Vec<String> = functions.extract(py)?;
            if !funcs.is_empty() {
                items.push(format!("⚡ functions: {}", funcs.join(", ")));
            }
        }
        
        // classes
        if let Some(classes) = api_dict.get("classes") {
            let cls: Vec<String> = classes.extract(py)?;
            if !cls.is_empty() {
                items.push(format!("🔷 classes: {}", cls.join(", ")));
            }
        }
        
        // constants
        if let Some(constants) = api_dict.get("constants") {
            let consts: Vec<String> = constants.extract(py)?;
            if !consts.is_empty() {
                items.push(format!("📌 constants: {}", consts.join(", ")));
            }
        }
        
        // Print items
        for (i, item) in items.iter().enumerate() {
            let is_last = i == items.len() - 1 && !has_submodules;
            let prefix = if is_last { "└── " } else { "├── " };
            result.push_str(&format!("{}{}\n", prefix, item));
        }
    }
    
    // submodules
    if let Some(submodules) = tree_dict.get("submodules") {
        let submods: std::collections::HashMap<String, pyo3::PyObject> = submodules.extract(py)?;
        let mut submod_names: Vec<_> = submods.keys().cloned().collect();
        submod_names.sort();
        
        if !submod_names.is_empty() {
            for (i, name) in submod_names.iter().enumerate() {
                let is_last = i == submod_names.len() - 1;
                let prefix = if is_last { "└── " } else { "├── " };
                result.push_str(&format!("{}📦 {}\n", prefix, name));
                
                if let Some(submod_tree) = submods.get(name) {
                    let submod_content = format_tree_recursive(
                        py,
                        submod_tree,
                        if is_last { "    " } else { "│   " },
                    )?;
                    result.push_str(&submod_content);
                }
            }
        }
    }
    
    Ok(result)
}

fn format_tree_recursive(
    py: Python,
    tree: &PyObject,
    prefix: &str,
) -> PyResult<String> {
    let tree_dict: std::collections::HashMap<String, pyo3::PyObject> = tree.extract(py)?;
    
    let mut result = String::new();
    
    // Extract the api dict
    if let Some(api) = tree_dict.get("api") {
        let api_dict: std::collections::HashMap<String, pyo3::PyObject> = api.extract(py)?;
        
        let mut items: Vec<String> = Vec::new();
        
        // Add __all__ if present
        if let Some(all_exports) = api_dict.get("all") {
            let exports: Vec<String> = all_exports.extract(py)?;
            if !exports.is_empty() {
                items.push(format!("📜 __all__: {}", exports.join(", ")));
            }
        }
        
        // functions
        if let Some(functions) = api_dict.get("functions") {
            let funcs: Vec<String> = functions.extract(py)?;
            if !funcs.is_empty() {
                items.push(format!("⚡ functions: {}", funcs.join(", ")));
            }
        }
        
        // classes
        if let Some(classes) = api_dict.get("classes") {
            let cls: Vec<String> = classes.extract(py)?;
            if !cls.is_empty() {
                items.push(format!("🔷 classes: {}", cls.join(", ")));
            }
        }
        
        // constants
        if let Some(constants) = api_dict.get("constants") {
            let consts: Vec<String> = constants.extract(py)?;
            if !consts.is_empty() {
                items.push(format!("📌 constants: {}", consts.join(", ")));
            }
        }
        
        // Check if there are submodules
        let has_submodules = tree_dict.get("submodules")
            .and_then(|s| s.extract::<std::collections::HashMap<String, pyo3::PyObject>>(py).ok())
            .map(|s| !s.is_empty())
            .unwrap_or(false);
        
        // Print items
        for (i, item) in items.iter().enumerate() {
            let is_last = i == items.len() - 1 && !has_submodules;
            let item_prefix = if is_last { "└── " } else { "├── " };
            result.push_str(&format!("{}{}{}\n", prefix, item_prefix, item));
        }
    }
    
    // Process submodules recursively
    if let Some(submodules) = tree_dict.get("submodules") {
        let submods: std::collections::HashMap<String, pyo3::PyObject> = submodules.extract(py)?;
        let mut submod_names: Vec<_> = submods.keys().cloned().collect();
        submod_names.sort();
        
        for (i, name) in submod_names.iter().enumerate() {
            let is_last = i == submod_names.len() - 1;
            let submod_prefix = if is_last { "└── " } else { "├── " };
            
            result.push_str(&format!("{}{}📦 {}\n", prefix, submod_prefix, name));
            
            if let Some(submod_tree) = submods.get(name) {
                let submod_content = format_tree_recursive(
                    py,
                    submod_tree,
                    &format!("{}{}", prefix, if is_last { "    " } else { "│   " }),
                )?;
                result.push_str(&submod_content);
            }
        }
    }
    
    Ok(result)
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
