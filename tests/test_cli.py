import sys
from unittest.mock import patch

import pytest

from pretty_mod.cli import create_parser, main, sig_command, tree_command


class TestCLIParser:
    def test_parser_creation(self):
        parser = create_parser()
        assert parser.prog == "pretty-mod"
        assert parser.description is not None
        assert "module tree explorer" in parser.description  # type: ignore[operator]

    def test_tree_subcommand(self):
        parser = create_parser()
        args = parser.parse_args(["tree", "json"])
        assert args.command == "tree"
        assert args.module == "json"
        assert args.depth == 2  # default

    def test_tree_with_depth(self):
        parser = create_parser()
        args = parser.parse_args(["tree", "json", "--depth", "3"])
        assert args.command == "tree"
        assert args.module == "json"
        assert args.depth == 3

    def test_sig_subcommand(self):
        parser = create_parser()
        args = parser.parse_args(["sig", "json:loads"])
        assert args.command == "sig"
        assert args.import_path == "json:loads"

    def test_no_subcommand(self):
        parser = create_parser()
        args = parser.parse_args([])
        assert not hasattr(args, "func")


class TestCLICommands:
    def test_tree_command(self, capsys):
        class MockArgs:
            module = "json"
            depth = 1

        tree_command(MockArgs())
        captured = capsys.readouterr()
        assert "📦 json" in captured.out

    def test_sig_command(self, capsys):
        class MockArgs:
            import_path = "builtins:len"

        sig_command(MockArgs())
        captured = capsys.readouterr()
        assert "📎 len" in captured.out

    def test_sig_command_error(self, capsys):
        class MockArgs:
            import_path = "nonexistent:function"

        sig_command(MockArgs())
        captured = capsys.readouterr()
        assert "Error:" in captured.out


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
            with pytest.raises(SystemExit) as exc_info:
                main()
            assert exc_info.value.code == 0  # type: ignore[attr-defined]

        captured = capsys.readouterr()
        assert "📦 json" in captured.out

    def test_main_sig(self, capsys):
        with patch.object(sys, "argv", ["pretty-mod", "sig", "builtins:len"]):
            with pytest.raises(SystemExit) as exc_info:
                main()
            assert exc_info.value.code == 0  # type: ignore[attr-defined]

        captured = capsys.readouterr()
        assert "📎 len" in captured.out

    def test_main_keyboard_interrupt(self):
        with patch.object(sys, "argv", ["pretty-mod", "tree", "json"]):
            with patch("pretty_mod.cli.tree_command") as mock_tree:
                mock_tree.side_effect = KeyboardInterrupt()
                with pytest.raises(SystemExit) as exc_info:
                    main()
                assert exc_info.value.code == 130  # type: ignore[attr-defined]

    def test_main_exception(self):
        with patch.object(sys, "argv", ["pretty-mod", "tree", "json"]):
            with patch("pretty_mod.cli.tree_command") as mock_tree:
                mock_tree.side_effect = RuntimeError("Test error")
                with pytest.raises(SystemExit) as exc_info:
                    main()
                assert exc_info.value.code == 1  # type: ignore[attr-defined]
