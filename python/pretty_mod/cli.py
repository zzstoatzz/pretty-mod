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
    tree_parser.add_argument(
        "-o",
        "--output",
        type=str,
        choices=["pretty", "json"],
        default="pretty",
        help="Output format (default: pretty)",
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
    sig_parser.add_argument(
        "-o",
        "--output",
        type=str,
        choices=["pretty", "json"],
        default="pretty",
        help="Output format (default: pretty)",
    )

    args = parser.parse_args()

    try:
        if args.command == "tree":
            # Call display_tree with format parameter
            display_tree(args.module, args.depth, args.quiet, args.output)
        elif args.command == "sig":
            # Call display_signature with format parameter
            result = display_signature(args.import_path, args.quiet, args.output)
            print(result)
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
