# overview

`pretty-mod` is a python package built on pyo3 to explore python packages for LLMs

# Getting oriented

- read @RELEASE_NOTES.md, @README.md, @pyproject.toml, and @justfile

# run the tests

```
just test
```

if for some reason you need to only build (just test does this automatically)

```
just build
```

# run the local python package

```
uv run pretty-mod tree fastapi.routing
```

# run the remote python package

```
uvx pretty-mod tree fastapi.routing
```