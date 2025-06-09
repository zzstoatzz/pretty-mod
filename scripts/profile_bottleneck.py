#!/usr/bin/env python3
"""Profile specific bottlenecks with detailed timing."""

import ast
import os
import time
from pathlib import Path

# Test the email._header_value_parser file specifically
file_path = Path(
    "/Users/nate/Library/Application Support/uv/python/cpython-3.12.8-macos-aarch64-none/lib/python3.12/email/_header_value_parser.py"
)

print(f"Testing file: {file_path}")
print(f"File size: {os.path.getsize(file_path):,} bytes")

# Test 1: Just reading the file
start = time.perf_counter()
with open(file_path) as f:
    content = f.read()
read_time = time.perf_counter() - start
print(f"\n1. File read time: {read_time:.4f}s")

# Test 2: Python AST parsing
start = time.perf_counter()
python_ast = ast.parse(content)
python_parse_time = time.perf_counter() - start
print(f"2. Python AST parse time: {python_parse_time:.4f}s")

# Test 3: rustpython-parser via our Rust code
from pretty_mod.explorer import ModuleTreeExplorer

start = time.perf_counter()
explorer = ModuleTreeExplorer("email._header_value_parser", max_depth=1)
tree = explorer.explore()
rust_total_time = time.perf_counter() - start
print(f"3. Rust total exploration time: {rust_total_time:.4f}s")

# Test 4: Just the ModuleInfo.from_python_file call
# We'll need to expose this for testing
print("\nBreakdown:")
print(
    f"- Reading overhead: {read_time:.4f}s ({read_time / rust_total_time * 100:.1f}%)"
)
print(f"- Python can parse in: {python_parse_time:.4f}s")
print(f"- Rust takes total: {rust_total_time:.4f}s")
print(f"- Rust is {rust_total_time / python_parse_time:.1f}x slower than Python AST")

# Test 5: Let's test the published version for comparison
import subprocess

start = time.perf_counter()
result = subprocess.run(
    ["uvx", "pretty-mod", "tree", "email._header_value_parser", "--depth", "1"],
    capture_output=True,
    text=True,
)
published_time = time.perf_counter() - start
print(f"\n4. Published (Python) version via uvx: {published_time:.4f}s")

# Test 6: Import and introspect timing
import importlib
import inspect

start = time.perf_counter()
module = importlib.import_module("email._header_value_parser")
import_time = time.perf_counter() - start

start = time.perf_counter()
funcs = [name for name, obj in inspect.getmembers(module, inspect.isfunction)]
classes = [name for name, obj in inspect.getmembers(module, inspect.isclass)]
introspect_time = time.perf_counter() - start

print("\n5. Python import + introspection:")
print(f"   - Import time: {import_time:.4f}s")
print(f"   - Introspection time: {introspect_time:.4f}s")
print(f"   - Total: {import_time + introspect_time:.4f}s")
print(f"   - Found {len(funcs)} functions, {len(classes)} classes")

# NEW: Test the recursion impact
print("\n\n=== RECURSION IMPACT TEST ===")
print("Testing how depth affects total time:\n")

from pretty_mod.explorer import ModuleTreeExplorer

for depth in [1, 2, 3]:
    start = time.perf_counter()
    e = ModuleTreeExplorer("email", max_depth=depth)
    tree = e.explore()
    duration = time.perf_counter() - start

    # Count total modules explored
    def count_modules(t):
        count = 1
        for sub in t.get("submodules", {}).values():
            count += count_modules(sub)
        return count

    module_count = count_modules(tree)
    print(
        f"Depth {depth}: {duration:.4f}s for {module_count} modules = {duration / module_count:.4f}s per module"
    )

# Check AST iteration overhead
print("\n\n=== AST ITERATION OVERHEAD ===")
with open(file_path) as f:
    content = f.read()
tree = ast.parse(content)

# Time single iteration
start = time.perf_counter()
count = 0
for stmt in tree.body:
    if isinstance(stmt, ast.FunctionDef):
        count += 1
single_iter_time = time.perf_counter() - start

# Time double iteration (what our code does)
start = time.perf_counter()
# First pass for __all__
for stmt in tree.body:
    if isinstance(stmt, ast.Assign):
        pass
# Second pass for functions
for stmt in tree.body:
    if isinstance(stmt, ast.FunctionDef):
        pass
double_iter_time = time.perf_counter() - start

print(f"Single iteration of {len(tree.body)} statements: {single_iter_time:.6f}s")
print(f"Double iteration: {double_iter_time:.6f}s")
print(f"Overhead: {double_iter_time - single_iter_time:.6f}s")
