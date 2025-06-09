#!/usr/bin/env -S uv run --with-editable . --script --quiet
# /// script
# requires-python = ">=3.12"
# dependencies = ["pyinstrument"]
# ///
"""Profile pretty-mod to find performance bottlenecks."""

import sys
import time

from pyinstrument import Profiler


def main():
    if len(sys.argv) < 2:
        print("Usage: ./scripts/profile.py MODULE [--depth N]")
        sys.exit(1)

    module_name = sys.argv[1]
    depth = 2

    if len(sys.argv) > 3 and sys.argv[2] == "--depth":
        depth = int(sys.argv[3])

    # Profile the module exploration with more detail
    profiler = Profiler(interval=0.001)  # Higher resolution
    profiler.start()

    # Time individual operations
    from pretty_mod.explorer import ModuleTreeExplorer  # type: ignore

    start_init = time.perf_counter()
    explorer = ModuleTreeExplorer(module_name, max_depth=depth)
    init_time = time.perf_counter() - start_init

    start_explore = time.perf_counter()
    explorer.explore()
    explore_time = time.perf_counter() - start_explore

    profiler.stop()

    print("\nTiming breakdown:")
    print(f"  Explorer init: {init_time * 1000:.2f}ms")
    print(f"  Exploration:   {explore_time * 1000:.2f}ms")
    print(f"  Total:         {(init_time + explore_time) * 1000:.2f}ms")

    print("\nDetailed profile:")
    print(profiler.output_text(unicode=True, color=True, show_all=True))


if __name__ == "__main__":
    main()
