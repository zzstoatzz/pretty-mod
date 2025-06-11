use ruff_python_ast::{self as ast, visitor::Visitor};
use ruff_python_parser::parse_module;
use std::path::Path;
use std::collections::HashMap;

use crate::module_info::{FunctionSignature, ModuleInfo};

/// Enhanced semantic analysis using ruff's AST visitor pattern
/// This approach uses what's publicly available from ruff crates
pub struct SemanticAnalyzer {
    /// Track the current scope stack to classify functions vs methods
    scope_stack: Vec<ScopeContext>,
    /// Map of function/method signatures found
    signatures: HashMap<String, FunctionSignature>,
    /// Import information found in the module
    imports: Vec<ImportInfo>,
}

#[derive(Debug, Clone)]
enum ScopeContext {
    Module,
    Class(String),
    Function(String),
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scope_stack: vec![ScopeContext::Module],
            signatures: HashMap::new(),
            imports: Vec::new(),
        }
    }

    /// Analyze a Python file using AST visitor pattern
    pub fn analyze_file(&mut self, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let source_code = std::fs::read_to_string(file_path)?;
        
        // Parse using ruff's parser
        let parsed = parse_module(&source_code)?;
        
        // Visit the AST to extract semantic information
        let module = parsed.into_syntax();
        for stmt in &module.body {
            self.visit_stmt(stmt);
        }
        
        Ok(())
    }

    /// Extract enhanced module info with method classification
    pub fn extract_module_info(&self, base_info: &mut ModuleInfo) -> Result<(), Box<dyn std::error::Error>> {
        // Add all signatures we found
        for (name, signature) in &self.signatures {
            base_info.signatures.insert(name.clone(), signature.clone());
        }
        
        Ok(())
    }

    /// Check if we're currently in a class scope
    fn in_class_scope(&self) -> Option<&String> {
        for scope in self.scope_stack.iter().rev() {
            if let ScopeContext::Class(class_name) = scope {
                return Some(class_name);
            }
        }
        None
    }

    /// Get import information
    pub fn get_imports(&self) -> Vec<ImportInfo> {
        self.imports.clone()
    }
}

/// Custom visitor implementation for semantic analysis
impl Visitor<'_> for SemanticAnalyzer {
    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        match stmt {
            ast::Stmt::FunctionDef(func_def) => {
                let signature = FunctionSignature {
                    name: func_def.name.as_str().to_string(),
                    parameters: crate::signature::format_parameters(&func_def.parameters),
                    return_type: func_def.returns.as_ref()
                        .map(|ret| crate::signature::format_annotation(ret)),
                };

                // Classify based on scope context
                if let Some(_class_name) = self.in_class_scope() {
                    // This is a method - could be enhanced with decorator analysis
                    // For now, we'll treat all functions in classes as methods
                    self.signatures.insert(func_def.name.as_str().to_string(), signature);
                } else {
                    // This is a module-level function
                    self.signatures.insert(func_def.name.as_str().to_string(), signature);
                }

                // Enter function scope
                self.scope_stack.push(ScopeContext::Function(func_def.name.as_str().to_string()));
                
                // Visit function body
                for stmt in &func_def.body {
                    self.visit_stmt(stmt);
                }
                
                // Exit function scope
                self.scope_stack.pop();
            }
            ast::Stmt::ClassDef(class_def) => {
                // Enter class scope
                self.scope_stack.push(ScopeContext::Class(class_def.name.as_str().to_string()));
                
                // Visit class body - this will extract all method signatures
                for stmt in &class_def.body {
                    self.visit_stmt(stmt);
                }
                
                // Exit class scope
                self.scope_stack.pop();
            }
            ast::Stmt::Import(import_stmt) => {
                for alias in &import_stmt.names {
                    self.imports.push(ImportInfo {
                        name: alias.name.as_str().to_string(),
                        module: alias.name.as_str().to_string(),
                        is_from_import: false,
                    });
                }
            }
            ast::Stmt::ImportFrom(from_import) => {
                if let Some(module) = &from_import.module {
                    let module_name = module.as_str().to_string();
                    for alias in &from_import.names {
                        self.imports.push(ImportInfo {
                            name: alias.name.as_str().to_string(),
                            module: module_name.clone(),
                            is_from_import: true,
                        });
                    }
                }
            }
            _ => {
                // Visit other statements recursively
                ast::visitor::walk_stmt(self, stmt);
            }
        }
    }
}

/// Information about an import statement
#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub name: String,
    pub module: String,
    pub is_from_import: bool,
}