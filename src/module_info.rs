use pyo3::prelude::*;
use ruff_python_parser::{parse, Mode};
use ruff_python_ast::{Mod, Stmt, StmtFunctionDef, StmtClassDef, StmtAssign, Expr, ExprName, ExprList};
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
    pub imports: Vec<String>,
    pub submodules: HashMap<String, ModuleInfo>,
    pub all_exports: Option<Vec<String>>,
}

impl ModuleInfo {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            classes: Vec::new(),
            constants: Vec::new(),
            imports: Vec::new(),
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


        let parsed = parse(&source, Mode::Module.into()).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PySyntaxError, _>(format!(
                "Failed to parse {}: {:?}",
                file_path.display(),
                e
            ))
        })?;
        
        let Mod::Module(module) = parsed.into_syntax() else {
            return Err(PyErr::new::<pyo3::exceptions::PySyntaxError, _>(
                "Expected a module"
            ));
        };

        // Parse AST and collect module information
        // This is only used as a fallback when Python import fails
        let mut raw_functions = Vec::new();
        let mut raw_classes = Vec::new();
        let mut raw_constants = Vec::new();
        
        for stmt in &module.body {
            match stmt {
                Stmt::FunctionDef(StmtFunctionDef { name, .. }) => {
                    if !name.as_str().starts_with('_') {
                        raw_functions.push(name.to_string());
                    }
                }
                Stmt::ClassDef(StmtClassDef { name, .. }) => {
                    if !name.as_str().starts_with('_') {
                        raw_classes.push(name.to_string());
                    }
                }
                Stmt::Assign(StmtAssign { targets, value, .. }) => {
                    // Look for __all__ assignments
                    if targets.len() == 1 {
                        if let Expr::Name(ExprName { id, .. }) = &targets[0] {
                            if id.as_str() == "__all__" {
                                if let Expr::List(ExprList { elts, .. }) = value.as_ref() {
                                    let mut all_items = Vec::new();
                                    for elt in elts {
                                        if let Expr::StringLiteral(string_lit) = elt {
                                            all_items.push(string_lit.value.to_string());
                                        }
                                    }
                                    if !all_items.is_empty() {
                                        info.all_exports = Some(all_items);
                                    }
                                }
                            } else if id.as_str().chars().all(|c| c.is_uppercase() || c == '_') 
                                && !id.as_str().starts_with('_') {
                                // This is a constant (all uppercase)
                                raw_constants.push(id.to_string());
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

}
