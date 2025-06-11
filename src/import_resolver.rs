use crate::module_info::FunctionSignature;
use pyo3::prelude::*;

/// Resolves symbols through import chains using existing infrastructure
pub struct ImportChainResolver;

impl ImportChainResolver {
    pub fn new() -> Self {
        Self
    }

    /// Try to resolve a symbol by following import chains
    /// Focus on specific patterns like prefect:flow -> prefect.flows:FlowDecorator
    pub fn resolve_symbol_signature(
        &self, 
        py: Python,
        module_path: &str, 
        symbol_name: &str
    ) -> Option<FunctionSignature> {
        // For known patterns, skip direct exploration and go straight to pattern matching
        // This handles cases where the base module is a namespace package
        if let Some(sig) = self.resolve_known_patterns(py, module_path, symbol_name) {
            return Some(sig);
        }
        
        // Fallback: try to get module info using our existing system for direct symbols
        let explorer = crate::explorer::ModuleTreeExplorer::new(module_path.to_string(), 2);
        if let Ok(module_info) = explorer.explore_module_pure_filesystem(py, module_path) {
            // Check if symbol is directly available
            if let Some(sig) = module_info.signatures.get(symbol_name) {
                return Some(sig.clone());
            }
        }
        
        None
    }

    /// Handle known import patterns for specific libraries
    fn resolve_known_patterns(
        &self,
        py: Python,
        module_path: &str,
        symbol_name: &str
    ) -> Option<FunctionSignature> {
        match (module_path, symbol_name) {
            // Prefect pattern: prefect:flow -> prefect.flows:FlowDecorator.__call__
            ("prefect", "flow") => {
                self.try_resolve_from_submodule(py, "prefect.flows", "FlowDecorator")
            }
            
            // FastMCP pattern: fastmcp:FastMCP -> fastmcp.server:FastMCP.__init__
            ("fastmcp", "FastMCP") => {
                // Try fastmcp.server first (allows download), then fastmcp (no download to avoid duplicates)
                self.try_resolve_from_submodule(py, "fastmcp.server", "FastMCP")
                    .or_else(|| self.try_resolve_from_submodule_internal(py, "fastmcp", "FastMCP", false))
            }
            
            // FastAPI pattern: fastapi:FastAPI -> fastapi.applications:FastAPI.__init__
            ("fastapi", "FastAPI") => {
                self.try_resolve_from_submodule(py, "fastapi.applications", "FastAPI")
            }
            
            // Add more patterns as needed
            _ => None
        }
    }

    /// Try to resolve a symbol from a specific submodule
    fn try_resolve_from_submodule(
        &self,
        py: Python,
        submodule_path: &str,
        class_name: &str
    ) -> Option<FunctionSignature> {
        self.try_resolve_from_submodule_internal(py, submodule_path, class_name, true)
    }

    /// Internal helper that can optionally skip download to avoid duplicates
    fn try_resolve_from_submodule_internal(
        &self,
        py: Python,
        submodule_path: &str,
        class_name: &str,
        allow_download: bool
    ) -> Option<FunctionSignature> {
        // Helper function to try exploration
        let try_get_signature = |py: Python| -> Option<FunctionSignature> {
            let explorer = crate::explorer::ModuleTreeExplorer::new(submodule_path.to_string(), 2);
            
            if let Ok(module_info) = explorer.explore_module_pure_filesystem(py, submodule_path) {
                // Try to find the class signature (which should be __init__ or __call__)
                if let Some(sig) = module_info.signatures.get(class_name) {
                    return Some(sig.clone());
                }
                
                // Also try looking for __call__ method signature
                let call_method = format!("{}.{}", class_name, "__call__");
                if let Some(sig) = module_info.signatures.get(&call_method) {
                    return Some(sig.clone());
                }
            }
            
            None
        };

        // First try direct filesystem exploration
        if let Some(sig) = try_get_signature(py) {
            return Some(sig);
        }

        // If download is not allowed, return None
        if !allow_download {
            return None;
        }

        // If that fails, extract the base package and try downloading
        let base_package = crate::utils::extract_base_package(submodule_path);
        
        // Check if this is a stdlib module - if so, don't try to download
        if crate::stdlib::is_stdlib_module(&base_package) {
            return None;
        }

        // Try downloading the package and re-attempting exploration
        let mut download_result = None;
        if let Ok(()) = crate::utils::try_download_and_import(py, &base_package, false, || {
            download_result = try_get_signature(py);
            Ok(())
        }) {
            return download_result;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_import_resolver_creation() {
        let resolver = ImportChainResolver::new();
        // Just test that it can be created
        assert!(std::mem::size_of_val(&resolver) >= 0);
    }
}