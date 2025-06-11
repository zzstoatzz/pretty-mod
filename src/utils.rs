use pyo3::prelude::*;

/// RAII guard for sys.path cleanup
struct PathGuard<'py> {
    sys_path: &'py pyo3::Bound<'py, pyo3::PyAny>,
    path: &'py str,
}

impl Drop for PathGuard<'_> {
    fn drop(&mut self) {
        // Best effort removal - don't panic in drop
        let _ = self.sys_path.call_method1("remove", (self.path,));
    }
}

/// Parse a package specification into name and version
/// e.g., "package@1.2.3" -> ("package", Some("1.2.3"))
/// e.g., "package" -> ("package", None)
pub fn parse_package_spec(spec: &str) -> (&str, Option<&str>) {
    if let Some((name, version)) = spec.split_once('@') {
        if version.is_empty() {
            (spec, None)
        } else {
            (name, Some(version))
        }
    } else {
        (spec, None)
    }
}

/// Extract the base package name from a module path
/// e.g., "prefect.server.api" -> "prefect"
pub fn extract_base_package(module_path: &str) -> &str {
    // First parse any @ version specifier
    let (module_path, _version) = parse_package_spec(module_path);

    // Then remove any PEP 508 version specifiers
    let module_name = module_path
        .split(&['[', '>', '<', '=', '!'][..])
        .next()
        .unwrap_or(module_path)
        .trim();

    // Then get the first component
    module_name.split('.').next().unwrap_or(module_name)
}

/// Try to download and temporarily add a package to sys.path
pub fn try_download_and_import<F, R>(
    py: Python,
    package_name: &str,
    quiet: bool,
    f: F,
) -> PyResult<R>
where
    F: FnOnce() -> PyResult<R>,
{
    // Show download message if not quiet
    if !quiet {
        let sys = py.import("sys")?;
        let stderr = sys.getattr("stderr")?;
        stderr.call_method1(
            "write",
            (format!(
                "Module '{}' not found locally. Attempting to download from PyPI...\n",
                package_name
            ),),
        )?;
        stderr.call_method0("flush")?;
    }

    // Parse package name (without version) for path operations
    let (base_name, _) = parse_package_spec(package_name);

    // Download and extract the package (with version if specified)
    let mut downloader =
        crate::package_downloader::PackageDownloader::new(package_name.to_string());
    let package_path = downloader.download_and_extract()?;

    // Add to sys.path temporarily with RAII cleanup
    let sys = py.import("sys")?;
    let sys_path = sys.getattr("path")?;

    // Determine the right directory to add to sys.path
    let parent_dir = if package_path.ends_with(base_name)
        || package_path.ends_with(base_name.replace('-', "_"))
    {
        package_path.parent().unwrap()
    } else {
        &package_path
    };

    let parent_dir_str = parent_dir.to_str().unwrap();
    sys_path.call_method1("insert", (0, parent_dir_str))?;

    // Create guard for cleanup
    let _guard = PathGuard {
        sys_path: &sys_path,
        path: parent_dir_str,
    };

    // Execute the provided function
    f()
}

/// Import an object from a module path (internal implementation)
pub fn import_object_impl(py: Python, import_path: &str) -> PyResult<PyObject> {
    // Support both colon and dot syntax
    if import_path.contains(':') {
        // Colon syntax: module:object
        let parts: Vec<&str> = import_path.split(':').collect();
        if parts.len() != 2 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Import path must be in format 'module:object' or 'module.object'",
            ));
        }
        let (module_spec, object_name) = (parts[0], parts[1]);
        // Parse version spec from module name
        let (module_name, _version) = parse_package_spec(module_spec);
        let module = py.import(module_name)?;
        module.getattr(object_name)?.extract()
    } else if import_path.contains('.') {
        // Parse version spec first
        let (path_without_version, _version) = parse_package_spec(import_path);

        // Dot syntax: try to find where module ends and attribute begins
        let parts: Vec<&str> = path_without_version.split('.').collect();

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
        let (module_name, _version) = parse_package_spec(path_without_version);
        py.import(module_name).map(|m| m.into())
    } else {
        // No dots or colons, assume it's a module name
        let (module_name, _version) = parse_package_spec(import_path);
        py.import(module_name).map(|m| m.into())
    }
}

/// Import an object from a module path with auto-download support
pub fn import_object_with_download(
    py: Python,
    import_path: &str,
    quiet: bool,
) -> PyResult<PyObject> {
    // First try normal import
    match import_object_impl(py, import_path) {
        Ok(obj) => Ok(obj),
        Err(e) => {
            // Check if it's a module not found error
            let err_str = e.to_string();
            if err_str.contains("No module named") || err_str.contains("ModuleNotFoundError") {
                // Extract the base module name
                let base_module = if import_path.contains(':') {
                    extract_base_package(import_path.split(':').next().unwrap())
                } else {
                    extract_base_package(import_path)
                };

                // Try downloading and importing
                try_download_and_import(py, base_module, quiet, || {
                    import_object_impl(py, import_path)
                })
            } else {
                Err(e)
            }
        }
    }
}
