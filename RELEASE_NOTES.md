# Release Notes - v0.1.1

## 🎯 Auto-Download Support

This release adds the ability to explore packages without having them installed! `pretty-mod` will automatically download and extract packages from PyPI when needed.

### ✨ New Features

- **Automatic package downloads**: Run `pretty-mod tree django` without having Django installed
- **`--quiet` flag**: Suppress download messages for cleaner output (especially useful for LLM consumption)

### 🏗️ Technical Improvements

- **Refactored `lib.rs`**: Split into focused modules (`signature.rs`, `tree_formatter.rs`, `package_downloader.rs`)
- **Fixed memory safety**: Added RAII guard to ensure `sys.path` is always cleaned up, even on errors
- **Cross-platform builds**: Fixed ARM64 Linux builds by using `manylinux_2_28` and `rustls` for TLS

### 📦 Installation

Download overhead is minimal (~200ms) and only applies to packages not already installed.

---

# Release Notes - v0.1.0

## 🎉 Major Performance Release

This release introduces a complete architectural rewrite that eliminates Python's import overhead, resulting in massive performance improvements, especially for large packages.

### 📊 Performance Improvements

Benchmarked against v0.0.4 (pure python) using `uvx`:

| Package | v0.0.4 (pure python) | v0.1.0 | Speedup |
|---------|-----------------|---------|---------|
| **prefect** | 1,319ms | 104ms | **12.7x faster** |
| **numpy** | 130ms | 59ms | **2.2x faster** |
| **pandas** | 218ms | 76ms | **2.9x faster** |

### 🔧 Technical Changes

- **Pure Rust implementation** using ruff's AST parser components
- **Zero Python imports** during module discovery
- **Direct filesystem traversal** with BFS algorithm
- **Rust extension** compiled with maturin for optimal performance

### 🐛 Bug Fixes

- Fixed critical comma-splitting bug in type annotations
- Improved parameter formatting (`*args`, `**kwargs`)
- Fixed submodule discovery in filesystem walker
- Better handling of namespace packages

### 📦 Installation

Wheels are available for all major platforms via PyPI thanks to [maturin](https://github.com/PyO3/maturin).

To install the latest version, use the `--refresh-package` flag with `uvx` at least once.

```bash
uvx --refresh-package pretty-mod pretty-mod tree json # bust cache

uvx pretty-mod tree json # use the latest version going forward
```





---

# Release Notes - v0.1.0-alpha.1

## 🚀 Performance Breakthrough

### Pure Filesystem-Based Discovery
- **Complete rewrite using ruff's low-level components** - Eliminated Python's import system from module discovery
- **Massive performance gains**:
  - json: 0.49ms (previously 0.45ms with imports)
  - urllib: 1.79ms (previously 1.88ms with imports)  
  - prefect: 21.71ms (previously ~1140ms with imports) - **52x faster!**
- **Uses ruff_python_parser for AST parsing** - Direct parsing of Python files without imports
- **BFS directory traversal** - Efficient filesystem walking similar to ruff/ty tools
- **Zero import overhead** - Module discovery now completely avoids Python's import machinery

## 🐛 Bug Fixes

### Signature Display Fixes
- **Fixed critical comma-splitting bug** in type annotations - `Dict[str, Any]` no longer gets split across multiple lines
- **Improved parameter formatting** to match Python syntax more closely:
  - `*args` and `**kwargs` now display with proper asterisk prefixes
  - Keyword-only parameters separated with `*` line instead of verbose `(keyword-only)` text
  - Default values formatted as `param=value` instead of `param = value`
- **Better type annotation filtering** - now shows `typing.Dict[str, Any]` instead of hiding all typing annotations
- **Preserved return type information** that was present in pre-0.1 versions

### Module Discovery Fixes
- **Fixed submodule discovery bug** in filesystem walker - Correctly handles module path resolution
- **Improved handling of namespace packages** - Works with packages without `__init__.py`

### Examples
**Before (broken):**
```
├── values: Dict[str
├── Any]
```

**After (fixed):**
```
├── values: typing.Dict[str, typing.Any]
```

**Parameter display improvements:**
```
# Old format:
├── skipkeys = False (keyword-only)
└── kw (**kwargs)

# New format:  
├── *
├── skipkeys=False
└── **kw
```

## ✅ Verification
- All README examples tested and working correctly
- Signature parsing now properly handles complex type annotations
- Tree exploration output matches stable version (with minor ordering differences)
- Performance gains verified on standard library and third-party packages
- No unused dependencies in Cargo.toml

---

# Release Notes - v0.1.0-alpha

## Overview
This alpha release introduces significant performance improvements to pretty-mod, particularly for exploring installed Python packages.

## Key Changes

### 🚀 Performance Improvements
- **New import-based discovery**: Now tries to import modules first before falling back to filesystem scanning
- **Optimized submodule discovery**: Uses Python's `pkgutil` for faster discovery of installed packages
- **Average speedup**: 160x faster than the published version on standard library modules
- **Consistent performance**: ~0.1ms per module regardless of package size

### 📊 Benchmarking
- Consolidated performance testing into a single comprehensive script (`scripts/perf_test.py`)
- Added bottleneck analysis mode to compare against Python's native AST parsing
- Added support for testing large third-party packages

### 🐛 Fixes
- Fixed early termination bug when `max_depth` was reached
- Improved error handling for modules that can't be found

## Performance Results

| Module Type | Average Speedup | Notes |
|------------|-----------------|-------|
| Small stdlib (json, itertools) | 400-900x | Minimal import overhead |
| Medium stdlib (urllib, email) | 3-10x | Some import overhead |
| Large packages (numpy, pandas) | Variable | Import overhead dominates |

## Known Issues
- Performance on packages with heavy import-time initialization (like Prefect) is limited by Python's import system, not our parser
- Some deprecation warnings may appear when exploring certain packages (e.g., numpy)

## Testing
Run performance tests with:
```bash
just perf-test
# or for specific analysis:
./scripts/perf_test.py --bottleneck MODULE
./scripts/perf_test.py --stress
```

## What's Next
- Further optimization of the parsing pipeline
- Better handling of namespace packages
- Performance profiling of the Rust parser itself

---

# Release Notes - v0.0.4

## Overview
Pure Python implementation focused on tree exploration and visualization.

## Key Features
- Enhanced tree display with better formatting
- Support for `__all__` exports
- Improved handling of module paths

**Full Changelog**: https://github.com/zzstoatzz/pretty-mod/compare/0.0.3...0.0.4

---

# Release Notes - v0.0.3

## Overview
Bug fixes and improved module discovery.

## Changes
- Fixed module path resolution for nested packages
- Better error messages for missing modules
- Improved CLI output formatting

**Full Changelog**: https://github.com/zzstoatzz/pretty-mod/compare/0.0.2...0.0.3

---

# Release Notes - v0.0.2

## Overview
Added signature inspection capabilities.

## New Features
- `pretty-mod sig module:function` command to inspect function signatures
- Parameter details including defaults and type annotations
- Support for keyword-only arguments

**Full Changelog**: https://github.com/zzstoatzz/pretty-mod/compare/0.0.1...0.0.2

---

# Release Notes - v0.0.1

## Overview
Initial release of pretty-mod.

## Features
- `pretty-mod tree` command for module exploration
- Recursive traversal with `max_depth` control
- Unicode tree visualization
- Basic function and class discovery

---

# Release Notes - v0.0.1-alpha.0

## Overview
Hello world! First pre-release of pretty-mod.

## Features
- Basic module tree exploration
- CLI interface
- Pure Python implementation