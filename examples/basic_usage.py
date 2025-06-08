"""
Basic usage examples for pretty-mod.

This example demonstrates how to explore module structures and display function signatures.
"""

from pretty_mod.explorer import ModuleTreeExplorer, display_signature, import_object


def explore_json_module():
    """Example: Explore the built-in json module structure."""
    print("🔍 Exploring the json module:")
    print("=" * 50)

    explorer = ModuleTreeExplorer("json", max_depth=2)
    print(explorer.get_tree_string())
    print()


def explore_function_signature():
    """Example: Display function signatures in a readable format."""
    print("📋 Function Signature Examples:")
    print("=" * 50)

    # Example with a built-in function
    print("Built-in function:")
    print(display_signature("builtins:len"))
    print()

    # Example with a module function
    print("Module function:")
    print(display_signature("json:loads"))
    print()


def import_and_use_objects():
    """Example: Import objects dynamically using import paths."""
    print("📦 Dynamic Import Examples:")
    print("=" * 50)

    # Import using colon syntax
    json_loads = import_object("json:loads")
    print(f"Imported json.loads: {json_loads}")

    # Test the imported function
    test_data = '{"name": "pretty-mod", "type": "module"}'
    parsed = json_loads(test_data)
    print(f"Parsed JSON: {parsed}")
    print()

    # Import using dot syntax
    sys_version = import_object("sys.version_info")
    print(
        f"Python version: {sys_version.major}.{sys_version.minor}.{sys_version.micro}"
    )
    print()


def explore_custom_depth():
    """Example: Explore modules with different depth limits."""
    print("🏗️ Exploring with different depths:")
    print("=" * 50)

    # Shallow exploration
    print("Shallow exploration (depth=1):")
    explorer_shallow = ModuleTreeExplorer("urllib", max_depth=1)
    print(explorer_shallow.get_tree_string())
    print()

    # Deeper exploration
    print("Deeper exploration (depth=2):")
    explorer_deep = ModuleTreeExplorer("urllib", max_depth=2)
    print(explorer_deep.get_tree_string())
    print()


if __name__ == "__main__":
    print("🎉 Welcome to pretty-mod examples!")
    print("=" * 60)
    print()

    explore_json_module()
    explore_function_signature()
    import_and_use_objects()
    explore_custom_depth()

    print("✨ That's pretty-mod in action! ✨")
