# pretty-mod

a python module tree explorer for LLMs (and humans)

> [!IMPORTANT]
> for all versions `>=0.1.0`, wheels for different operating systems are built via `maturin` and published to pypi, install `<0.1.0` for a pure python version

> [!NOTE]
> Starting from v0.2.0, output includes colors by default. Use `PRETTY_MOD_NO_COLOR=1` to disable.

```bash
# Explore module structure
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

# Inspect function signatures
» uvx pretty-mod sig json:dumps
📎 dumps
├── Parameters:
├── obj
├── *
├── skipkeys=False
├── ensure_ascii=True
├── check_circular=True
├── allow_nan=True
├── cls=None
├── indent=None
├── separators=None
├── default=None
├── sort_keys=False
└── **kw

# Auto-download packages from PyPI (no install needed!)
» uvx pretty-mod tree requests --quiet
📦 requests
├── 📜 __all__: delete, get, head, options, patch, post, put, request
├── ⚡ functions: check_compatibility, delete, get, head, options, patch, post, put, request
├── 🔷 classes: ConnectTimeout, ConnectionError, DependencyWarning, FileModeWarning, HTTPError, JSONDecodeError, NullHandler, PreparedRequest, ReadTimeout, Request, RequestException, RequestsDependencyWarning, Response, Session, Timeout, TooManyRedirects, URLRequired
├── 📌 constants: __author__, __author_email__, __build__, __cake__, __copyright__, __description__, __license__, __title__, __url__, codes
├── 📦 adapters
│   ├── 🔷 classes: BaseAdapter, HTTPAdapter
│   └── 📌 constants: DEFAULT_POOL_TIMEOUT, DEFAULT_POOLBLOCK, DEFAULT_POOLSIZE, DEFAULT_RETRIES
├── 📦 api
│   └── ⚡ functions: delete, get, head, options, patch, post, put, request
├── 📦 auth
│   └── 🔷 classes: AuthBase, HTTPBasicAuth, HTTPDigestAuth, HTTPProxyAuth
├── 📦 certs
├── 📦 compat
├── 📦 cookies
├── 📦 exceptions
├── 📦 help
├── 📦 hooks
├── 📦 models
├── 📦 sessions
├── 📦 status_codes
├── 📦 structures
└── 📦 utils
```

## installation

```bash
uv add pretty-mod
```

## usage

```python
from pretty_mod import display_tree

# Explore a module structure  
display_tree("collections", max_depth=2)
```

<details>
<summary>Example output</summary>

```text
display_tree("collections", max_depth=2)

📦 collections
├── 📜 __all__: ChainMap, Counter, OrderedDict, UserDict, UserList, UserString, defaultdict, deque, namedtuple
├── 🔷 classes: ChainMap, Counter, OrderedDict, UserDict, UserList, UserString, defaultdict, deque
├── ⚡ functions: namedtuple
└── 📦 abc
    ├── 📜 __all__: Awaitable, Coroutine, AsyncIterable, AsyncIterator, AsyncGenerator, Hashable, Iterable, Iterator, Generator, Reversible, Sized, Container, Callable, Collection, Set, MutableSet, Mapping, MutableMapping, MappingView, KeysView, ItemsView, ValuesView, Sequence, MutableSequence, ByteString, Buffer
    └── 🔷 classes: AsyncGenerator, AsyncIterable, AsyncIterator, Awaitable, Buffer, ByteString, Callable, Collection, Container, Coroutine, Generator, Hashable, ItemsView, Iterable, Iterator, KeysView, Mapping, MappingView, MutableMapping, MutableSequence, MutableSet, Reversible, Sequence, Set, Sized, ValuesView
```
</details>



```python
from pretty_mod import display_signature

# Display the signature of a callable (function, class constructor, etc.)
print(display_signature("json:loads"))
```

<details>
<summary>Example output</summary>

```text
📎 loads
├── Parameters:
├── s
├── *
├── cls=None
├── object_hook=None
├── parse_float=None
├── parse_int=None
├── parse_constant=None
├── object_pairs_hook=None
└── **kw
```
</details>

## CLI

Pretty-mod includes a command-line interface for quick exploration:

