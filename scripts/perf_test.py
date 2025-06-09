#!/usr/bin/env -S uv run --with-editable . --script --quiet
# /// script
# requires-python = ">=3.12"
# dependencies = ["rich>=13.0.0", "prefect", "requests", "pandas", "numpy", "fastapi"]
# ///
"""
Comprehensive performance analysis for pretty-mod.

This script consolidates all performance testing functionality:
- Benchmarking against published version
- Profiling specific bottlenecks
- Testing on various module sizes
- Analyzing Python vs Rust performance

Usage:
    ./scripts/perf_test.py               # Run standard benchmarks
    ./scripts/perf_test.py --bottleneck email  # Analyze specific module
    ./scripts/perf_test.py --stress      # Test large packages
    ./scripts/perf_test.py --quick       # Quick test (3 modules only)
"""

import ast
import importlib
import inspect
import subprocess
import sys
import time
import warnings
from collections import namedtuple
from contextlib import redirect_stderr, redirect_stdout
from io import StringIO

from rich.console import Console
from rich.panel import Panel
from rich.table import Table
from rich.text import Text

ExplorationResult = namedtuple(
    "ExplorationResult",
    [
        "duration",
        "total_functions",
        "total_classes",
        "total_submodules",
        "has_warnings",
    ],
)

ProfileResult = namedtuple(
    "ProfileResult",
    [
        "import_time",
        "introspect_time",
        "ast_parse_time",
        "file_size",
        "function_count",
        "class_count",
    ],
)


def count_items_recursive(tree: dict) -> tuple[int, int, int]:
    """Recursively count all functions, classes, and submodules in the tree."""
    api = tree.get("api", {})
    submodules = tree.get("submodules", {})

    total_functions = len(api.get("functions", []))
    total_classes = len(api.get("classes", []))
    total_submodules = len(submodules)

    # Recursively count in submodules
    for submodule in submodules.values():
        sub_funcs, sub_classes, sub_submods = count_items_recursive(submodule)
        total_functions += sub_funcs
        total_classes += sub_classes
        total_submodules += sub_submods

    return total_functions, total_classes, total_submodules


def time_exploration_local(module_name: str, max_depth: int = 2) -> ExplorationResult:
    """Time how long it takes to explore a module tree using local version."""
    from pretty_mod.explorer import ModuleTreeExplorer

    start = time.perf_counter()
    has_warnings = False

    try:
        stdout_capture = StringIO()
        stderr_capture = StringIO()

        with warnings.catch_warnings(record=True) as w:
            warnings.simplefilter("always")

            with redirect_stdout(stdout_capture), redirect_stderr(stderr_capture):
                explorer = ModuleTreeExplorer(module_name, max_depth=max_depth)
                tree = explorer.explore()

        has_warnings = (
            len(w) > 0
            or stdout_capture.getvalue().strip()
            or stderr_capture.getvalue().strip()
        )

        end = time.perf_counter()
        duration = end - start

        # Count recursively
        total_functions, total_classes, total_submodules = count_items_recursive(tree)

        return ExplorationResult(
            duration, total_functions, total_classes, total_submodules, has_warnings
        )

    except ImportError:
        return ExplorationResult(0.0, 0, 0, 0, False)


def time_exploration_published(module_name: str, max_depth: int = 2) -> float:
    """Time how long it takes using the published version via uvx."""
    cmd = ["uvx", "pretty-mod", "tree", module_name, "--depth", str(max_depth)]

    start = time.perf_counter()

    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)

        end = time.perf_counter()

        if result.returncode == 0:
            return end - start
        else:
            return 0.0

    except subprocess.TimeoutExpired:
        return 0.0
    except Exception:
        return 0.0


