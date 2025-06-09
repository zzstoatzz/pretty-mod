use pyo3::prelude::*;
use rustpython_parser::{ast, Parse};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Rust representation of module information
#[derive(Serialize, Deserialize, Clone, Debug, IntoPyObject)]
pub struct ModuleInfo {
    pub functions: Vec<String>,
    pub classes: Vec<String>,
    pub constants: Vec<String>,
    pub submodules: HashMap<String, ModuleInfo>,
    pub all_exports: Option<Vec<String>>,
}

impl ModuleInfo {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            classes: Vec::new(),
            constants: Vec::new(),
            submodules: HashMap::new(),
            all_exports: None,
        }
    }

    /// Parse a Python file and extract module information
    pub fn from_python_file(file_path: &Path) -> PyResult<Self> {
        let mut info = ModuleInfo::new();

        let source = fs::read_to_string(file_path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to read {}: {}",
                file_path.display(),
                e
            ))
        })?;


        let suite =
            ast::Suite::parse(&source, file_path.to_string_lossy().as_ref()).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PySyntaxError, _>(format!(
                    "Failed to parse {}: {}",
                    file_path.display(),
                    e
                ))
            })?;

        // Parse AST and collect module information
        // This is only used as a fallback when Python import fails
        let mut raw_functions = Vec::new();
        let mut raw_classes = Vec::new();
        let mut raw_constants = Vec::new();
        
        for stmt in suite {
            match stmt {
                ast::Stmt::FunctionDef(func) => {
                    if !func.name.starts_with('_') {
                        raw_functions.push(func.name.to_string());
                    }
                }
                ast::Stmt::ClassDef(class) => {
                    if !class.name.starts_with('_') {
                        raw_classes.push(class.name.to_string());
                    }
                }
                ast::Stmt::Assign(assign) => {
                    // Look for __all__ assignments
                    if assign.targets.len() == 1 {
                        if let ast::Expr::Name(name) = &assign.targets[0] {
                            if name.id == *"__all__" {
                                if let ast::Expr::List(list) = assign.value.as_ref() {
                                    let mut all_items = Vec::new();
                                    for elt in &list.elts {
                                        if let ast::Expr::Constant(constant) = elt {
                                            if let ast::Constant::Str(s) = &constant.value {
                                                all_items.push(s.clone());
                                            }
                                        }
                                    }
                                    if !all_items.is_empty() {
                                        info.all_exports = Some(all_items);
                                    }
                                }
                            } else if name.id.chars().all(|c| c.is_uppercase() || c == '_') 
                                && !name.id.starts_with('_') {
                                // This is a constant (all uppercase)
                                raw_constants.push(name.id.to_string());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Apply __all__ filter if present
        if let Some(ref all_exports) = info.all_exports {
            let export_set: HashSet<&str> = all_exports.iter().map(|s| s.as_str()).collect();
            info.functions = raw_functions.into_iter().filter(|f| export_set.contains(f.as_str())).collect();
            info.classes = raw_classes.into_iter().filter(|c| export_set.contains(c.as_str())).collect();
            info.constants = raw_constants.into_iter().filter(|c| export_set.contains(c.as_str())).collect();
        } else {
            info.functions = raw_functions;
            info.classes = raw_classes;
            info.constants = raw_constants;
        }

        Ok(info)
    }

    /// Extract module info from a Python module object (API only, no submodules)
    pub fn from_python_module(py: Python, module: &Bound<'_, PyAny>) -> PyResult<Self> {
        let mut info = ModuleInfo::new();

        // Get __all__ if it exists
        if let Ok(all_attr) = module.getattr("__all__") {
            if let Ok(all_list) = all_attr.extract::<Vec<String>>() {
                info.all_exports = Some(all_list);
            }
        }

        // Get all attributes from the module
        let dir_result = py.import("builtins")?.getattr("dir")?.call1((module,))?;
        let attrs: Vec<String> = dir_result.extract()?;

        let inspect_mod = py.import("inspect")?;
        let module_name = module.getattr("__name__")?.extract::<String>()?;

        // Determine which attributes to process
        let items_to_process = if let Some(ref all_exports) = info.all_exports {
            // If __all__ exists, only process those items
            all_exports.clone()
        } else {
            // Otherwise, process all non-underscore attributes
            attrs
                .into_iter()
                .filter(|name| !name.starts_with('_'))
                .collect()
        };

        for attr_name in &items_to_process {
            if let Ok(attr) = module.getattr(attr_name) {
                // Check if the item is defined in this module (for proper categorization)
                let is_defined_here = if let Ok(item_module) = attr.getattr("__module__") {
                    if let Ok(item_module_str) = item_module.extract::<String>() {
                        item_module_str == module_name
                    } else {
                        false
                    }
                } else {
                    false
                };

                // Only categorize items that are defined here
                if is_defined_here {
                    if let Ok(true) = inspect_mod
                        .getattr("isfunction")?
                        .call1((&attr,))?
                        .extract::<bool>()
                    {
                        info.functions.push(attr_name.clone());
                    } else if let Ok(true) = inspect_mod
                        .getattr("isclass")?
                        .call1((&attr,))?
                        .extract::<bool>()
                    {
                        info.classes.push(attr_name.clone());
                    } else if attr_name.chars().all(|c| c.is_uppercase() || c == '_') {
                        // Check if it's not a module, function, or class - then it's a constant
                        if let Ok(false) = inspect_mod
                            .getattr("ismodule")?
                            .call1((&attr,))?
                            .extract::<bool>()
                        {
                            info.constants.push(attr_name.clone());
                        }
                    }
                }
            }
        }

        Ok(info)
    }
}
