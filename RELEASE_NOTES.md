# Release Notes - v0.2.1

## 📊 JSON Output Support & Better Type Annotation Handling

This release adds machine-readable JSON output and fixes a critical bug with complex type annotations.

### ✨ New Features

- **📊 JSON Output Support**: Export tree and signature data as JSON for programmatic use
  - `pretty-mod tree json -o json` - Get module structure as JSON
  - `pretty-mod sig json:dumps -o json` - Get function signature as JSON
  - Perfect for piping to `jq` or other JSON processors
  - Follows the Kubernetes pattern of `-o <format>` for output selection
  - Example: `pretty-mod tree json -o json | jq '.tree.submodules | keys'`

### 🏗️ Technical Improvements

- **Visitor Pattern**: Implemented output formatters using the Visitor pattern for extensibility
  - Clean separation between data structure and formatting
  - Easy to add new output formats in the future
  - Type-safe implementation using Rust traits


---

# Release Notes - v0.2.0

## 🎨 Customizable Display & Colors + Enhanced Signature Support

This release introduces customizable display characters, color output, full type annotation support in signatures, and a new double-colon syntax for handling packages where the PyPI name differs from the module name.

### 🚨 Breaking Changes
- **Color output by default**: Tree and signature displays now include ANSI color codes
- **Minor version bump**: Due to visual output changes, this is a minor version release

### ✨ New Features

- **🔗 Double-colon syntax**: Handle packages where PyPI name differs from module name
  - `pretty-mod tree pydocket::docket` - Download 'pydocket' package, explore 'docket' module
  - `pretty-mod tree pillow::PIL` - Download 'pillow' package, explore 'PIL' module
  - Works with version specifiers: `pretty-mod tree pillow::PIL@10.0.0`
  - Works with signatures: `pretty-mod sig pillow::PIL.Image:open`

- **📝 Full Type Annotation Support**: Signatures now display complete type information
  - Union types: `str | None`
  - Generic types: `list[Tool | Callable[..., Any]]`
  - Literal types: `Literal['protocol', 'path']`
  - Complex nested types properly rendered from AST

- **🎨 Color Support**: Earth-tone/pastel color scheme
  - Modules: Saddle brown (#8B7355)
  - Functions: Olive drab (#6B8E23)
  - Classes: Steel blue (#4682B4)
  - Constants: Rosy brown (#BC8F8F)
  - Warning messages: Goldenrod (#DAA520)
  - And more subtle colors for parameters, types, and tree structures

- **🔧 Customizable Display Characters**: Configure via environment variables
  - `PRETTY_MOD_MODULE_ICON`: Icon for modules (default: 📦)
  - `PRETTY_MOD_FUNCTION_ICON`: Icon for functions (default: ⚡)
  - `PRETTY_MOD_CLASS_ICON`: Icon for classes (default: 🔷)
  - `PRETTY_MOD_CONSTANT_ICON`: Icon for constants (default: 📌)
  - `PRETTY_MOD_EXPORTS_ICON`: Icon for __all__ exports (default: 📜)
  - `PRETTY_MOD_SIGNATURE_ICON`: Icon for signatures (default: 📎)

- **🖥️ ASCII Mode**: For terminals without Unicode support
  ```bash
  PRETTY_MOD_ASCII=1 pretty-mod tree json
  ```

- **🚫 Disable Colors**: For clean output or piping
  ```bash
  PRETTY_MOD_NO_COLOR=1 pretty-mod tree json
  # or use the standard NO_COLOR environment variable
  ```

- **🎯 Custom Colors**: Override any color with hex values
  ```bash
  PRETTY_MOD_MODULE_COLOR=#FF6B6B pretty-mod tree json
  ```

### 🏗️ Technical Improvements

- **Configuration system**: Centralized configuration module with environment variable support
- **Color rendering**: ANSI 24-bit true color support with automatic hex-to-RGB conversion
- **Consistent styling**: Both tree and signature displays use the same configuration system
- **Enhanced AST parsing**: Better handling of complex type annotations and expressions
- **Code organization**: Consolidated signature parsing logic for better maintainability

### 🐛 Bug Fixes

- **Complex type annotations**: Fixed parameter splitting for nested generics
  - Previously: `Callable[[Any], str]` would split incorrectly on the comma
  - Now: Properly handles all nested brackets and quotes in type annotations
  - Affects all complex types like `Dict[str, List[int]]`, `Literal['a', 'b']`, etc.
- **Stdlib module handling**: Built-in modules no longer trigger PyPI download attempts
- **Signature discovery**: Improved recursive search for symbols exported in `__all__`
- **Download messages**: Colored warning messages for better visibility

---

# Release Notes - v0.1.2

## 🔧 Code Quality Improvements

### ✨ New Features

- **Version specifiers**: Support for `@version` syntax (e.g., `pretty-mod tree toml@0.10.2`)
  - Use `@latest` to force download of the latest version
  - Specify exact versions like `@1.2.3`
  - Works with both `tree` and `sig` commands

### 🏗️ Technical Improvements

- **Performance**: Use Rust's native `println!` instead of Python's print for better performance
- **Architecture**: Consolidated shared utilities into a dedicated `utils.rs` module
- **Code organization**: Moved all implementation details out of `lib.rs`, which now only contains PyO3 bindings

---

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