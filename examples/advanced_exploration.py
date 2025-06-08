"""
Advanced exploration examples for pretty-mod.

This example shows practical use cases for exploring and understanding codebases,
particularly useful for LLMs and developers working with unfamiliar modules.
"""

from pretty_mod.explorer import ModuleTreeExplorer, display_signature


def compare_module_apis():
    """Compare APIs of related modules to understand their differences."""
    print("🔄 Comparing related module APIs:")
    print("=" * 60)

    modules = ["json", "pickle", "csv"]

    for module_name in modules:
        print(f"\n📦 {module_name.upper()} module API:")
        print("-" * 30)

        explorer = ModuleTreeExplorer(module_name, max_depth=1)
        tree = explorer.explore()

        api = tree.get("api", {})

        if api.get("functions"):
            print(f"Functions: {', '.join(sorted(api['functions']))}")
        if api.get("classes"):
            print(f"Classes: {', '.join(sorted(api['classes']))}")
        if api.get("constants"):
            print(f"Constants: {', '.join(sorted(api['constants']))}")


def explore_third_party_module():
    """Example of exploring a third-party module (if available)."""
    print("\n🌐 Exploring third-party modules:")
    print("=" * 60)

    # Try to explore common third-party modules
    modules_to_try = ["requests", "numpy", "pandas", "pathlib"]

    for module_name in modules_to_try:
        try:
            explorer = ModuleTreeExplorer(module_name, max_depth=1)
            tree = explorer.explore()

            if tree.get("api"):
                print(f"\n✅ Found {module_name}:")
                print(explorer.get_tree_string())
                break
        except Exception:
            continue
    else:
        print("\n⚠️  No common third-party modules found.")
        print("Try installing requests or another package to see this in action!")


def function_signature_analysis():
    """Analyze function signatures to understand usage patterns."""
    print("\n🔍 Function Signature Analysis:")
    print("=" * 60)

    # Functions with different signature patterns
    functions = [
        ("builtins:print", "Built-in with *args"),
        ("json:dumps", "Serialization function"),
        ("os.path:join", "Path manipulation"),
        ("re:match", "Pattern matching"),
    ]

    for func_path, description in functions:
        try:
            print(f"\n{description}:")
            print(display_signature(func_path))
        except Exception as e:
            print(f"❌ Could not analyze {func_path}: {e}")


def find_similar_functions():
    """Find functions with similar names across different modules."""
    print("\n🎯 Finding similar functions across modules:")
    print("=" * 60)

    target_functions = ["loads", "dumps", "open"]
    modules = ["json", "pickle", "os"]

    for func_name in target_functions:
        print(f"\nLooking for '{func_name}' functions:")

        for module_name in modules:
            try:
                explorer = ModuleTreeExplorer(module_name, max_depth=1)
                tree = explorer.explore()
                functions = tree.get("api", {}).get("functions", [])

                matching = [f for f in functions if func_name in f.lower()]
                if matching:
                    print(f"  📦 {module_name}: {', '.join(matching)}")
            except Exception:
                continue


def generate_module_summary():
    """Generate a comprehensive summary of a module."""
    print("\n📊 Module Summary Generator:")
    print("=" * 60)

    module_name = "datetime"
    explorer = ModuleTreeExplorer(module_name, max_depth=2)
    tree = explorer.explore()

    print(f"Summary for '{module_name}' module:")
    print("-" * 40)

    api = tree.get("api", {})
    submodules = tree.get("submodules", {})

    # Main module stats
    print(f"Classes: {len(api.get('classes', []))}")
    print(f"Functions: {len(api.get('functions', []))}")
    print(f"Constants: {len(api.get('constants', []))}")
    print(f"Submodules: {len(submodules)}")

    if api.get("all"):
        print(f"Public API (__all__): {len(api['all'])} items")

    # Show class details
    if api.get("classes"):
        print(f"\nMain classes: {', '.join(api['classes'][:5])}")
        if len(api["classes"]) > 5:
            print(f"... and {len(api['classes']) - 5} more")

    # Show submodule details
    if submodules:
        print("\nSubmodules:")
        for sub_name, sub_tree in list(submodules.items())[:3]:
            sub_api = sub_tree.get("api", {})
            total_items = (
                len(sub_api.get("classes", []))
                + len(sub_api.get("functions", []))
                + len(sub_api.get("constants", []))
            )
            print(f"  📦 {sub_name}: {total_items} public items")


def interactive_exploration():
    """Show how to use pretty-mod for interactive exploration."""
    print("\n🎮 Interactive Exploration Pattern:")
    print("=" * 60)

    print("Here's how you might explore an unfamiliar module step by step:")
    print()

    # Step 1: High-level overview
    print("1. Get a high-level overview:")
    print("   explorer = ModuleTreeExplorer('os', max_depth=1)")
    print("   print(explorer.get_tree_string())")
    print()

    # Step 2: Deep dive
    print("2. Deep dive into interesting submodules:")
    print("   explorer = ModuleTreeExplorer('os.path', max_depth=2)")
    print("   print(explorer.get_tree_string())")
    print()

    # Step 3: Function analysis
    print("3. Analyze specific functions:")
    print("   print(display_signature('os.path:join'))")
    print("   print(display_signature('os:getcwd'))")
    print()

    # Step 4: Dynamic testing
    print("4. Dynamically import and test:")
    print("   join_func = import_object('os.path:join')")
    print("   result = join_func('home', 'user', 'documents')")
    print()

    # Demonstrate this pattern
    print("Let's run this pattern on the 'pathlib' module:")
    try:
        explorer = ModuleTreeExplorer("pathlib", max_depth=1)
        print(explorer.get_tree_string())
    except Exception as e:
        print(f"Could not explore pathlib: {e}")


if __name__ == "__main__":
    print("🚀 Advanced pretty-mod exploration examples")
    print("=" * 70)

    compare_module_apis()
    explore_third_party_module()
    function_signature_analysis()
    find_similar_functions()
    generate_module_summary()
    interactive_exploration()

    print("\n🎯 These patterns help you quickly understand any Python codebase!")
