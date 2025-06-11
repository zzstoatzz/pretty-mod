"""
Deep dive into module tree exploration.

The display_tree function and ModuleTreeExplorer class provide powerful ways
to understand module structures without executing code.
"""

from pretty_mod import display_tree
from pretty_mod.explorer import ModuleTreeExplorer

print("🌳 MODULE TREE EXPLORATION")
print("=" * 60)

# Basic tree display
print("\n1️⃣ Basic usage - display a module tree:")
display_tree("collections", max_depth=2)

# Using the explorer class directly for more control
print("\n2️⃣ Using ModuleTreeExplorer for programmatic access:")

# Note: ModuleTreeExplorer provides a lazy interface
# The tree is only explored when you call get_tree_string() or access the tree property
explorer = ModuleTreeExplorer("datetime", max_depth=1)

# Get the formatted tree string (this triggers exploration)
print("\nFormatted output:")
print(explorer.get_tree_string())

# Properties available
print("\nExplorer properties:")
print(f"  Root module: {explorer.root_module_path}")
print(f"  Max depth: {explorer.max_depth}")

# Different depth levels
print("\n3️⃣ Controlling exploration depth:")
print("\nShallow (depth=1):")
display_tree("email", max_depth=1)

print("\nDeeper (depth=3):")
display_tree("email.mime", max_depth=3)

# Auto-download with version
print("\n4️⃣ Auto-download with version specifiers:")
print("\nExploring specific version of a package:")
display_tree("click@8.0.0", max_depth=1, quiet=True)

# Quiet mode
print("\n5️⃣ Using quiet mode to suppress download messages:")
display_tree("six", max_depth=1, quiet=True)

# Error handling
print("\n6️⃣ Error handling:")
try:
    display_tree("module:with:colons", max_depth=1)
except ValueError as e:
    print(f"❌ Error: {e}")

# Performance tip
print("\n💡 Performance tip:")
print("pretty-mod uses Rust-based AST parsing for incredible speed.")
print("It's ~12x faster than import-based exploration for large packages!")
