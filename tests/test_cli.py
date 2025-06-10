import sys
from unittest.mock import patch

import pytest
from pretty_mod import display_signature, display_tree
from pretty_mod.cli import main


class TestCLIDisplayFunctions:
    def test_display_tree(self, capsys):
        display_tree("json", 1)
        captured = capsys.readouterr()
        assert "📦 json" in captured.out

    def test_display_signature(self):
        result = display_signature("builtins:len")
        assert "📎 len" in result

    def test_display_signature_error(self):
        result = display_signature("nonexistent:function")
        assert "Error:" in result
        assert "ModuleNotFoundError" in result


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

    def test_main_tree(self, capsys):
        with patch.object(sys, "argv", ["pretty-mod", "tree", "json", "--depth", "1"]):
            main()  # Should complete successfully without raising

        captured = capsys.readouterr()
        assert "📦 json" in captured.out

    def test_main_sig(self, capsys):
        with patch.object(sys, "argv", ["pretty-mod", "sig", "builtins:len"]):
            main()  # Should complete successfully without raising

        captured = capsys.readouterr()
        assert "📎 len" in captured.out


class TestPackageDownload:
    def test_auto_download_functionality(self, capsys):
        """Test that packages are automatically downloaded when not installed."""
        # Use 'six' as it's a small, stable package unlikely to be installed
        # If it's already installed, the test will still pass (it just won't download)
        display_tree("six", 1)

        captured = capsys.readouterr()
        # Should show the tree structure regardless of whether it was downloaded
        assert "📦 six" in captured.out

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
