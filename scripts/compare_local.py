#!/usr/bin/env -S uv run --script
"""Compare published vs local pretty-mod performance."""

import subprocess
import time
from statistics import mean, stdev


def benchmark_published(
    module: str, depth: int = 2, runs: int = 20
) -> tuple[float, float]:
    """Benchmark the published version of pretty-mod using uvx."""
    # Warm up
    for _ in range(3):
        subprocess.run(
            ["uvx", "pretty-mod", "tree", module, "--depth", str(depth)],
            capture_output=True,
        )

    # Actual runs
    times = []
    for _ in range(runs):
        start = time.perf_counter()
        result = subprocess.run(
            ["uvx", "pretty-mod", "tree", module, "--depth", str(depth)],
            capture_output=True,
        )
        if result.returncode != 0:
            raise Exception(f"Command failed: {result.stderr.decode()}")
        times.append(time.perf_counter() - start)

    return mean(times), stdev(times)


def benchmark_local(module: str, depth: int = 2, runs: int = 20) -> tuple[float, float]:
    """Benchmark the local version of pretty-mod using uv run."""
    # Warm up
    for _ in range(3):
        subprocess.run(
            ["uv", "run", "pretty-mod", "tree", module, "--depth", str(depth)],
            capture_output=True,
            cwd="/Users/nate/github.com/zzstoatzz/pretty-mod",
        )

    # Actual runs
    times = []
    for _ in range(runs):
        start = time.perf_counter()
        result = subprocess.run(
            ["uv", "run", "pretty-mod", "tree", module, "--depth", str(depth)],
            capture_output=True,
            cwd="/Users/nate/github.com/zzstoatzz/pretty-mod",
        )
        if result.returncode != 0:
            raise Exception(f"Command failed: {result.stderr.decode()}")
        times.append(time.perf_counter() - start)

    return mean(times), stdev(times)


def benchmark_download_case(runs: int = 5) -> tuple[float, float, float, float]:
    """Benchmark the download case with a package not typically installed."""
    test_package = "six"  # Small, stable package

    # Test published version (should fail)
    pub_times = []
    for _ in range(runs):
        start = time.perf_counter()
        subprocess.run(
            ["uvx", "pretty-mod", "tree", test_package, "--depth", "1"],
            capture_output=True,
        )
        pub_times.append(time.perf_counter() - start)

    # Test local version with download
    local_times = []
    for _ in range(runs):
        start = time.perf_counter()
        subprocess.run(
            ["uv", "run", "pretty-mod", "tree", test_package, "--depth", "1"],
            capture_output=True,
            cwd="/Users/nate/github.com/zzstoatzz/pretty-mod",
        )
        local_times.append(time.perf_counter() - start)

    return mean(pub_times), stdev(pub_times), mean(local_times), stdev(local_times)


def main():
    print("🔬 Performance Comparison: Published vs Local")
    print("=" * 60)

    # Test modules that should already be installed
    modules = ["json", "urllib", "os", "sys"]

    print("\n📊 Testing already-installed modules (no download needed):")
    print("-" * 60)

    for module in modules:
        print(f"\nModule: {module}")

        try:
            # Published version
            pub_avg, pub_std = benchmark_published(module, depth=2, runs=10)
            print(f"  Published: {pub_avg * 1000:.2f}ms ± {pub_std * 1000:.2f}ms")

            # Local version
            local_avg, local_std = benchmark_local(module, depth=2, runs=10)
            print(f"  Local:     {local_avg * 1000:.2f}ms ± {local_std * 1000:.2f}ms")

            # Compare
            diff = (local_avg - pub_avg) / pub_avg * 100
            print(f"  Diff:      {diff:+.1f}% {'(slower)' if diff > 0 else '(faster)'}")

        except Exception as e:
            print(f"  Error: {e}")

    print("\n\n📦 Testing download case (package not installed):")
    print("-" * 60)
    print("\nPackage: six")

    try:
        pub_avg, pub_std, local_avg, local_std = benchmark_download_case(runs=3)
        print(
            f"  Published: {pub_avg * 1000:.2f}ms ± {pub_std * 1000:.2f}ms (will fail)"
        )
        print(
            f"  Local:     {local_avg * 1000:.2f}ms ± {local_std * 1000:.2f}ms (with download)"
        )
        print("  Note: Local version downloads and extracts the package")
    except Exception as e:
        print(f"  Error: {e}")

    print("\n\n🔄 Testing repeated download case (caching opportunity):")
    print("-" * 60)

    # Run the download case multiple times to see if there's caching
    print("\nRunning 'pretty-mod tree six' 5 times in a row:")
    times = []
    for i in range(5):
        start = time.perf_counter()
        subprocess.run(
            ["uv", "run", "pretty-mod", "tree", "six", "--depth", "1", "--quiet"],
            capture_output=True,
            cwd="/Users/nate/github.com/zzstoatzz/pretty-mod",
        )
        elapsed = time.perf_counter() - start
        times.append(elapsed)
        print(f"  Run {i + 1}: {elapsed * 1000:.2f}ms")

    print(f"\n  First run:  {times[0] * 1000:.2f}ms")
    print(f"  Subsequent: {mean(times[1:]) * 1000:.2f}ms average")
    print(
        f"  Potential caching opportunity: {(times[0] - mean(times[1:])) * 1000:.2f}ms"
    )


if __name__ == "__main__":
    main()
