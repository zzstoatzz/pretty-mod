"""
Command-line interface for pretty-mod.
"""

import argparse

from ._pretty_mod import display_signature, display_tree


def main():
    """CLI entry point."""
    import sys

    parser = argparse.ArgumentParser(description="Module tree exploration CLI")
    subparsers = parser.add_subparsers(dest="command", help="Commands")

    tree_parser = subparsers.add_parser("tree", help="Display module tree structure")
    tree_parser.add_argument(
        "module", help="Root module path (e.g., 'json', 'prefect.client')"
    )
    tree_parser.add_argument(
        "--depth", type=int, default=2, help="Maximum depth to explore (default: 2)"
    )

    sig_parser = subparsers.add_parser("sig", help="Display function signature")
    sig_parser.add_argument(
        "import_path", help="Import path to the function (e.g., 'json:loads')"
    )

    args = parser.parse_args()

    try:
        if args.command == "tree":
            display_tree(args.module, args.depth)
        elif args.command == "sig":
            print(display_signature(args.import_path))
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
