# overview

`pretty-mod` is a python package built on pyo3 to explore python packages for LLMs

## getting oriented

- read @RELEASE_NOTES.md, @README.md, @pyproject.toml, and @justfile

## run the tests

```
just test
```

if you only need to build (`just test` runs `just build` automatically)

```
just build
```

## run the local python package

```
uv run pretty-mod tree fastapi.routing
```

## run the remote python package

```
uvx pretty-mod tree fastapi.routing
```

# IMPORTANT
- avoid breaking changes to the public api defined by type stubs in @python/pretty_mod/_pretty_mod.pyi