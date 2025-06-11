"""
Command-line interface for pretty-mod.
"""

import argparse
import sys

from ._pretty_mod import display_signature, display_tree


def main():
    """CLI entry point."""

    parser = argparse.ArgumentParser(description="Module tree exploration CLI")
    subparsers = parser.add_subparsers(dest="command", help="Commands")

    tree_parser = subparsers.add_parser("tree", help="Display module tree structure")
    tree_parser.add_argument(
        "module", help="Root module path (e.g., 'json', 'prefect.client')"
    )
    tree_parser.add_argument(
        "--depth", type=int, default=2, help="Maximum depth to explore (default: 2)"
    )
    tree_parser.add_argument(
        "--quiet",
        action="store_true",
        help="Suppress warnings and informational messages",
    )

    sig_parser = subparsers.add_parser("sig", help="Display function signature")
    sig_parser.add_argument(
        "import_path", help="Import path to the function (e.g., 'json:loads')"
    )
    sig_parser.add_argument(
        "--quiet",
        action="store_true",
        help="Suppress download messages",
    )

    args = parser.parse_args()

    try:
        if args.command == "tree":
            display_tree(args.module, args.depth, args.quiet)
        elif args.command == "sig":
            print(display_signature(args.import_path, args.quiet))
        else:
            parser.print_help()
            sys.exit(1)
    except KeyboardInterrupt:
        sys.exit(130)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
