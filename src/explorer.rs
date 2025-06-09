use crate::module_info::ModuleInfo;
use pyo3::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

/// A Rust implementation of module tree exploration
#[pyclass]
pub struct ModuleTreeExplorer {
    root_module_path: String,
    max_depth: usize,
    tree: Mutex<Option<PyObject>>,
}

#[pymethods]
impl ModuleTreeExplorer {
    #[new]
    #[pyo3(signature = (root_module_path, max_depth = 2))]
    pub fn new(root_module_path: String, max_depth: usize) -> Self {
        Self {
            root_module_path,
            max_depth,
            tree: Mutex::new(None),
        }
    }

    #[getter]
    pub fn root_module_path(&self) -> &str {
        &self.root_module_path
    }

    #[getter]
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    #[getter]
    pub fn tree(&self, py: Python) -> PyResult<PyObject> {
        let tree_guard = self.tree.lock().unwrap();
        match &*tree_guard {
            Some(tree) => Ok(tree.clone_ref(py)),
            None => {
                // Return empty dict if not explored yet
                let dict = pyo3::types::PyDict::new(py);
                Ok(dict.into_pyobject(py)?.into_any().unbind())
            }
        }
    }

    pub fn explore(&self, py: Python) -> PyResult<PyObject> {
        // Build tree using Python's sys.path for module discovery but Rust for parsing
        let module_info = self.explore_module_rust(py, &self.root_module_path, 0)?;
        
        // Create the wrapped format that tests expect: {"api": {...}, "submodules": {...}}
        let tree_dict = pyo3::types::PyDict::new(py);
        
        // Create the "api" dict with the expected structure
        let api_dict = pyo3::types::PyDict::new(py);
        api_dict.set_item("all", module_info.all_exports.as_ref().unwrap_or(&Vec::new()))?;
        api_dict.set_item("functions", &module_info.functions)?;
        api_dict.set_item("classes", &module_info.classes)?;
        api_dict.set_item("constants", &module_info.constants)?;
        tree_dict.set_item("api", api_dict)?;
        
        // Convert submodules to the expected format
        let submodules_dict = pyo3::types::PyDict::new(py);
        for (name, submodule_info) in module_info.submodules {
            let submodule_dict = pyo3::types::PyDict::new(py);
            
            // Create api dict for submodule
            let sub_api_dict = pyo3::types::PyDict::new(py);
            sub_api_dict.set_item("all", submodule_info.all_exports.as_ref().unwrap_or(&Vec::new()))?;
            sub_api_dict.set_item("functions", &submodule_info.functions)?;
            sub_api_dict.set_item("classes", &submodule_info.classes)?;
            sub_api_dict.set_item("constants", &submodule_info.constants)?;
            submodule_dict.set_item("api", sub_api_dict)?;
            
            // Convert nested submodules recursively
            let nested_submodules_dict = pyo3::types::PyDict::new(py);
            for (nested_name, nested_info) in submodule_info.submodules {
                let nested_dict = convert_module_info_to_dict(py, &nested_info)?;
                nested_submodules_dict.set_item(nested_name, nested_dict)?;
            }
            submodule_dict.set_item("submodules", nested_submodules_dict)?;
            
            submodules_dict.set_item(name, submodule_dict)?;
        }
        tree_dict.set_item("submodules", submodules_dict)?;
        
        let py_tree: PyObject = tree_dict.into();
        
        // Store in the tree attribute
        let mut tree_guard = self.tree.lock().unwrap();
        *tree_guard = Some(py_tree.clone_ref(py));
        
        Ok(py_tree)
    }
    
    pub fn get_tree_string(&self, py: Python) -> PyResult<String> {
        // Get the tree, exploring if necessary
        let tree_obj = {
            let tree_guard = self.tree.lock().unwrap();
            match &*tree_guard {
                Some(tree) => tree.clone_ref(py),
                None => {
                    drop(tree_guard);  // Release lock before exploring
                    self.explore(py)?
                }
            }
        };
        
        // Use the display_tree formatting logic, which expects the wrapped format
        crate::format_tree_display(py, &tree_obj, &self.root_module_path)
    }
}

/// Convert a ModuleInfo struct to a Python dict
fn convert_module_info_to_dict(py: Python, info: &ModuleInfo) -> PyResult<PyObject> {
    let dict = pyo3::types::PyDict::new(py);
    
    // Create api dict
    let api_dict = pyo3::types::PyDict::new(py);
    api_dict.set_item("all", info.all_exports.as_ref().unwrap_or(&Vec::new()))?;
    api_dict.set_item("functions", &info.functions)?;
    api_dict.set_item("classes", &info.classes)?;
    api_dict.set_item("constants", &info.constants)?;
    dict.set_item("api", api_dict)?;
    
    // Convert submodules recursively
    let submodules_dict = pyo3::types::PyDict::new(py);
    for (name, sub_info) in &info.submodules {
        let sub_dict = convert_module_info_to_dict(py, sub_info)?;
        submodules_dict.set_item(name, sub_dict)?;
    }
    dict.set_item("submodules", submodules_dict)?;
    
    Ok(dict.into())
}

