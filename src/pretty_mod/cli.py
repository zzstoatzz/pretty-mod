"""
Command-line interface for pretty-mod.
"""

import argparse
import sys

from .explorer import ModuleTreeExplorer, display_signature


def tree_command(args) -> None:
    """Handle the tree subcommand."""
    explorer = ModuleTreeExplorer(args.module, max_depth=args.depth)
    print(explorer.get_tree_string())


def sig_command(args) -> None:
    """Handle the sig subcommand."""
    result = display_signature(args.import_path)
    print(result)


def create_parser() -> argparse.ArgumentParser:
    """Create the argument parser."""
    parser = argparse.ArgumentParser(
        prog="pretty-mod",
        description="A module tree explorer for humans and LLMs",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  pretty-mod tree json                       # Explore json module
  pretty-mod tree requests --depth 3        # Explore with custom depth  
  pretty-mod sig json:loads                  # Show function signature
  pretty-mod sig os.path:join                # Show function signature
        """.strip(),
    )

    subparsers = parser.add_subparsers(
        dest="command",
        help="Available commands",
        metavar="COMMAND",
    )

    # Tree subcommand
    tree_parser = subparsers.add_parser(
        "tree",
        help="Explore module structure",
        description="Explore the structure of a Python module and display it as a tree.",
    )
    tree_parser.add_argument(
        "module",
        help="Module to explore (e.g., 'json', 'os.path', 'requests')",
    )
    tree_parser.add_argument(
        "--depth",
        type=int,
        default=2,
        help="Maximum depth to explore (default: 2)",
    )
    tree_parser.set_defaults(func=tree_command)

    # Sig subcommand
    sig_parser = subparsers.add_parser(
        "sig",
        help="Display function signature",
        description="Display the signature of a function in a readable format.",
    )
    sig_parser.add_argument(
        "import_path",
        help="Import path to function (e.g., 'json:loads', 'os.path:join')",
    )
    sig_parser.set_defaults(func=sig_command)

    return parser


def main() -> None:
    """Main CLI entry point."""
    parser = create_parser()
    args = parser.parse_args()

    if not hasattr(args, "func"):
        parser.print_help()
        sys.exit(1)

    try:
        args.func(args)
        sys.exit(0)
    except KeyboardInterrupt:
        print("\nInterrupted", file=sys.stderr)
        sys.exit(130)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