def profile_python_native(module_name: str) -> ProfileResult:
    """Profile Python's native import and introspection capabilities."""
    result = {
        "import_time": 0.0,
        "introspect_time": 0.0,
        "ast_parse_time": 0.0,
        "file_size": 0,
        "function_count": 0,
        "class_count": 0,
    }

    # Import timing
    start = time.perf_counter()
    try:
        module = importlib.import_module(module_name)
        result["import_time"] = time.perf_counter() - start
    except ImportError:
        return ProfileResult(**result)

    # Introspection timing
    start = time.perf_counter()
    funcs = list(inspect.getmembers(module, inspect.isfunction))
    classes = list(inspect.getmembers(module, inspect.isclass))
    result["introspect_time"] = time.perf_counter() - start
    result["function_count"] = len(funcs)
    result["class_count"] = len(classes)

    # AST parsing timing (if we have the source)
    try:
        source_file = inspect.getfile(module)
        if source_file.endswith(".py"):
            with open(source_file) as f:
                content = f.read()

            start = time.perf_counter()
            ast.parse(content)
            result["ast_parse_time"] = time.perf_counter() - start
            result["file_size"] = len(content)
    except Exception:
        pass

    return ProfileResult(**result)


def analyze_bottlenecks(module_name: str, console: Console):
    """Deep dive into performance bottlenecks for a specific module."""
    console.print(
        Panel.fit(f"🔍 Bottleneck Analysis: {module_name}", style="bold blue")
    )

    # Test different depths to understand recursion impact
    console.print("\n📊 Depth Impact Analysis:", style="bold cyan")
    depth_table = Table(show_header=True, header_style="bold")
    depth_table.add_column("Depth", justify="center")
    depth_table.add_column("Time (s)", justify="right")
    depth_table.add_column("Modules", justify="right")
    depth_table.add_column("Per Module (ms)", justify="right")

    for depth in [1, 2, 3]:
        result = time_exploration_local(module_name, depth)
        if result.duration > 0:
            total_modules = 1 + result.total_submodules
            per_module = (
                (result.duration / total_modules * 1000) if total_modules > 0 else 0
            )
            depth_table.add_row(
                str(depth),
                f"{result.duration:.4f}",
                str(total_modules),
                f"{per_module:.1f}",
            )

    console.print(depth_table)

    # Compare all approaches
    console.print("\n🔬 Method Comparison:", style="bold cyan")

    # 1. Our Rust version
    rust_result = time_exploration_local(module_name, max_depth=1)

    # 2. Published Python version
    pub_time = time_exploration_published(module_name, max_depth=1)

    # 3. Python native capabilities
    py_profile = profile_python_native(module_name)

    comparison_table = Table(show_header=True, header_style="bold")
    comparison_table.add_column("Method", style="cyan")
    comparison_table.add_column("Time (s)", justify="right")
    comparison_table.add_column("Relative", justify="right")
    comparison_table.add_column("Notes")

    # Calculate relative times
    fastest = min(
        rust_result.duration if rust_result.duration > 0 else float("inf"),
        pub_time if pub_time > 0 else float("inf"),
        py_profile.import_time + py_profile.introspect_time,
    )

    if rust_result.duration > 0:
        comparison_table.add_row(
            "Rust (local)",
            f"{rust_result.duration:.4f}",
            f"{rust_result.duration / fastest:.1f}x",
            f"Found {rust_result.total_functions} functions, {rust_result.total_classes} classes",
        )

    if pub_time > 0:
        comparison_table.add_row(
            "Published version",
            f"{pub_time:.4f}",
            f"{pub_time / fastest:.1f}x",
            "Via uvx (includes startup overhead)",
        )

    py_total = py_profile.import_time + py_profile.introspect_time
    comparison_table.add_row(
        "Python import+introspect",
        f"{py_total:.4f}",
        f"{py_total / fastest:.1f}x",
        f"Import: {py_profile.import_time:.4f}s, Introspect: {py_profile.introspect_time:.4f}s",
    )

    if py_profile.ast_parse_time > 0:
        comparison_table.add_row(
            "Python AST parse only",
            f"{py_profile.ast_parse_time:.4f}",
            f"{py_profile.ast_parse_time / fastest:.1f}x",
            f"File size: {py_profile.file_size:,} bytes",
        )

    console.print(comparison_table)

    # Analysis
    if rust_result.duration > 0 and py_profile.ast_parse_time > 0:
        console.print("\n💡 Analysis:", style="bold yellow")
        rust_vs_ast = rust_result.duration / py_profile.ast_parse_time
        console.print(
            f"  • Rust is {rust_vs_ast:.1f}x slower than Python's AST parsing"
        )
        console.print(
            "  • This suggests the bottleneck may be in rustpython-parser or our processing"
        )