impl ModuleTreeExplorer {
    /// Recursively explore a module and build ModuleInfo tree using Python's sys.path
    fn explore_module_rust(
        &self,
        py: Python,
        module_path: &str,
        current_depth: usize,
    ) -> PyResult<ModuleInfo> {
        if current_depth >= self.max_depth {
            return Ok(ModuleInfo::new());
        }

        // Try filesystem-based approach first
        match self.find_module_path(py, module_path) {
            Ok(module_file_path) => {
                self.explore_filesystem_module(py, module_path, module_file_path, current_depth)
            }
            Err(_) => {
                // Fallback: try built-in module via Python introspection
                self.explore_builtin_module(py, module_path)
            }
        }
    }

    /// Explore a module found on the filesystem
    fn explore_filesystem_module(
        &self,
        py: Python,
        module_path: &str,
        module_file_path: PathBuf,
        current_depth: usize,
    ) -> PyResult<ModuleInfo> {
        let mut info = if module_file_path.is_dir() {
            // Package: read __init__.py
            let init_py = module_file_path.join("__init__.py");
            if init_py.exists() {
                ModuleInfo::from_python_file(&init_py)?
            } else {
                ModuleInfo::new()
            }
        } else {
            // Regular module: read the .py file
            ModuleInfo::from_python_file(&module_file_path)?
        };

        // Find submodules by scanning the directory
        if module_file_path.is_dir() {
            if let Ok(entries) = fs::read_dir(&module_file_path) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();

                    // Check for Python modules/packages
                    let submodule_name = if entry.path().is_dir() {
                        // Directory with __init__.py is a package
                        let init_py = entry.path().join("__init__.py");
                        if init_py.exists() {
                            Some(file_name_str.to_string())
                        } else {
                            None
                        }
                    } else if file_name_str.ends_with(".py") && file_name_str != "__init__.py" {
                        // .py file is a module
                        Some(file_name_str.trim_end_matches(".py").to_string())
                    } else {
                        None
                    };

                    if let Some(submodule_name) = submodule_name {
                        let full_submodule_path = format!("{}.{}", module_path, submodule_name);

                        match self.explore_module_rust(py, &full_submodule_path, current_depth + 1)
                        {
                            Ok(submodule_info) => {
                                info.submodules.insert(submodule_name, submodule_info);
                            }
                            Err(_) => {
                                // Skip failed modules
                            }
                        }
                    }
                }
            }
        }

        Ok(info)
    }

    /// Explore a built-in module using Python introspection
    fn explore_builtin_module(&self, py: Python, module_path: &str) -> PyResult<ModuleInfo> {
        // Try to import the module via Python
        let module = match py.import(module_path) {
            Ok(module) => module,
            Err(_) => {
                return Err(PyErr::new::<pyo3::exceptions::PyModuleNotFoundError, _>(
                    format!("No module named '{}'", module_path),
                ));
            }
        };

        // Use Python introspection to get module info
        ModuleInfo::from_python_module(py, &module)
    }

    /// Get Python's sys.path to guide module discovery
    fn get_sys_path(&self, py: Python) -> PyResult<Vec<PathBuf>> {
        let sys = py.import("sys")?;
        let sys_path: Vec<String> = sys.getattr("path")?.extract()?;
        Ok(sys_path.into_iter().map(PathBuf::from).collect())
    }

    /// Find the filesystem path for a Python module using Python's sys.path
    fn find_module_path(&self, py: Python, module_path: &str) -> PyResult<PathBuf> {
        let parts: Vec<&str> = module_path.split('.').collect();
        let sys_paths = self.get_sys_path(py)?;

        // Try to find the module in sys.path
        for sys_path in sys_paths {
            let mut current_path = sys_path;

            for (i, part) in parts.iter().enumerate() {
                if i == parts.len() - 1 {
                    // Last part: could be module.py or package directory
                    let py_file = current_path.join(format!("{}.py", part));
                    let pkg_dir = current_path.join(part);

                    if py_file.exists() {
                        return Ok(py_file);
                    } else if pkg_dir.is_dir() && pkg_dir.join("__init__.py").exists() {
                        return Ok(pkg_dir);
                    }
                } else {
                    // Intermediate parts: must be packages
                    current_path = current_path.join(part);
                    if !current_path.is_dir() || !current_path.join("__init__.py").exists() {
                        break; // This path doesn't work
                    }
                }
            }
        }

        Err(PyErr::new::<pyo3::exceptions::PyModuleNotFoundError, _>(
            format!("No module named '{}'", module_path),
        ))
    }
}
