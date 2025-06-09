#!/usr/bin/env -S uv run --with-editable . --script --quiet
# /// script
# requires-python = ">=3.12"
# ///
"""
Explore a module using pretty-mod.

Usage:
    ./scripts/perf_test.py MODULE [--depth N]
    ./scripts/perf_test.py MODULE --benchmark [--runs N] [--warmup N]

Examples:
    ./scripts/perf_test.py json
    ./scripts/perf_test.py urllib --depth 3
    ./scripts/perf_test.py prefect --benchmark --runs 100
"""

import argparse
import sys
import time
from statistics import mean, stdev


def explore_module(module_name: str, depth: int = 2, silent: bool = False) -> float:
    """Explore a module and optionally print its tree. Returns time taken."""
    from pretty_mod import display_tree
    from pretty_mod.explorer import ModuleTreeExplorer

    start = time.perf_counter()
    try:
        if silent:
            # Just explore without printing
            explorer = ModuleTreeExplorer(module_name, max_depth=depth)
            explorer.explore()
        else:
            # Normal display mode
            display_tree(module_name, max_depth=depth)
        elapsed = time.perf_counter() - start
        return elapsed
    except Exception as e:
        print(f"Error exploring {module_name}: {e}", file=sys.stderr)
        sys.exit(1)


def benchmark_module(
    module_name: str, depth: int = 2, runs: int = 50, warmup: int = 5
) -> None:
    """Benchmark module exploration with multiple runs."""
    print(f"Benchmarking {module_name} (depth={depth})")
    print(f"Warmup: {warmup} runs, Benchmark: {runs} runs\n")

    # Warmup runs
    print("Warming up...", end="", flush=True)
    for _ in range(warmup):
        explore_module(module_name, depth, silent=True)
        print(".", end="", flush=True)
    print(" done")

    # Actual benchmark runs
    times = []
    print(f"Running {runs} iterations...", end="", flush=True)
    for i in range(runs):
        elapsed = explore_module(module_name, depth, silent=True)
        times.append(elapsed)
        if (i + 1) % 10 == 0:
            print(".", end="", flush=True)
    print(" done\n")

    # Calculate statistics
    avg = mean(times)
    std = stdev(times) if len(times) > 1 else 0.0
    min_time = min(times)
    max_time = max(times)

    # Convert to milliseconds for readability
    print(f"Results for {module_name}:")
    print(f"  Average: {avg * 1000:.2f}ms ± {std * 1000:.2f}ms")
    print(f"  Min:     {min_time * 1000:.2f}ms")
    print(f"  Max:     {max_time * 1000:.2f}ms")
    print(f"  Total:   {sum(times) * 1000:.2f}ms for {runs} runs")


def main():
    parser = argparse.ArgumentParser(
        description="Explore a Python module using pretty-mod"
    )
    parser.add_argument("module", help="Module to explore (e.g. json, urllib, prefect)")
    parser.add_argument(
        "--depth",
        "-d",
        type=int,
        default=2,
        help="Maximum depth to explore (default: 2)",
    )
    parser.add_argument(
        "--benchmark",
        "-b",
        action="store_true",
        help="Run benchmark mode with multiple iterations",
    )
    parser.add_argument(
        "--runs",
        "-r",
        type=int,
        default=50,
        help="Number of benchmark runs (default: 50)",
    )
    parser.add_argument(
        "--warmup",
        "-w",
        type=int,
        default=5,
        help="Number of warmup runs (default: 5)",
    )
    args = parser.parse_args()

    if args.benchmark:
        benchmark_module(args.module, args.depth, args.runs, args.warmup)
    else:
        explore_module(args.module, args.depth)


if __name__ == "__main__":
    main()