def run_standard_benchmark(console: Console, quick: bool = False):
    """Run the standard benchmark suite."""
    table = Table(show_header=True, header_style="bold blue")
    table.add_column("Module", style="bold", width=15)
    table.add_column("Status", justify="center", width=8)
    table.add_column("Local (Rust)", justify="right", width=12)
    table.add_column("Published", justify="right", width=12)
    table.add_column("Speedup", justify="right", width=8)
    table.add_column("Functions", justify="right", width=9)
    table.add_column("Classes", justify="right", width=7)
    table.add_column("Submodules", justify="right", width=10)

    test_suites = [
        (
            "📚 Standard Library (small)",
            [
                ("json", 3),
                ("collections", 3),
                ("itertools", 2),
            ],
        ),
        (
            "📦 Standard Library (medium)",
            [
                ("urllib", 3),
                ("xml", 3),
                ("email", 3),
            ],
        ),
        (
            "🏗️ Standard Library (complex)",
            [
                ("multiprocessing", 2),
                ("concurrent", 3),
                ("logging", 2),
            ],
        ),
    ]

    if quick:
        test_suites = test_suites[:1]  # Only run small modules

    all_results = []

    for category_name, modules in test_suites:
        console.print(f"\n{category_name}", style="bold cyan")

        for module_name, max_depth in modules:
            # Test local version
            duration_local, funcs, classes, submods, has_warnings = (
                time_exploration_local(module_name, max_depth)
            )

            # Test published version
            duration_published = time_exploration_published(module_name, max_depth)

            if duration_local == 0.0:
                status_text = Text("SKIP", style="red")
                local_text = Text("--", style="dim")
                published_text = Text("--", style="dim")
                speedup_text = Text("--", style="dim")
                func_text = Text("--", style="dim")
                class_text = Text("--", style="dim")
                submod_text = Text("--", style="dim")
            else:
                if has_warnings:
                    status_text = Text("⚠", style="yellow")
                else:
                    status_text = Text("✓", style="green")

                time_style = get_time_style(duration_local)
                local_text = Text(f"{duration_local:.4f}s", style=time_style)

                if duration_published > 0:
                    published_text = Text(
                        f"{duration_published:.4f}s",
                        style=get_time_style(duration_published),
                    )
                    speedup = duration_published / duration_local
                    speedup_text = Text(
                        f"{speedup:.1f}x", style=get_speedup_style(speedup)
                    )
                else:
                    published_text = Text("--", style="dim")
                    speedup_text = Text("--", style="dim")

                func_text = Text(str(funcs), style=get_count_style(funcs, "functions"))
                class_text = Text(
                    str(classes), style=get_count_style(classes, "classes")
                )
                submod_text = Text(
                    str(submods), style=get_count_style(submods, "submodules")
                )

            table.add_row(
                module_name,
                status_text,
                local_text,
                published_text,
                speedup_text,
                func_text,
                class_text,
                submod_text,
            )

            all_results.append(
                (
                    module_name,
                    duration_local,
                    duration_published,
                    funcs + classes + submods,
                )
            )

    console.print(table)

    # Summary
    console.print("\n📊 Summary", style="bold green")

    successful_results = [
        (name, dur_local, dur_pub, count)
        for name, dur_local, dur_pub, count in all_results
        if dur_local > 0.0
    ]

    if successful_results:
        total_time_local = sum(dur_local for _, dur_local, _, _ in successful_results)
        total_items = sum(count for _, _, _, count in successful_results)
        avg_time_local = total_time_local / len(successful_results)

        # Calculate average speedup
        speedups = []
        for _, dur_local, dur_pub, _ in successful_results:
            if dur_pub > 0 and dur_local > 0:
                speedups.append(dur_pub / dur_local)

        avg_speedup = sum(speedups) / len(speedups) if speedups else 0

        summary_table = Table(show_header=False, box=None)
        summary_table.add_column("Metric", style="cyan")
        summary_table.add_column("Value", style="white")

        summary_table.add_row("Total modules tested:", str(len(all_results)))
        summary_table.add_row("Successfully explored:", str(len(successful_results)))
        summary_table.add_row("Total time (local Rust):", f"{total_time_local:.4f}s")
        summary_table.add_row("Average time per module:", f"{avg_time_local:.4f}s")
        summary_table.add_row("Total items found:", str(total_items))
        if avg_speedup > 0:
            summary_table.add_row("Average speedup:", f"{avg_speedup:.1f}x faster")

        console.print(summary_table)


