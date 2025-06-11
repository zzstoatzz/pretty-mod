import sys
from unittest.mock import patch

import pytest
from pretty_mod import display_signature, display_tree
from pretty_mod.cli import main


class TestCLIDisplayFunctions:
    def test_display_tree(self):
        # Just test that it doesn't raise an exception
        # We can't capture Rust's println! output easily in Python tests
        display_tree("json", 1)

    def test_display_signature(self):
        result = display_signature("builtins:len")
        assert "📎 len" in result

    def test_display_signature_error(self):
        result = display_signature("nonexistent:function")
        assert "Error:" in result
        assert "ModuleNotFoundError" in result

    def test_display_signature_auto_download(self):
        # Test that sig can auto-download packages
        result = display_signature("six:print_", quiet=True)
        assert "📎 print" in result  # The function name is normalized to 'print'
        # print_ is a function in six that maps to print


class TestCLIMain:
    def test_main_with_help(self):
        with patch.object(sys, "argv", ["pretty-mod", "--help"]):
            with pytest.raises(SystemExit) as exc_info:
                main()
            assert exc_info.value.code == 0  # type: ignore[attr-defined]

    def test_main_no_args(self):
        with patch.object(sys, "argv", ["pretty-mod"]):
            with pytest.raises(SystemExit) as exc_info:
                main()
            assert exc_info.value.code == 1  # type: ignore[attr-defined]

    def test_main_tree(self):
        # Just test that it completes successfully without raising
        # We can't capture Rust's println! output via capsys
        with patch.object(sys, "argv", ["pretty-mod", "tree", "json", "--depth", "1"]):
            main()

    def test_main_sig(self, capsys):
        with patch.object(sys, "argv", ["pretty-mod", "sig", "builtins:len"]):
            main()  # Should complete successfully without raising

        captured = capsys.readouterr()
        assert "📎 len" in captured.out


class TestPackageDownload:
    def test_auto_download_functionality(self):
        """Test that packages are automatically downloaded when not installed."""
        # Use 'toml' as it's a small, stable package
        # Add quiet=True to avoid stderr messages interfering with the test
        # Just test that it doesn't raise an exception
        display_tree("toml", 1, quiet=True)

    def test_tree_with_colon_syntax_error(self):
        """Test that tree rejects module paths with colons."""
        with pytest.raises(ValueError) as exc_info:
            display_tree("module:object", 1)

        assert "Invalid module path" in str(exc_info.value)
        assert "use 'pretty-mod sig'" in str(exc_info.value)

    def test_auto_download_submodule(self):
        """Test that submodules trigger download of the base package."""
        # Use toml.decoder as it's a submodule of toml
        # Just test that it doesn't raise an exception
        display_tree("toml.decoder", 1, quiet=True)

    def test_download_with_quiet_flag(self, capsys):
        """Test that --quiet suppresses download messages."""
        # Use a package that's unlikely to be installed
        test_package = "tinynetrc"  # Small package unlikely to be pre-installed

        with patch.object(
            sys, "argv", ["pretty-mod", "tree", test_package, "--quiet", "--depth", "1"]
        ):
            try:
                main()
            except SystemExit:
                pass  # It's OK if it exits with an error

        captured = capsys.readouterr()
        # With --quiet, the download message should not appear in stderr
        assert "not found locally" not in captured.err

    def test_download_nonexistent_package(self, capsys):
        """Test handling of non-existent packages."""
        # Try to display a tree for a package that doesn't exist
        with pytest.raises(SystemExit) as exc_info:
            with patch.object(
                sys,
                "argv",
                [
                    "pretty-mod",
                    "tree",
                    "this-package-definitely-does-not-exist-12345",
                    "--depth",
                    "1",
                ],
            ):
                main()

        assert exc_info.value.code == 1  # type: ignore[attr-defined]

        captured = capsys.readouterr()
        assert "Error:" in captured.err

    def test_main_keyboard_interrupt(self):
        with patch.object(sys, "argv", ["pretty-mod", "tree", "json"]):
            with patch("pretty_mod.cli.display_tree") as mock_tree:
                mock_tree.side_effect = KeyboardInterrupt()
                with pytest.raises(SystemExit) as exc_info:
                    main()
                assert exc_info.value.code == 130  # type: ignore[attr-defined]

    def test_main_exception(self):
        with patch.object(sys, "argv", ["pretty-mod", "tree", "json"]):
            with patch("pretty_mod.cli.display_tree") as mock_tree:
                mock_tree.side_effect = RuntimeError("Test error")
                with pytest.raises(SystemExit) as exc_info:
                    main()
                assert exc_info.value.code == 1  # type: ignore[attr-defined]
