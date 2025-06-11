/// Check if a module is part of the Python standard library
/// This is a simplified version focused on common stdlib modules
pub fn is_stdlib_module(module_name: &str) -> bool {
    // Extract the base module name (before the first dot)
    let base_module = module_name.split('.').next().unwrap_or(module_name);
    
    // Common stdlib modules - this list covers the most frequently used ones
    matches!(
        base_module,
        "abc" | "argparse" | "ast" | "asyncio" | "base64" | "builtins" | "collections"
        | "contextlib" | "copy" | "csv" | "datetime" | "decimal" | "enum" | "functools"
        | "gc" | "glob" | "hashlib" | "importlib" | "inspect" | "io" | "itertools"
        | "json" | "logging" | "math" | "os" | "pathlib" | "pickle" | "platform"
        | "pprint" | "queue" | "random" | "re" | "shutil" | "signal" | "socket"
        | "sqlite3" | "string" | "struct" | "subprocess" | "sys" | "tempfile"
        | "textwrap" | "threading" | "time" | "timeit" | "types" | "typing"
        | "unittest" | "urllib" | "uuid" | "warnings" | "weakref" | "xml" | "zipfile"
        // Also include common internal modules that might be referenced
        | "_ast" | "_collections" | "_functools" | "_io" | "_json" | "_pickle"
        | "_socket" | "_sqlite3" | "_thread" | "_warnings" | "_weakref"
    )
}

/// Check if a stdlib module is implemented in C and has no Python source
/// These modules cannot have signatures extracted from AST
pub fn is_builtin_module(module_name: &str) -> bool {
    let base_module = module_name.split('.').next().unwrap_or(module_name);
    
    matches!(
        base_module,
        "builtins" | "sys" | "gc" | "math" | "time" | "_ast" | "_collections" 
        | "_functools" | "_io" | "_json" | "_pickle" | "_socket" | "_sqlite3" 
        | "_thread" | "_warnings" | "_weakref"
    )
}