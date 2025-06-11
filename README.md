# pretty-mod

a python module tree explorer for LLMs (and humans)

> [!IMPORTANT]
> for all versions `>=0.1.0`, wheels for different operating systems are built via `maturin` and published to pypi, install `<0.1.0` for a pure python version

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

## Installation

```bash
uv add pretty-mod
```

## Usage

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
```

## Examples

See the [`examples/`](examples/) directory for more detailed usage patterns and advanced features.

## Development

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