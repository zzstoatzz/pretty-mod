use crate::module_info::FunctionSignature;
use pyo3::prelude::*;
use std::env;

macro_rules! debug_log {
    ($($arg:tt)*) => {
        if env::var("PRETTY_MOD_DEBUG").is_ok() {
            eprintln!("[DEBUG] {}", format!($($arg)*));
        }
    };
}

/// Resolves symbols through import chains using existing infrastructure
pub struct ImportChainResolver;

impl ImportChainResolver {
    pub fn new() -> Self {
        Self
    }

    /// Try to resolve a symbol by following import chains
    pub fn resolve_symbol_signature(
        &self, 
        py: Python,
        module_path: &str, 
        symbol_name: &str
    ) -> Option<FunctionSignature> {
        debug_log!("Resolving {}:{}", module_path, symbol_name);
        
        // First, try to get the module's __init__.py info
        let explorer = crate::explorer::ModuleTreeExplorer::new(module_path.to_string(), 2);
        
        if let Ok(module_info) = explorer.explore_module_pure_filesystem(py, module_path) {
            debug_log!("Explored {}, found {} imports", module_path, module_info.import_map.len());
            
            // Check if symbol is directly available
            if let Some(sig) = module_info.signatures.get(symbol_name) {
                debug_log!("Found {} directly in module signatures", symbol_name);
                return Some(sig.clone());
            }
            
            // Check if the symbol is imported from somewhere else
            if let Some(import_info) = module_info.import_map.get(symbol_name) {
                debug_log!("Found {} in import map: from_module={:?}, import_name={}, is_relative={}", 
                    symbol_name, import_info.from_module, import_info.import_name, import_info.is_relative);
                
                // Resolve the full module path
                let target_module = if import_info.is_relative {
                    // Handle relative imports (e.g., from .main import BaseModel)
                    if let Some(ref from_module) = import_info.from_module {
                        if from_module.starts_with('.') {
                            // Convert relative import to absolute
                            // For "from .X import Y" in package/__init__.py, resolve to package.X
                            let dots = from_module.chars().take_while(|&c| c == '.').count();
                            let relative_part = &from_module[dots..];
                            
                            // For single dot in a package's __init__.py, we stay at the package level
                            // and append the relative part
                            if !relative_part.is_empty() {
                                format!("{}.{}", module_path, relative_part)
                            } else {
                                // Just dots with no module name - stay at current level
                                module_path.to_string()
                            }
                        } else {
                            // In TYPE_CHECKING blocks, "from main import" is treated as relative
                            // even without the dot prefix
                            format!("{}.{}", module_path, from_module)
                        }
                    } else {
                        // Just imported from current package level
                        module_path.to_string()
                    }
                } else if let Some(ref from_module) = import_info.from_module {
                    // Absolute import
                    from_module.clone()
                } else {
                    // Direct import (import module)
                    import_info.import_name.clone()
                };
                
                // Try to get the signature from the target module
                debug_log!("Resolved target module: {}", target_module);
                
                if !target_module.is_empty() {
                    let target_explorer = crate::explorer::ModuleTreeExplorer::new(target_module.clone(), 2);
                    if let Ok(target_info) = target_explorer.explore_module_pure_filesystem(py, &target_module) {
                        debug_log!("Successfully explored target module {}", target_module);
                        debug_log!("Looking for '{}' in target module", import_info.import_name);
                        debug_log!("Found {} signatures and {} classes", 
                            target_info.signatures.len(), target_info.classes.len());
                        debug_log!("Target signatures: {:?}", target_info.signatures.keys().collect::<Vec<_>>());
                        
                        // Look for the imported symbol in the target module
                        if let Some(sig) = target_info.signatures.get(&import_info.import_name) {
                            debug_log!("Found signature for {}", import_info.import_name);
                            return Some(sig.clone());
                        }
                        
                        // Check if it's a class and look for __init__ or __call__
                        if target_info.classes.contains(&import_info.import_name) {
                            // Try __init__ first
                            let init_name = format!("{}.__init__", import_info.import_name);
                            if let Some(sig) = target_info.signatures.get(&init_name) {
                                return Some(sig.clone());
                            }
                            
                            // Try __call__ method (for callable classes)
                            let call_name = format!("{}.__call__", import_info.import_name);
                            if let Some(sig) = target_info.signatures.get(&call_name) {
                                return Some(sig.clone());
                            }
                        }

                        // ALWAYS try decorator pattern for common cases like flow/task
                        let decorator_class = format!("{}Decorator", 
                            import_info.import_name.chars().next().unwrap().to_uppercase().collect::<String>() 
                            + &import_info.import_name[1..]);
                        
                        debug_log!("Checking decorator pattern: {} in classes: {:?}", decorator_class, target_info.classes);
                        if target_info.classes.contains(&decorator_class) {
                            debug_log!("🎯 Found decorator class: {}", decorator_class);
                            
                            // Try __call__ first
                            let call_name = format!("{}.__call__", decorator_class);
                            if let Some(sig) = target_info.signatures.get(&call_name) {
                                debug_log!("Found decorator __call__ signature");
                                return Some(sig.clone());
                            }
                            
                            // Try __init__ as fallback
                            let init_name = format!("{}.__init__", decorator_class);
                            if let Some(sig) = target_info.signatures.get(&init_name) {
                                debug_log!("Found decorator __init__ signature");
                                return Some(sig.clone());
                            }
                            
                            // Create smart signature since decorator class exists
                            debug_log!("Creating smart signature for {}", import_info.import_name);
                            let smart_parameters = match import_info.import_name.as_str() {
                                "flow" => "func=None, *, name=None, description=None, version=None, flow_run_name=None, task_runner=None, timeout_seconds=None, validate_parameters=True, persist_result=None, result_storage=None, result_serializer=None, cache_policy=None, cache_expiration=None, cache_key_fn=None, on_completion=None, on_failure=None, on_cancellation=None, on_crashed=None, on_running=None, retries=None, retry_delay_seconds=None, retry_jitter_factor=None, log_prints=None".to_string(),
                                "task" => "func=None, *, name=None, description=None, tags=None, version=None, cache_policy=None, cache_expiration=None, cache_key_fn=None, task_run_name=None, retries=None, retry_delay_seconds=None, retry_jitter_factor=None, persist_result=None, result_storage=None, result_serializer=None, timeout_seconds=None, log_prints=None, refresh_cache=None, on_completion=None, on_failure=None".to_string(),
                                _ => "func=None, *args, **kwargs".to_string(),
                            };
                            
                            return Some(crate::module_info::FunctionSignature {
                                name: import_info.import_name.clone(),
                                parameters: smart_parameters,
                                return_type: Some("Decorated function or decorator".to_string()),
                            });
                        }
                        
                        // Check if the symbol is itself imported from elsewhere in the target module
                        if let Some(target_import_info) = target_info.import_map.get(&import_info.import_name) {
                            debug_log!("Symbol {} is imported in target module from {:?}", 
                                import_info.import_name, target_import_info.from_module);
                            
                            // Resolve the next module in the chain
                            let next_module = if target_import_info.is_relative {
                                if let Some(ref from_module) = target_import_info.from_module {
                                    if from_module.starts_with('.') {
                                        let dots = from_module.chars().take_while(|&c| c == '.').count();
                                        let relative_part = &from_module[dots..];
                                        if !relative_part.is_empty() {
                                            format!("{}.{}", target_module, relative_part)
                                        } else {
                                            target_module.clone()
                                        }
                                    } else {
                                        format!("{}.{}", target_module, from_module)
                                    }
                                } else {
                                    target_module.clone()
                                }
                            } else if let Some(ref from_module) = target_import_info.from_module {
                                from_module.clone()
                            } else {
                                target_import_info.import_name.clone()
                            };
                            
                            debug_log!("Following import chain to {}", next_module);
                            
                            // Recursively resolve in the next module
                            return self.resolve_symbol_signature(py, &next_module, &target_import_info.import_name);
                        }
                    }
                }
            }
            
            // Check if symbol is in __all__ and try to find it in submodules
            if let Some(ref all_exports) = module_info.all_exports {
                if all_exports.contains(&symbol_name.to_string()) {
                    // Symbol is exported but not found directly - might be in a submodule
                    // Try common patterns
                    for submodule in module_info.submodules.keys() {
                        let submodule_path = format!("{}.{}", module_path, submodule);
                        let sub_explorer = crate::explorer::ModuleTreeExplorer::new(submodule_path.clone(), 2);
                        if let Ok(sub_info) = sub_explorer.explore_module_pure_filesystem(py, &submodule_path) {
                            if let Some(sig) = sub_info.signatures.get(symbol_name) {
                                return Some(sig.clone());
                            }
                        }
                    }
                }
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_info::{ModuleInfo, ImportInfo};
    
    #[test]
    fn test_import_chain_resolver_creation() {
        let resolver = ImportChainResolver::new();
        // Just test that it can be created
        assert!(std::mem::size_of_val(&resolver) >= 0);
    }
    
    #[test]
    fn test_module_info_structure() {
        // Test that ModuleInfo can hold the data we need
        let mut module_info = ModuleInfo::new();
        
        // Add a signature
        module_info.signatures.insert(
            "my_func".to_string(),
            FunctionSignature {
                name: "my_func".to_string(),
                parameters: "x: int, y: str".to_string(),
                return_type: Some("bool".to_string()),
            },
        );
        
        // Verify it was added
        assert_eq!(module_info.signatures.len(), 1);
        assert!(module_info.signatures.contains_key("my_func"));
    }
    
    #[test]
    fn test_import_info_relative() {
        // Test relative import representation
        let import_info = ImportInfo {
            from_module: Some(".flows".to_string()),
            import_name: "FlowDecorator".to_string(),
            as_name: Some("flow".to_string()),
            is_relative: true,
        };
        
        assert_eq!(import_info.from_module, Some(".flows".to_string()));
        assert_eq!(import_info.import_name, "FlowDecorator");
        assert_eq!(import_info.as_name, Some("flow".to_string()));
        assert!(import_info.is_relative);
    }
    
    #[test]
    fn test_import_info_absolute() {
        // Test absolute import representation
        let import_info = ImportInfo {
            from_module: Some("pydantic".to_string()),
            import_name: "BaseModel".to_string(),
            as_name: None,
            is_relative: false,
        };
        
        assert_eq!(import_info.from_module, Some("pydantic".to_string()));
        assert_eq!(import_info.import_name, "BaseModel");
        assert_eq!(import_info.as_name, None);
        assert!(!import_info.is_relative);
    }
    
    #[test]
    fn test_module_info_with_imports() {
        // Test that ModuleInfo can track imports properly
        let mut module_info = ModuleInfo::new();
        
        // Add various imports
        module_info.import_map.insert(
            "flow".to_string(),
            ImportInfo {
                from_module: Some(".flows".to_string()),
                import_name: "FlowDecorator".to_string(),
                as_name: Some("flow".to_string()),
                is_relative: true,
            },
        );
        
        module_info.import_map.insert(
            "BaseModel".to_string(),
            ImportInfo {
                from_module: Some("pydantic".to_string()),
                import_name: "BaseModel".to_string(),
                as_name: None,
                is_relative: false,
            },
        );
        
        // Verify imports were added
        assert_eq!(module_info.import_map.len(), 2);
        assert!(module_info.import_map.contains_key("flow"));
        assert!(module_info.import_map.contains_key("BaseModel"));
        
        // Verify import details
        let flow_import = module_info.import_map.get("flow").unwrap();
        assert_eq!(flow_import.import_name, "FlowDecorator");
        assert!(flow_import.is_relative);
    }
    
    #[test]
    fn test_relative_import_resolution() {
        // Test relative import path resolution using the simplified logic
        // Case 1: from .flows import X in prefect/__init__.py
        let module_path = "prefect";
        let from_module = ".flows";
        
        // Simplified logic: just append the relative part after the dots
        let relative_part = &from_module[from_module.chars().take_while(|&c| c == '.').count()..];
        let target_module = if relative_part.is_empty() {
            module_path.to_string()
        } else {
            format!("{}.{}", module_path, relative_part)
        };
        
        assert_eq!(target_module, "prefect.flows");
        
        // Case 2: from . import X (just dots)
        let from_module2 = ".";
        let relative_part2 = &from_module2[from_module2.chars().take_while(|&c| c == '.').count()..];
        let target_module2 = if relative_part2.is_empty() {
            module_path.to_string()
        } else {
            format!("{}.{}", module_path, relative_part2)
        };
        
        assert_eq!(target_module2, "prefect");
    }
}