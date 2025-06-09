# pretty-mod

A module tree explorer for humans and LLMs.

```bash
» uvx pretty-mod tree json
📦 json
├── 📜 __all__: dump, dumps, load, loads, JSONDecoder, JSONDecodeError, JSONEncoder
├── ⚡ functions: dump, dumps, load, loads
├── 📦 decoder
│   ├── 📜 __all__: JSONDecoder, JSONDecodeError
│   └── 🔷 classes: JSONDecodeError, JSONDecoder
├── 📦 encoder
│   ├── 🔷 classes: JSONEncoder
│   └── ⚡ functions: py_encode_basestring, py_encode_basestring_ascii
├── 📦 scanner
│   └── 📜 __all__: make_scanner
└── 📦 tool
    └── ⚡ functions: main

» uvx pretty-mod sig json:dumps
📎 dumps
├── Parameters:
├── obj
├── skipkeys = False (keyword-only)
├── ensure_ascii = True (keyword-only)
├── check_circular = True (keyword-only)
├── allow_nan = True (keyword-only)
├── cls = None (keyword-only)
├── indent = None (keyword-only)
├── separators = None (keyword-only)
├── default = None (keyword-only)
├── sort_keys = False (keyword-only)
└── kw (**kwargs)
```

## Installation

```bash
uv add pretty-mod
```

## Usage

```python
from pretty_mod import display_tree

# Explore a module structure  
display_tree("json", max_depth=2)
```

<details>
<summary>Example output</summary>

```text
📦 json
└── 📜 __all__: dump, dumps, load, loads, JSONDecoder, JSONDecodeError, JSONEncoder
├── ⚡ functions: dump, dumps, load, loads
├── 📦 decoder
    ├── 📜 __all__: JSONDecoder, JSONDecodeError
    ├── 🔷 classes: JSONDecodeError, JSONDecoder
├── 📦 encoder
    ├── 🔷 classes: JSONEncoder
    ├── ⚡ functions: py_encode_basestring, py_encode_basestring_ascii
├── 📦 scanner
    ├── 📜 __all__: make_scanner
└── 📦 tool
    └── ⚡ functions: main
```
</details>



```python
from pretty_mod import display_signature

# Display function signatures
print(display_signature("json:loads"))
```

<details>
<summary>Example output</summary>

```text
📎 loads
├── Parameters:
├── s
├── cls = None (keyword-only)
├── object_hook = None (keyword-only)
├── parse_float = None (keyword-only)
├── parse_int = None (keyword-only)
├── parse_constant = None (keyword-only)
├── object_pairs_hook = None (keyword-only)
└── kw (**kwargs)
```
</details>

## CLI

Pretty-mod includes a command-line interface for quick exploration:

```bash
# Explore module structure
pretty-mod tree json
pretty-mod tree requests --depth 3

# Display function signatures  
pretty-mod sig json:loads
pretty-mod sig os.path:join

# inspect libraries you don't have installed
uvx --with fastapi pretty-mod tree fastapi.routing

uvx --with fastapi pretty-mod sig fastapi.routing:run_endpoint_function
```

## Examples

See the [`examples/`](examples/) directory for more detailed usage patterns and advanced features.

## Development

```bash
git clone https://github.com/zzstoatzz/pretty-mod.git
cd pretty-mod
uv sync
uv run pytest
```
