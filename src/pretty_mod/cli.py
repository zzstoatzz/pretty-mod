"""
Command-line interface for pretty-mod.
"""

from ._internal.cli_impl import _main_impl


def main() -> None:
    """Main CLI entry point."""
    _main_impl()


if __name__ == "__main__":
    main()