def stress_test_large_packages(console: Console):
    """Test performance on very large packages."""
    console.print(Panel.fit("🔥 Stress Test: Large Packages", style="bold red"))

    large_packages = [
        ("numpy", "numpy", 2),
        ("pandas", "pandas", 2),
        ("django", "django", 2),
        ("scipy", "scipy", 1),
        ("matplotlib", "matplotlib", 2),
    ]

    console.print("\nTesting large third-party packages (if available)...")

    stress_table = Table(show_header=True, header_style="bold")
    stress_table.add_column("Package", style="bold")
    stress_table.add_column("Depth", justify="center")
    stress_table.add_column("Time (s)", justify="right")
    stress_table.add_column("Modules", justify="right")
    stress_table.add_column("Per Module (ms)", justify="right")
    stress_table.add_column("Status")

    for pkg, module, depth in large_packages:
        result = time_exploration_local(module, depth)

        if result.duration > 0:
            total_modules = 1 + result.total_submodules
            per_module = (
                (result.duration / total_modules * 1000) if total_modules > 0 else 0
            )

            stress_table.add_row(
                pkg,
                str(depth),
                f"{result.duration:.3f}",
                str(total_modules),
                f"{per_module:.1f}",
                Text("✓", style="green"),
            )
        else:
            stress_table.add_row(
                pkg, str(depth), "--", "--", "--", Text("Not installed", style="dim")
            )

    console.print(stress_table)


def get_time_style(duration: float) -> str:
    """Get rich style for time based on performance."""
    if duration == 0.0:
        return "red"
    elif duration < 0.01:
        return "green"
    elif duration < 0.1:
        return "cyan"
    elif duration < 1.0:
        return "yellow"
    else:
        return "red"


def get_speedup_style(speedup: float) -> str:
    """Get rich style for speedup."""
    if speedup > 10:
        return "bold green"
    elif speedup > 5:
        return "green"
    elif speedup > 2:
        return "cyan"
    elif speedup > 1.5:
        return "yellow"
    else:
        return "red"


def get_count_style(count: int, category: str) -> str:
    """Get rich style for counts."""
    if category == "submodules":
        if count > 20:
            return "red"
        elif count > 10:
            return "yellow"
        else:
            return "green"
    else:  # functions/classes
        if count > 50:
            return "red"
        elif count > 10:
            return "yellow"
        else:
            return "cyan"


def main():
    """Main entry point."""
    console = Console()

    # Parse command line arguments
    if len(sys.argv) > 1:
        if sys.argv[1] == "--help":
            console.print("Usage: perf_test.py [OPTIONS]")
            console.print("\nOptions:")
            console.print(
                "  --bottleneck MODULE  Analyze bottlenecks for specific module"
            )
            console.print("  --stress            Run stress tests on large packages")
            console.print("  --quick             Run quick benchmark (3 modules only)")
            console.print("  --help              Show this help")
            return

        elif sys.argv[1] == "--bottleneck" and len(sys.argv) > 2:
            analyze_bottlenecks(sys.argv[2], console)
            return

        elif sys.argv[1] == "--stress":
            stress_test_large_packages(console)
            return

        elif sys.argv[1] == "--quick":
            console.print(
                Panel.fit(
                    "🚀 Pretty-mod Performance Test (Quick Mode)", style="bold magenta"
                )
            )
            run_standard_benchmark(console, quick=True)
            return

    # Default: run standard benchmark
    console.print(Panel.fit("🚀 Pretty-mod Performance Analysis", style="bold magenta"))
    run_standard_benchmark(console)

    # Also show specific bottleneck analysis
    console.print("\n")
    analyze_bottlenecks("email._header_value_parser", console)


if __name__ == "__main__":
    main()
