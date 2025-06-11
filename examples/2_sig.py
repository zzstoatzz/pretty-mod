"""
Deep dive into function signature inspection.

The display_signature function shows detailed parameter information,
making it easy to understand how to call functions.
"""

from pretty_mod import display_signature

print("📝 FUNCTION SIGNATURE INSPECTION")
print("=" * 60)

# Basic signature display
print("\n1️⃣ Basic usage - display a function signature:")
print(display_signature("json:dumps"))

# Different import syntaxes
print("\n2️⃣ Different ways to specify functions:")
print("\nColon syntax (module:function):")
print(display_signature("os:getcwd"))

print("\nDot syntax for nested attributes:")
print(display_signature("os.path.join"))

print("\nBuiltin functions:")
print(display_signature("builtins:print"))

# Exploring class methods
print("\n3️⃣ Class methods and complex signatures:")
print("\nClass constructor:")
print(display_signature("datetime.datetime"))

print("\nStatic method:")
print(display_signature("datetime.datetime.now"))

# Parameters with defaults and annotations
print("\n4️⃣ Rich parameter information:")
print("\nFunction with defaults and keyword-only args:")
print(display_signature("json:dump"))

# Understanding complex signatures
print("\n5️⃣ Understanding complex signatures:")
print("Inspecting urllib.parse.urlencode:")
print(display_signature("urllib.parse.urlencode"))

# Auto-download for signatures
print("\n6️⃣ Auto-download for external packages:")
print("\nExploring a function from a specific version:")
print(display_signature("toml@0.10.2:loads", quiet=True))

# Error handling
print("\n7️⃣ Error handling:")

print("\nNon-existent function:")
print(display_signature("fake_module:fake_function"))

print("\nNon-callable object:")
print(display_signature("sys:path"))

# Advanced: exploring decorators and special functions
print("\n8️⃣ Special functions and decorators:")
print("\nExploring functools.lru_cache:")
print(display_signature("functools:lru_cache"))

print("\nContext managers:")
print(display_signature("contextlib.contextmanager"))

# Performance note
print("\n💡 Pro tip:")
print("display_signature returns a string, making it perfect for:")
print("- Documentation generation")
print("- LLM-powered code analysis")
print("- Interactive development environments")