> [!IMPORTANT]
> all commands below can be run ephemerally with `uvx`, e.g. `uvx pretty-mod tree json`

```bash
# Explore module structure
pretty-mod tree json

# Go deeper into the tree with --depth
pretty-mod tree requests --depth 3

# Display function signatures  
pretty-mod sig json:loads
pretty-mod sig os.path:join

# Explore packages even without having them installed
pretty-mod tree django
pretty-mod tree flask --depth 1

# Use --quiet to suppress download messages
pretty-mod tree requests --quiet

# Version specifiers - explore specific versions
pretty-mod tree toml@0.10.2
pretty-mod sig toml@0.10.2:loads

# Submodules with version specifiers (correct syntax)
pretty-mod tree prefect.server@2.10.0  # ✅ Works
pretty-mod tree prefect@2.10.0.server  # ❌ Invalid - version must come last

# Package name differs from module name
pretty-mod tree pydocket::docket       # PyPI package 'pydocket' contains module 'docket'
pretty-mod tree pillow::PIL            # PyPI package 'pillow' contains module 'PIL'
pretty-mod tree pillow::PIL@10.0.0    # Specific version of pillow
pretty-mod sig pillow::PIL.Image:open  # Works with signatures too
```

## customization

pretty-mod supports extensive customization through environment variables:

### display characters

```bash
# Use ASCII-only mode for terminals without Unicode support
PRETTY_MOD_ASCII=1 pretty-mod tree json

# Customize individual icons
PRETTY_MOD_MODULE_ICON="[M]" pretty-mod tree json
PRETTY_MOD_FUNCTION_ICON="fn" pretty-mod tree json
PRETTY_MOD_CLASS_ICON="cls" pretty-mod tree json
```

### colors

pretty-mod uses an earth-tone color scheme by default:

```bash
# Disable colors entirely
PRETTY_MOD_NO_COLOR=1 pretty-mod tree json
# or use the standard NO_COLOR environment variable
NO_COLOR=1 pretty-mod tree json

# Override specific colors with hex values
PRETTY_MOD_MODULE_COLOR="#FF6B6B" pretty-mod tree json
PRETTY_MOD_FUNCTION_COLOR="#4ECDC4" pretty-mod tree json
```

available color environment variables:
- `PRETTY_MOD_MODULE_COLOR` - Modules/packages (default: #8B7355)
- `PRETTY_MOD_FUNCTION_COLOR` - Functions (default: #6B8E23)
- `PRETTY_MOD_CLASS_COLOR` - Classes (default: #4682B4)
- `PRETTY_MOD_CONSTANT_COLOR` - Constants (default: #BC8F8F)
- `PRETTY_MOD_EXPORTS_COLOR` - __all__ exports (default: #9370DB)
- `PRETTY_MOD_SIGNATURE_COLOR` - Signatures (default: #5F9EA0)
- `PRETTY_MOD_TREE_COLOR` - Tree structure lines (default: #696969)
- `PRETTY_MOD_PARAM_COLOR` - Parameter names (default: #708090)
- `PRETTY_MOD_TYPE_COLOR` - Type annotations (default: #778899)
- `PRETTY_MOD_DEFAULT_COLOR` - Default values (default: #8FBC8F)

## examples

see the [`examples/`](examples/) directory for more detailed usage patterns and advanced features.

## development

```bash
gh repo clone zzstoatzz/pretty-mod && cd pretty-mod
just --list # see https://github.com/casey/just
```

<details>
<summary>Performance Testing</summary>

The performance test script (`scripts/perf_test.py`) supports both single-run exploration and proper benchmarking with multiple iterations:

```bash
# Run a proper benchmark with multiple iterations
./scripts/perf_test.py json --benchmark
./scripts/perf_test.py urllib --benchmark --runs 100 --warmup 10

# Compare performance between local and published versions
just compare-perf prefect 2

# Benchmark multiple modules
just benchmark-modules

# Or use shell timing for quick single-run comparisons
time ./scripts/perf_test.py numpy --depth 3
time uvx pretty-mod tree numpy --depth 3
```

Benchmark mode provides:
- Warmup runs to account for cold starts
- Multiple iterations for statistical significance
- Mean, standard deviation, min/max timing statistics
- Silent operation (no tree output) for accurate timing

</details>