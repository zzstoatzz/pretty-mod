"""
Command-line interface for pretty-mod.
"""

import argparse
import sys

from ._pretty_mod import display_signature, display_tree, download_package


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

    args = parser.parse_args()

    try:
        if args.command == "tree":
            # First try to display the tree normally
            try:
                display_tree(args.module, args.depth)
            except Exception as e:
                # If module not found, try downloading it
                if "No module named" in str(e) or "ModuleNotFoundError" in str(
                    e.__class__.__name__
                ):
                    if not args.quiet:
                        print(
                            f"Module '{args.module}' not found locally. Attempting to download from PyPI...",
                            file=sys.stderr,
                        )
                    try:
                        # Download the package
                        downloaded = download_package(args.module)

                        # Extract the base package name
                        base_name = (
                            args.module.split("[")[0]
                            .split(">")[0]
                            .split("<")[0]
                            .split("=")[0]
                            .split("!")[0]
                            .strip()
                        )

                        # Add the appropriate directory to sys.path
                        # If the downloaded path ends with the package name, use its parent
                        # Otherwise, use the downloaded path itself
                        path_str = str(downloaded.path)
                        if path_str.endswith(base_name) or path_str.endswith(
                            base_name.replace("-", "_")
                        ):
                            parent_dir = str(downloaded.path.parent)
                        else:
                            parent_dir = path_str

                        sys.path.insert(0, parent_dir)

                        try:
                            # Try again with the downloaded package
                            display_tree(base_name, args.depth)
                        finally:
                            # Always clean up sys.path
                            if parent_dir in sys.path:
                                sys.path.remove(parent_dir)
                    except Exception as download_error:
                        print(
                            f"Error downloading package: {download_error}",
                            file=sys.stderr,
                        )
                        sys.exit(1)
                else:
                    raise
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
