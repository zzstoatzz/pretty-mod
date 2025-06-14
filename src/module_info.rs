use crate::{semantic, signature};
use pyo3::prelude::*;
use ruff_python_ast::{Expr, ExprList, ExprName, Mod, Stmt, StmtAssign};
use ruff_python_parser::{parse, Mode};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Function signature information
#[derive(Serialize, Deserialize, Clone, Debug, IntoPyObject)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: String,
    pub return_type: Option<String>,
}

/// Import information tracking where symbols come from
#[derive(Serialize, Deserialize, Clone, Debug, IntoPyObject)]
pub struct ImportInfo {
    pub from_module: Option<String>,  // e.g., ".main" for "from .main import BaseModel"
    pub import_name: String,          // e.g., "BaseModel"
    pub as_name: Option<String>,      // e.g., "Model" for "import BaseModel as Model"
    pub is_relative: bool,            // true for "from .main import"
}

/// Rust representation of module information
#[derive(Serialize, Deserialize, Clone, Debug, Default, IntoPyObject)]
pub struct ModuleInfo {
    pub functions: Vec<String>,
    pub classes: Vec<String>,
    pub constants: Vec<String>,
    pub imports: Vec<String>,
    pub submodules: HashMap<String, ModuleInfo>,
    pub all_exports: Option<Vec<String>>,
    pub signatures: HashMap<String, FunctionSignature>,
    pub import_map: HashMap<String, ImportInfo>,  // Maps symbol name to where it's imported from
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
            signatures: HashMap::new(),
            import_map: HashMap::new(),
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
                "Expected a module",
            ));
        };

        // Try enhanced semantic analysis first
        let mut analyzer = semantic::SemanticAnalyzer::new();
        if analyzer.analyze_file(file_path).is_ok() {
            // Extract signatures using semantic analysis (includes methods!)
            if analyzer.extract_module_info(&mut info).is_ok() {
                // Semantic analysis succeeded - we now have method signatures too
            }
        }

        // Parse AST and collect module information
        // This is only used as a fallback when Python import fails
        let mut raw_functions = Vec::new();
        let mut raw_classes = Vec::new();
        let mut raw_constants = Vec::new();

        // Helper function to process statements recursively
        fn process_statements(stmts: &[Stmt], info: &mut ModuleInfo, raw_functions: &mut Vec<String>, raw_classes: &mut Vec<String>, raw_constants: &mut Vec<String>) {
            for stmt in stmts {
                process_statement(stmt, info, raw_functions, raw_classes, raw_constants);
            }
        }
        
        fn process_statement(stmt: &Stmt, info: &mut ModuleInfo, raw_functions: &mut Vec<String>, raw_classes: &mut Vec<String>, raw_constants: &mut Vec<String>) {
            match stmt {
                Stmt::FunctionDef(func_def) => {
                    if !func_def.name.as_str().starts_with('_') {
                        let name_str = func_def.name.to_string();
                        raw_functions.push(name_str.clone());

                        // Extract signature
                        let parameters = signature::format_parameters(&func_def.parameters);
                        let return_type = func_def
                            .returns
                            .as_ref()
                            .map(|ret| signature::format_annotation(ret));

                        info.signatures.insert(
                            name_str.clone(),
                            FunctionSignature {
                                name: name_str,
                                parameters,
                                return_type,
                            },
                        );
                    }
                }
                Stmt::ClassDef(class_def) => {
                    if !class_def.name.as_str().starts_with('_') {
                        let class_name = class_def.name.to_string();
                        raw_classes.push(class_name.clone());

                        // Look for __init__ method to get constructor signature
                        for stmt in &class_def.body {
                            if let Stmt::FunctionDef(func_def) = stmt {
                                if func_def.name.as_str() == "__init__" {
                                    let parameters =
                                        signature::format_parameters(&func_def.parameters);
                                    // Store class constructor signature
                                    info.signatures.insert(
                                        class_name.clone(),
                                        FunctionSignature {
                                            name: class_name.clone(),
                                            parameters,
                                            return_type: None, // Constructors don't have explicit return types
                                        },
                                    );
                                    break;
                                }
                            }
                        }
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
                                            if let Some(single) = string_lit.as_single_part_string()
                                            {
                                                all_items.push(single.as_str().to_string());
                                            }
                                        }
                                    }
                                    if !all_items.is_empty() {
                                        info.all_exports = Some(all_items);
                                    }
                                }
                            } else if id.as_str().chars().all(|c| c.is_uppercase() || c == '_')
                                && !id.as_str().starts_with('_')
                            {
                                // This is a constant (all uppercase)
                                raw_constants.push(id.to_string());
                            }
                        }
                    }
                }
                Stmt::Import(import) => {
                    // Handle "import module" statements
                    for alias in &import.names {
                        let import_name = alias.name.as_str().to_string();
                        let as_name = alias.asname.as_ref().map(|n| n.as_str().to_string());
                        let final_name = as_name.as_ref().unwrap_or(&import_name);
                        
                        info.import_map.insert(
                            final_name.clone(),
                            ImportInfo {
                                from_module: None,
                                import_name,
                                as_name,
                                is_relative: false,
                            },
                        );
                    }
                }
                Stmt::ImportFrom(import_from) => {
                    // Handle "from module import ..." statements
                    let from_module = import_from.module.as_ref().map(|m| m.to_string());
                    let is_relative = import_from.level > 0;
                    
                    for alias in &import_from.names {
                        let import_name = alias.name.as_str().to_string();
                        let as_name = alias.asname.as_ref().map(|n| n.as_str().to_string());
                        let final_name = as_name.as_ref().unwrap_or(&import_name);
                        
                        info.import_map.insert(
                            final_name.clone(),
                            ImportInfo {
                                from_module: from_module.clone(),
                                import_name,
                                as_name,
                                is_relative,
                            },
                        );
                    }
                }
                Stmt::If(if_stmt) => {
                    // Process statements inside if blocks (e.g., if TYPE_CHECKING:)
                    process_statements(&if_stmt.body, info, raw_functions, raw_classes, raw_constants);
                    // Process elif and else clauses
                    for clause in &if_stmt.elif_else_clauses {
                        process_statements(&clause.body, info, raw_functions, raw_classes, raw_constants);
                    }
                }
                _ => {}
            }
        }
        
        // Process all statements in the module
        process_statements(&module.body, &mut info, &mut raw_functions, &mut raw_classes, &mut raw_constants);

        // Apply __all__ filter if present
        if let Some(ref all_exports) = info.all_exports {
            let export_set: HashSet<&str> = all_exports.iter().map(|s| s.as_str()).collect();
            info.functions = raw_functions
                .into_iter()
                .filter(|f| export_set.contains(f.as_str()))
                .collect();
            info.classes = raw_classes
                .into_iter()
                .filter(|c| export_set.contains(c.as_str()))
                .collect();
            info.constants = raw_constants
                .into_iter()
                .filter(|c| export_set.contains(c.as_str()))
                .collect();
        } else {
            info.functions = raw_functions;
            info.classes = raw_classes;
            info.constants = raw_constants;
        }

        Ok(info)
    }
}
