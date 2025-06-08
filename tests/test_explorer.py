import sys
from types import ModuleType
from unittest.mock import Mock, patch

import pytest

from pretty_mod.explorer import (
    ModuleTreeExplorer,
    display_signature,
    import_object,
)


class TestModuleTreeExplorer:
    def test_init(self):
        explorer = ModuleTreeExplorer("test.module", max_depth=3)
        assert explorer.root_module_path == "test.module"
        assert explorer.max_depth == 3
        assert explorer.tree == {}

    def test_init_defaults(self):
        explorer = ModuleTreeExplorer("test.module")
        assert explorer.max_depth == 2

    @patch("pretty_mod.explorer.importlib.import_module")
    def test_import_module_success(self, mock_import):
        mock_module = Mock(spec=ModuleType)
        mock_import.return_value = mock_module

        explorer = ModuleTreeExplorer("test")
        result = explorer._import_module("test.module")

        assert result == mock_module
        mock_import.assert_called_once_with("test.module")

    @patch("pretty_mod.explorer.importlib.import_module")
    def test_import_module_failure(self, mock_import, capsys):
        mock_import.side_effect = ImportError("Module not found")

        explorer = ModuleTreeExplorer("test")
        result = explorer._import_module("nonexistent.module")

        assert result is None
        captured = capsys.readouterr()
        assert "Warning: Could not import nonexistent.module" in captured.out

    def test_is_defined_in_module(self):
        explorer = ModuleTreeExplorer("test")

        # Test with a function that has __module__
        class TestClass:
            pass

        TestClass.__module__ = "test.module"

        assert explorer._is_defined_in_module(TestClass, "test.module") is True
        assert explorer._is_defined_in_module(TestClass, "other.module") is False

    def test_is_defined_in_module_no_module_attr(self):
        explorer = ModuleTreeExplorer("test")
        obj = object()  # object() doesn't have __module__

        assert explorer._is_defined_in_module(obj, "test.module") is False

    def test_get_module_public_api_with_all(self):
        explorer = ModuleTreeExplorer("test")

        # Create a mock module with __all__
        mock_module = Mock(spec=ModuleType)
        mock_module.__name__ = "test.module"

        # Create mock items
        def mock_func():
            pass

        mock_func.__module__ = "test.module"

        class MockClass:
            pass

        MockClass.__module__ = "test.module"

        # Set up the module with __all__ and the actual attributes
        mock_module.__all__ = ["func1", "Class1"]
        mock_module.func1 = mock_func
        mock_module.Class1 = MockClass

        result = explorer._get_module_public_api(mock_module)

        # Check that __all__ items are captured
        assert "func1" in result["all"]
        assert "Class1" in result["all"]
        # Check that items are properly categorized
        assert "func1" in result["functions"]
        assert "Class1" in result["classes"]

    def test_get_module_public_api_without_all(self):
        explorer = ModuleTreeExplorer("test")

        # Create a real mock module without __all__
        mock_module = Mock(spec=ModuleType)
        mock_module.__name__ = "test.module"
        delattr(mock_module, "__all__")  # Ensure no __all__ attribute

        def mock_func():
            pass

        mock_func.__module__ = "test.module"

        # Set up the module attributes directly
        mock_module.public_func = mock_func
        mock_module._private_func = lambda: None
        mock_module.__dunder__ = "value"

        # Mock dir to return our test attributes
        with patch("builtins.dir") as mock_dir:
            mock_dir.return_value = ["public_func", "_private_func", "__dunder__"]
            result = explorer._get_module_public_api(mock_module)

        # The function should be categorized properly
        assert isinstance(result, dict)
        assert "all" in result
        assert "functions" in result
        assert result["all"] == []  # No __all__, so this should be empty


class TestImportObject:
    def test_import_with_colon_syntax(self):
        # Test importing sys.version_info using colon syntax
        result = import_object("sys:version_info")
        assert result == sys.version_info

    def test_import_with_dot_syntax(self):
        # Test importing sys.version_info using dot syntax
        result = import_object("sys.version_info")
        assert result == sys.version_info

    def test_import_module(self):
        # Test importing entire module
        result = import_object("sys")
        assert result == sys

    def test_import_nonexistent(self):
        with pytest.raises(ImportError):
            import_object("nonexistent.module")

    def test_import_nonexistent_attribute(self):
        with pytest.raises(ImportError):
            import_object("sys.nonexistent_attribute")


class TestDisplaySignature:
    def test_display_signature_simple_function(self):
        # Test with a built-in function that can be imported
        result = display_signature("builtins:len")
        assert "📎 len" in result
        assert "Parameters:" in result

    def test_display_signature_with_module_colon_syntax(self):
        # Test with sys:exit
        result = display_signature("sys:exit")
        assert "📎 exit" in result
        assert "Parameters:" in result

    def test_display_signature_non_callable(self):
        # Test with non-callable object
        result = display_signature("sys:version_info")
        assert "Error:" in result
        assert "not callable" in result

    def test_display_signature_nonexistent(self):
        # Test with nonexistent import
        result = display_signature("nonexistent.function")
        assert "Error:" in result
        assert "Could not import" in result


class TestIntegration:
    def test_explore_builtin_module(self):
        # Test exploring a small built-in module
        explorer = ModuleTreeExplorer("json", max_depth=1)
        tree = explorer.explore()

        assert isinstance(tree, dict)
        assert "api" in tree
        assert "submodules" in tree

    def test_get_tree_string(self):
        explorer = ModuleTreeExplorer("json", max_depth=1)

        tree_string = explorer.get_tree_string()
        assert "📦 json" in tree_string
        assert isinstance(tree_string, str)
        assert len(tree_string) > 0


# Edge case and error handling tests
class TestErrorHandling:
    @patch("pretty_mod.explorer.importlib.import_module")
    def test_explore_with_import_failure(self, mock_import, capsys):
        mock_import.return_value = None

        explorer = ModuleTreeExplorer("nonexistent")
        result = explorer.explore()

        assert result == {}

    def test_tree_string_with_empty_tree(self):
        explorer = ModuleTreeExplorer("test")
        # Don't call explore(), so tree remains empty

        result = explorer.get_tree_string()
        assert "📦 test" in result
