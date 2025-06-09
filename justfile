# Check for uv installation
check-uv:
    #!/usr/bin/env sh
    if ! command -v uv >/dev/null 2>&1; then
        echo "uv is not installed or not found in expected locations."
        case "$(uname)" in
            "Darwin")
                echo "To install uv on macOS, run one of:"
                echo "• brew install uv"
                echo "• curl -LsSf https://astral.sh/uv/install.sh | sh"
                ;;
            "Linux")
                echo "To install uv, run:"
                echo "• curl -LsSf https://astral.sh/uv/install.sh | sh"
                ;;
            *)
                echo "To install uv, visit: https://github.com/astral-sh/uv"
                ;;
        esac
        exit 1
    fi

# Install development dependencies
install: check-uv
    uv sync

typecheck: check-uv
    uv run ty check python tests

# Clean up environment
clean: check-uv
    deactivate || true
    rm -rf .venv

run-pre-commits: check-uv
    uv run pre-commit run --all-files

# Build Rust extension in release mode for performance
build:
    uvx maturin develop --uv --release

# Run tests after building
test: build
    uv run pytest -v

# Run performance test
perf MODULE='json': build
    ./scripts/perf_test.py {{MODULE}} --benchmark

# Profile a module to find bottlenecks  
profile MODULE='json': build
    ./scripts/profile.py {{MODULE}}