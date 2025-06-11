use std::fs;
use std::path::{Path, PathBuf};

use pyo3::prelude::*;
use tempfile::TempDir;

/// Downloads and extracts a Python package from PyPI
#[derive(Debug)]
pub struct PackageDownloader {
    package_name: String,
    version_spec: Option<String>,
    temp_dir: Option<TempDir>,
}

impl PackageDownloader {
    pub fn new(package_name: String) -> Self {
        // Parse version spec if present
        let (name, version) = crate::utils::parse_package_spec(&package_name);
        Self {
            package_name: name.to_string(),
            version_spec: version.map(|v| v.to_string()),
            temp_dir: None,
        }
    }

    /// Download and extract the package, returning the path to the extracted package
    pub fn download_and_extract(&mut self) -> PyResult<PathBuf> {
        // Create a temporary directory
        let temp_dir = tempfile::tempdir().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to create temp dir: {}",
                e
            ))
        })?;

        // Query PyPI's simple API
        let package_info = self.fetch_package_info()?;

        // Download the wheel or source distribution
        let downloaded_path = self.download_package(&package_info, temp_dir.path())?;

        // Extract the package
        let extracted_path = self.extract_package(&downloaded_path, temp_dir.path())?;

        // Find the actual package directory
        let package_path = self.find_package_root(&extracted_path)?;

        // Store temp_dir to keep it alive
        self.temp_dir = Some(temp_dir);

        Ok(package_path)
    }

    /// Query PyPI's JSON API for package info
    fn fetch_package_info(&self) -> PyResult<PackageInfo> {
        let clean_name = self.normalize_package_name(&self.package_name);
        let url = format!("https://pypi.org/pypi/{}/json", clean_name);

        let response = reqwest::blocking::get(&url).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to fetch package info: {}",
                e
            ))
        })?;

        if !response.status().is_success() {
            return Err(PyErr::new::<pyo3::exceptions::PyModuleNotFoundError, _>(
                format!("Package '{}' not found on PyPI", self.package_name),
            ));
        }

        let json: serde_json::Value = response.json().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to parse JSON: {}", e))
        })?;

        // Determine which version to download
        let target_version = match &self.version_spec {
            Some(spec) if spec == "latest" => {
                // Use the latest version
                json["info"]["version"].as_str().ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>("Missing version info")
                })?
            }
            Some(spec) => {
                // Check if the specific version exists
                if json["releases"][spec].is_null() {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        format!("Version '{}' not found for package '{}'", spec, self.package_name),
                    ));
                }
                spec
            }
            None => {
                // Default to latest version
                json["info"]["version"].as_str().ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>("Missing version info")
                })?
            }
        };

        // Find a wheel or source distribution for the target version
        let releases = json["releases"][target_version].as_array().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("Missing release info")
        })?;

        // Prefer wheels over source distributions
        let wheel_url = releases
            .iter()
            .find(|r| r["filename"].as_str().unwrap_or("").ends_with(".whl"))
            .and_then(|r| r["url"].as_str());

        let (url, filename) = if let Some(wheel_url) = wheel_url {
            let filename = wheel_url.split('/').last().unwrap_or("package.whl");
            (wheel_url.to_string(), filename.to_string())
        } else {
            // Fall back to source distribution
            let sdist = releases
                .iter()
                .find(|r| {
                    let filename = r["filename"].as_str().unwrap_or("");
                    filename.ends_with(".tar.gz") || filename.ends_with(".zip")
                })
                .ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "No suitable distribution found",
                    )
                })?;

            let url = sdist["url"]
                .as_str()
                .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("Missing URL"))?;
            let filename = sdist["filename"].as_str().ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>("Missing filename")
            })?;

            (url.to_string(), filename.to_string())
        };

        Ok(PackageInfo { url, filename })
    }

    /// Download the package file
    fn download_package(&self, info: &PackageInfo, dest_dir: &Path) -> PyResult<PathBuf> {
        let response = reqwest::blocking::get(&info.url).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to download package: {}",
                e
            ))
        })?;

        if !response.status().is_success() {
            return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to download package: HTTP {}",
                response.status()
            )));
        }

        let content = response.bytes().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to read response: {}", e))
        })?;

        let dest_path = dest_dir.join(&info.filename);
        fs::write(&dest_path, content).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to write file: {}", e))
        })?;

        Ok(dest_path)
    }

    /// Extract the downloaded package
    fn extract_package(&self, archive_path: &Path, dest_dir: &Path) -> PyResult<PathBuf> {
        let extract_dir = dest_dir.join("extracted");
        fs::create_dir_all(&extract_dir).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to create extract dir: {}",
                e
            ))
        })?;

        if archive_path.extension().map(|s| s.to_str()) == Some(Some("whl")) {
            // Extract wheel using zip
            let file = fs::File::open(archive_path).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to open wheel: {}", e))
            })?;

            let mut archive = zip::ZipArchive::new(file).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to read wheel: {}", e))
            })?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                        "Failed to read zip entry: {}",
                        e
                    ))
                })?;

                let path = extract_dir.join(file.name());

                if file.is_dir() {
                    fs::create_dir_all(&path).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                            "Failed to create dir: {}",
                            e
                        ))
                    })?;
                } else {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent).map_err(|e| {
                            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                                "Failed to create parent dir: {}",
                                e
                            ))
                        })?;
                    }

                    let mut outfile = fs::File::create(&path).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                            "Failed to create file: {}",
                            e
                        ))
                    })?;

                    std::io::copy(&mut file, &mut outfile).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                            "Failed to extract file: {}",
                            e
                        ))
                    })?;
                }
            }
        } else if archive_path.to_str().unwrap_or("").ends_with(".tar.gz") {
            // Extract tar.gz
            let file = fs::File::open(archive_path).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "Failed to open archive: {}",
                    e
                ))
            })?;

            let gz = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(gz);

            archive.unpack(&extract_dir).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "Failed to extract archive: {}",
                    e
                ))
            })?;
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
                "Unsupported archive format",
            ));
        }

        Ok(extract_dir)
    }

    /// Find the actual package directory within the extracted files
    fn find_package_root(&self, extract_dir: &Path) -> PyResult<PathBuf> {
        let normalized_name = self.normalize_package_name(&self.package_name);

        // First, check if the package is directly in the extract directory (common for wheels)
        let direct_path = extract_dir.join(&normalized_name);
        if direct_path.exists() && direct_path.is_dir() {
            return Ok(direct_path);
        }

        // For source distributions, look for setup.py or pyproject.toml
        for entry in fs::read_dir(extract_dir).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to read dir: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to read entry: {}", e))
            })?;
            let path = entry.path();

            if path.is_dir() {
                // Check if this looks like a source distribution root
                if path.join("setup.py").exists() || path.join("pyproject.toml").exists() {
                    // Look for the package inside
                    let package_path = path.join(&normalized_name);
                    if package_path.exists() && package_path.is_dir() {
                        return Ok(package_path);
                    }

                    // Sometimes packages are in a src/ directory
                    let src_path = path.join("src").join(&normalized_name);
                    if src_path.exists() && src_path.is_dir() {
                        return Ok(src_path);
                    }
                }
            }
        }

        // If we can't find the expected structure, just return the extract directory
        Ok(extract_dir.to_path_buf())
    }

    /// Normalize package name (replace - with _, lowercase)
    fn normalize_package_name(&self, name: &str) -> String {
        // Extract base name from version specifiers
        let base_name = name
            .split(&['>', '<', '=', '!', '['][..])
            .next()
            .unwrap_or(name)
            .trim();

        base_name.replace('-', "_").to_lowercase()
    }
}

#[derive(Debug)]
struct PackageInfo {
    url: String,
    filename: String,
}

/// Temporary directory path holder
/// Returns the path of a downloaded package for use in Python
#[pyclass]
pub struct DownloadedPackage {
    pub path: PathBuf,
    _temp_dir: TempDir, // Keep the temp directory alive
}

#[pymethods]
impl DownloadedPackage {
    #[getter]
    fn path(&self, py: Python) -> PyResult<PyObject> {
        // Convert PathBuf to Python Path object
        let pathlib = py.import("pathlib")?;
        let path_class = pathlib.getattr("Path")?;
        let path_str = self.path.to_str().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid path encoding")
        })?;
        path_class.call1((path_str,))?.extract()
    }
}

