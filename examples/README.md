# Pretty-Mod Examples

This directory contains practical examples demonstrating how to use `pretty-mod` for module exploration and code understanding.

## Examples

### 📚 `basic_usage.py`
Demonstrates core functionality:
- Exploring module structures with `ModuleTreeExplorer`
- Displaying function signatures with `display_signature`
- Dynamic imports with `import_object`
- Working with different exploration depths

**Run it:**
```bash
uv run python examples/basic_usage.py
```

### 🚀 `advanced_exploration.py`
Shows practical use cases:
- Comparing APIs across related modules
- Generating module summaries
- Finding similar functions across codebases
- Interactive exploration patterns for unfamiliar code

**Run it:**
```bash
uv run python examples/advanced_exploration.py
```

## Key Use Cases

- **Code Discovery**: Quickly understand the structure of unfamiliar modules
- **API Comparison**: Compare similar libraries to understand differences
- **Documentation**: Generate overviews of module capabilities
- **LLM Integration**: Perfect for AI assistants exploring codebases
- **Interactive Development**: Dynamically explore and test code

## Tips

- Start with shallow exploration (`max_depth=1`) for quick overviews
- Use deeper exploration for detailed analysis
- Combine signature display with dynamic imports for testing
- Great for exploring both built-in and third-party modules 