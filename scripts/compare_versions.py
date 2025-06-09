#!/usr/bin/env -S uv run --with prefect --script
"""Compare stable vs pre-release pretty-mod performance."""

import subprocess
import time
from statistics import mean, stdev


def benchmark_version(
    version_flag: list[str], module: str, depth: int = 2, runs: int = 20
) -> tuple[float, float]:
    """Benchmark a specific version of pretty-mod."""
    # Warm up
    for _ in range(3):
        subprocess.run(
            ["uvx"]
            + version_flag
            + ["--with", module, "pretty-mod", "tree", module, "--depth", str(depth)],
            capture_output=True,
        )

    # Actual runs
    times = []
    for _ in range(runs):
        start = time.perf_counter()
        subprocess.run(
            ["uvx"]
            + version_flag
            + ["--with", module, "pretty-mod", "tree", module, "--depth", str(depth)],
            capture_output=True,
        )
        times.append(time.perf_counter() - start)

    return mean(times), stdev(times)


def main():
    modules = ["prefect", "numpy", "pandas"]

    for module in modules:
        print(f"\n{'=' * 60}")
        print(f"Module: {module}")
        print("=" * 60)

        try:
            # Stable version
            print("\n📦 STABLE VERSION (latest)")
            stable_avg, stable_std = benchmark_version([], module)
            print(f"   Average: {stable_avg * 1000:.2f}ms ± {stable_std * 1000:.2f}ms")

            # Pre-release version
            print("\n🚀 PRE-RELEASE VERSION")
            pre_avg, pre_std = benchmark_version(["--prerelease=allow"], module)
            print(f"   Average: {pre_avg * 1000:.2f}ms ± {pre_std * 1000:.2f}ms")

            # Speedup
            speedup = stable_avg / pre_avg
            print(
                f"\n📊 RESULT: {speedup:.1f}x {'faster' if speedup > 1 else 'slower'}"
            )
            print(f"   Time saved: {(stable_avg - pre_avg) * 1000:.2f}ms per run")

        except subprocess.CalledProcessError:
            print(f"   Error: Module {module} not available")
        except Exception as e:
            print(f"   Error: {e}")


if __name__ == "__main__":
    main()
