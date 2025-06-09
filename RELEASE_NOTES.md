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