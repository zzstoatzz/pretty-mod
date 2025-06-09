#!/usr/bin/env -S uv run --with-editable . --script --quiet
# /// script
# requires-python = ">=3.12"
# dependencies = ["rich>=13.0.0", "prefect", "requests", "pandas", "numpy", "fastapi"]
# ///
"""
Performance testing script for pretty-mod module exploration.
Tests exploration speed on various libraries of different sizes.
Compares local (Rust) version with published (Python) version.

Usage:
    ./scripts/perf_test.py
    uv run scripts/perf_test.py
"""

import subprocess
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
    "Result",
    [
        "duration",
        "total_functions",
        "total_classes",
        "total_submodules",
        "has_warnings",
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
    # Import here to avoid affecting timing of other modules
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
    # Build the command with the appropriate --with flag for the module
    cmd = ["uvx"]

    # Add --with flags for the modules we're testing
    if module_name in ["prefect", "requests", "pandas", "numpy", "fastapi"]:
        cmd.extend(["--with", module_name])

    cmd.extend(["pretty-mod", "tree", module_name, "--depth", str(max_depth)])

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
    """Test exploration performance on various libraries."""
    console = Console()

    console.print(
        Panel.fit(
            "🚀 Pretty-mod Performance Test (Local vs Published)", style="bold magenta"
        )
    )

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
            "🚀 Third-party Libraries",
            [
                ("prefect", 3),
                ("requests", 3),
                ("pandas", 2),
                ("numpy", 2),
                ("fastapi", 3),
            ],
        ),
    ]

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
                    if speedup > 10:
                        speedup_style = "bold green"
                    elif speedup > 5:
                        speedup_style = "green"
                    elif speedup > 2:
                        speedup_style = "cyan"
                    elif speedup > 1.5:
                        speedup_style = "yellow"
                    else:
                        speedup_style = "red"
                    speedup_text = Text(f"{speedup:.1f}x", style=speedup_style)
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
                (module_name, duration_local, duration_published, has_warnings)
            )

    console.print(table)

    console.print("\n📊 Summary", style="bold green")

    successful_results = [
        (name, dur_local, dur_pub)
        for name, dur_local, dur_pub, _ in all_results
        if dur_local > 0.0
    ]
    if successful_results:
        total_time_local = sum(dur_local for _, dur_local, _ in successful_results)
        avg_time_local = total_time_local / len(successful_results)

        # Calculate average speedup for modules where we have both measurements
        speedups = []
        for _, dur_local, dur_pub in successful_results:
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
        if avg_speedup > 0:
            summary_table.add_row("Average speedup:", f"{avg_speedup:.1f}x faster")

        console.print(summary_table)

    console.print("\n💡 Legend", style="bold yellow")
    legend_items = [
        "✓ = Success",
        "⚠ = Warnings during import",
        "SKIP = Module not installed",
        "Time colors: [green]<10ms[/] [cyan]<100ms[/] [yellow]<1s[/] [red]≥1s[/]",
        "Speedup colors: [bold green]>10x[/] [green]>5x[/] [cyan]>2x[/] [yellow]>1.5x[/] [red]≤1.5x[/]",
    ]
    for item in legend_items:
        console.print(f"  {item}")

    console.print("\n🔍 [bold]Note:[/] Published times include uvx startup overhead")


if __name__ == "__main__":
    main()
