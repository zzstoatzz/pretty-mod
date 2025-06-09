#!/usr/bin/env -S uv run --with-editable . --script --quiet
# /// script
# requires-python = ">=3.12"
# dependencies = ["rich>=13.0.0", "prefect", "requests", "pandas", "numpy", "fastapi"]
# ///
"""
Performance testing script for pretty-mod module exploration.
Tests exploration speed on various libraries of different sizes.

Usage:
    ./scripts/perf_test.py
    uv run scripts/perf_test.py
"""

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


def time_exploration(module_name: str, max_depth: int = 2) -> ExplorationResult:
    """Time how long it takes to explore a module tree."""
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

        api = tree.get("api", {})
        submodules = tree.get("submodules", {})

        end = time.perf_counter()
        duration = end - start

        total_functions = len(api.get("functions", []))
        total_classes = len(api.get("classes", []))
        total_submodules = len(submodules)

        return ExplorationResult(
            duration, total_functions, total_classes, total_submodules, has_warnings
        )

    except ImportError:
        return ExplorationResult(0.0, 0, 0, 0, False)


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

    console.print(Panel.fit("🚀 Pretty-mod Performance Test", style="bold magenta"))

    table = Table(show_header=True, header_style="bold blue")
    table.add_column("Module", style="bold", width=15)
    table.add_column("Status", justify="center", width=8)
    table.add_column("Time", justify="right", width=8)
    table.add_column("Functions", justify="right", width=9)
    table.add_column("Classes", justify="right", width=7)
    table.add_column("Submodules", justify="right", width=10)

    test_suites = [
        (
            "📚 Standard Library (small)",
            [
                ("json", 2),
                ("collections", 2),
                ("itertools", 2),
            ],
        ),
        (
            "📦 Standard Library (medium)",
            [
                ("urllib", 2),
                ("xml", 2),
                ("email", 2),
            ],
        ),
        (
            "🚀 Third-party Libraries",
            [
                ("prefect", 2),
                ("requests", 2),
                ("pandas", 1),
                ("numpy", 1),
                ("fastapi", 2),
            ],
        ),
    ]

    all_results = []

    for category_name, modules in test_suites:
        console.print(f"\n{category_name}", style="bold cyan")

        for module_name, max_depth in modules:
            duration, funcs, classes, submods, has_warnings = time_exploration(
                module_name, max_depth
            )

            if duration == 0.0:
                status_text = Text("SKIP", style="red")
                time_text = Text("--", style="dim")
                func_text = Text("--", style="dim")
                class_text = Text("--", style="dim")
                submod_text = Text("--", style="dim")
            else:
                if has_warnings:
                    status_text = Text("⚠", style="yellow")
                else:
                    status_text = Text("✓", style="green")

                time_style = get_time_style(duration)
                time_text = Text(f"{duration:.4f}s", style=time_style)
                func_text = Text(str(funcs), style=get_count_style(funcs, "functions"))
                class_text = Text(
                    str(classes), style=get_count_style(classes, "classes")
                )
                submod_text = Text(
                    str(submods), style=get_count_style(submods, "submodules")
                )

            table.add_row(
                module_name, status_text, time_text, func_text, class_text, submod_text
            )

            all_results.append((module_name, duration, has_warnings))

    console.print(table)

    console.print("\n📊 Summary", style="bold green")

    successful_results = [(name, dur) for name, dur, _ in all_results if dur > 0.0]
    if successful_results:
        total_time = sum(dur for _, dur in successful_results)
        avg_time = total_time / len(successful_results)
        fastest = min(successful_results, key=lambda x: x[1])
        slowest = max(successful_results, key=lambda x: x[1])

        summary_table = Table(show_header=False, box=None)
        summary_table.add_column("Metric", style="cyan")
        summary_table.add_column("Value", style="white")

        summary_table.add_row("Total modules tested:", str(len(all_results)))
        summary_table.add_row("Successfully explored:", str(len(successful_results)))
        summary_table.add_row("Total exploration time:", f"{total_time:.4f}s")
        summary_table.add_row("Average time per module:", f"{avg_time:.4f}s")
        summary_table.add_row("Fastest:", f"{fastest[0]} ({fastest[1]:.4f}s)")
        summary_table.add_row("Slowest:", f"{slowest[0]} ({slowest[1]:.4f}s)")

        console.print(summary_table)

    console.print("\n💡 Legend", style="bold yellow")
    legend_items = [
        "✓ = Success",
        "⚠ = Warnings during import",
        "SKIP = Module not installed",
        "Time colors: [green]<10ms[/] [cyan]<100ms[/] [yellow]<1s[/] [red]≥1s[/]",
    ]
    for item in legend_items:
        console.print(f"  {item}")

    console.print(
        "\n🔍 [bold]Tip:[/] Use [cyan]uv run pyinstrument scripts/perf_test.py[/] for detailed profiling"
    )


if __name__ == "__main__":
    main()
