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
explorer = ModuleTreeExplorer("datetime", max_depth=2)
tree_data = explorer.explore()

# The tree data is a dictionary with module information
print(f"\nModule: {tree_data['module']}")
print(f"Path: {tree_data['path']}")

# Access the API information
api = tree_data.get("api", {})
print("\nAPI contents:")
print(f"  Functions: {', '.join(api.get('functions', []))}")
print(f"  Classes: {', '.join(api.get('classes', []))}")
print(f"  Constants: {', '.join(api.get('constants', []))}")

# Get the formatted tree string
print("\n3️⃣ Getting formatted output:")
print(explorer.get_tree_string())

# Explore submodules
print("\n4️⃣ Exploring submodules:")
if submodules := tree_data.get("submodules", {}):
    for name, subtree in submodules.items():
        sub_api = subtree.get("api", {})
        total = (
            len(sub_api.get("functions", []))
            + len(sub_api.get("classes", []))
            + len(sub_api.get("constants", []))
        )
        print(f"  📦 {name}: {total} items")

# Different depth levels
print("\n5️⃣ Controlling exploration depth:")
print("\nShallow (depth=1):")
display_tree("email", max_depth=1)

print("\nDeeper (depth=3):")
display_tree("email.mime", max_depth=3)

# Auto-download with version
print("\n6️⃣ Auto-download with version specifiers:")
print("\nExploring specific version of a package:")
display_tree("click@8.0.0", max_depth=1, quiet=True)

# Quiet mode
print("\n7️⃣ Using quiet mode to suppress download messages:")
display_tree("six", max_depth=1, quiet=True)

# Error handling
print("\n8️⃣ Error handling:")
try:
    display_tree("module:with:colons", max_depth=1)
except ValueError as e:
    print(f"❌ Error: {e}")

# Performance tip
print("\n💡 Performance tip:")
print("pretty-mod uses Rust-based AST parsing for incredible speed.")
print("It's ~12x faster than import-based exploration for large packages!")
