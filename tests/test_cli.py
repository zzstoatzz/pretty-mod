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
