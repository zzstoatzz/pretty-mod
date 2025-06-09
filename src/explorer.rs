use crate::module_info::ModuleInfo;
use pyo3::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
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
        // ALWAYS use pure file-based discovery (like ty/ruff)
        let module_info = self.explore_module_pure_filesystem(py, &self.root_module_path)?;

        // Create the wrapped format that tests expect: {"api": {...}, "submodules": {...}}
        let tree_dict = pyo3::types::PyDict::new(py);

        // Create the "api" dict with the expected structure
        let api_dict = pyo3::types::PyDict::new(py);
        api_dict.set_item(
            "all",
            module_info.all_exports.as_ref().unwrap_or(&Vec::new()),
        )?;
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
            sub_api_dict.set_item(
                "all",
                submodule_info.all_exports.as_ref().unwrap_or(&Vec::new()),
            )?;
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
                    drop(tree_guard); // Release lock before exploring
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
    /// Get Python's sys.path to guide module discovery
    fn get_sys_path(&self, py: Python) -> PyResult<Vec<PathBuf>> {
        let sys = py.import("sys")?;
        let sys_path: Vec<String> = sys.getattr("path")?.extract()?;
        Ok(sys_path.into_iter().map(PathBuf::from).collect())
    }

    /// Pure filesystem-based module discovery (similar to ty/ruff approach)
    fn explore_module_pure_filesystem(
        &self,
        py: Python,
        module_path: &str,
    ) -> PyResult<ModuleInfo> {
        // Handle dotted module paths (e.g., "urllib.request")
        let parts: Vec<&str> = module_path.split('.').collect();

        // Find the root module's filesystem path
        let (root_path, start_index) = self.find_module_path_filesystem(py, &parts)?;

        // Build the module tree from the found path
        // Use start_index+1 to skip the part that was already resolved
        self.build_module_tree_from_parts(&root_path, &parts[start_index + 1..], module_path, 0)
    }

    /// Build module tree by walking filesystem (like ruff does)
    fn build_module_tree_filesystem(
        &self,
        path: &Path,
        module_path: &str,
        depth: usize,
    ) -> PyResult<ModuleInfo> {
        let mut info = if path.is_file() {
            // Parse the .py file directly
            ModuleInfo::from_python_file(path)?
        } else if path.is_dir() {
            // Check for __init__.py
            let init_py = path.join("__init__.py");
            if init_py.exists() {
                ModuleInfo::from_python_file(&init_py)?
            } else {
                // Namespace package
                ModuleInfo::new()
            }
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Invalid path: {}",
                path.display()
            )));
        };

        // Only explore submodules if we're within depth and path is a directory
        if depth < self.max_depth && path.is_dir() {
            // Collect all Python modules in this directory
            let mut submodules = Vec::new();

            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();

                    // Skip private modules
                    if file_name_str.starts_with('_') && file_name_str != "__init__.py" {
                        continue;
                    }

                    // Check if it's a Python module
                    let submodule_name = if entry_path.is_dir() {
                        // Directory is a package if it has __init__.py
                        if entry_path.join("__init__.py").exists() {
                            Some(file_name_str.to_string())
                        } else {
                            // Could be a namespace package, check if it has .py files
                            if has_python_files(&entry_path) {
                                Some(file_name_str.to_string())
                            } else {
                                None
                            }
                        }
                    } else if file_name_str.ends_with(".py") && file_name_str != "__init__.py" {
                        // Regular .py file
                        Some(file_name_str.trim_end_matches(".py").to_string())
                    } else {
                        None
                    };

                    if let Some(name) = submodule_name {
                        submodules.push((name, entry_path));
                    }
                }
            }

            // Sort for consistent ordering
            submodules.sort_by(|a, b| a.0.cmp(&b.0));

            // Process submodules
            for (submodule_name, submodule_path) in submodules {
                let full_module_path = format!("{}.{}", module_path, submodule_name);

                match self.build_module_tree_filesystem(
                    &submodule_path,
                    &full_module_path,
                    depth + 1,
                ) {
                    Ok(submodule_info) => {
                        info.submodules.insert(submodule_name, submodule_info);
                    }
                    Err(_) => {
                        // Skip modules that fail to parse
                    }
                }
            }
        }

        Ok(info)
    }

    /// Find module path using only filesystem operations (handles dotted paths)
    fn find_module_path_filesystem(
        &self,
        py: Python,
        parts: &[&str],
    ) -> PyResult<(PathBuf, usize)> {
        let sys_paths = self.get_sys_path(py)?;

        for sys_path in sys_paths {
            // Try to resolve as many parts as possible from this sys_path
            let mut current_path = sys_path.clone();

            for (i, part) in parts.iter().enumerate() {
                // Try as a .py file
                let py_file = current_path.join(format!("{}.py", part));
                if py_file.exists() {
                    // Found it! Return the path and where we are in the parts
                    return Ok((py_file, i));
                }

                // Try as a package directory
                let pkg_dir = current_path.join(part);
                if pkg_dir.is_dir() {
                    let init_py = pkg_dir.join("__init__.py");
                    if i == parts.len() - 1 {
                        // Last part - return the directory
                        return Ok((pkg_dir, i));
                    } else if init_py.exists() || has_python_files(&pkg_dir) {
                        // Intermediate package - continue
                        current_path = pkg_dir;
                    } else {
                        // Not a package, stop here
                        break;
                    }
                } else {
                    // Can't continue from here
                    break;
                }
            }
        }

        Err(PyErr::new::<pyo3::exceptions::PyModuleNotFoundError, _>(
            format!("No module named '{}'", parts.join(".")),
        ))
    }

    /// Build module tree from a found path and remaining parts
    fn build_module_tree_from_parts(
        &self,
        path: &Path,
        remaining_parts: &[&str],
        full_module_path: &str,
        depth: usize,
    ) -> PyResult<ModuleInfo> {
        if remaining_parts.is_empty() {
            // We've resolved all parts, build from this path
            self.build_module_tree_filesystem(path, full_module_path, depth)
        } else {
            // We have more parts to resolve within this module
            let mut info = if path.is_file() {
                ModuleInfo::from_python_file(path)?
            } else {
                let init_py = path.join("__init__.py");
                if init_py.exists() {
                    ModuleInfo::from_python_file(&init_py)?
                } else {
                    ModuleInfo::new()
                }
            };

            // Continue resolving the remaining parts
            let next_part = remaining_parts[0];
            let next_path = path.join(next_part);

            if next_path.exists() || next_path.with_extension("py").exists() {
                let sub_info = self.build_module_tree_from_parts(
                    &next_path,
                    &remaining_parts[1..],
                    full_module_path,
                    depth + 1,
                )?;
                info.submodules.insert(next_part.to_string(), sub_info);
            }

            Ok(info)
        }
    }
}

/// Check if a directory contains any Python files
fn has_python_files(path: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
                    if ext == "py" {
                        return true;
                    }
                }
            } else if entry_path.is_dir() {
                // Check subdirectories recursively
                if has_python_files(&entry_path) {
                    return true;
                }
            }
        }
    }
    false
}
