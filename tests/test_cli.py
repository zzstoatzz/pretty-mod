import sys
from unittest.mock import patch

import pytest
from pretty_mod import display_signature, display_tree, download_package
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
    def test_download_package(self):
        """Test downloading a small package from PyPI."""
        # Use 'six' as it's a small, stable package
        downloaded = download_package("six")

        # Check that we got a valid path
        assert downloaded.path

        # Check that the path exists and is a Path object
        from pathlib import Path

        assert isinstance(downloaded.path, Path)
        assert downloaded.path.exists()
        assert downloaded.path.is_dir()

        # The package should contain at least some Python files
        # (either in the directory itself or in a subdirectory)
        has_py_files = any(
            file.suffix == ".py"
            for file in downloaded.path.rglob("*")
            if file.is_file()
        )
        assert has_py_files, "Downloaded package should contain Python files"

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

    def test_download_nonexistent_package(self):
        """Test handling of non-existent packages."""
        with pytest.raises(Exception) as exc_info:
            download_package("this-package-definitely-does-not-exist-12345")

        assert "not found on PyPI" in str(exc_info.value)

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
