"""
Welcome to pretty-mod!

This is a Python library for exploring and understanding Python modules programmatically.
Perfect for LLMs, developers exploring unfamiliar code, or building developer tools.

Key features:
- Explore module structures without importing them (filesystem-based discovery)
- Display function signatures in a beautiful tree format
- Auto-download packages from PyPI when needed
- Support for version specifiers (e.g., requests@2.31.0)
"""

from pretty_mod import display_signature, display_tree

# Quick overview - let's explore Python's json module
print("📦 Exploring Python's json module:")
display_tree("json", max_depth=1)

print("\n📎 Looking at a function signature:")
print(display_signature("json:dumps"))

print("\n✨ Auto-download example (if requests isn't installed):")
# This will download requests from PyPI if not installed!
display_tree("requests", max_depth=1, quiet=True)

print("\n🎯 Version specifiers - explore specific versions:")
# Download and explore a specific version
display_tree("toml@0.10.2", max_depth=1, quiet=True)
