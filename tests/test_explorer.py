import sys

import pytest
from pretty_mod.explorer import ModuleTreeExplorer, display_signature, import_object


class TestModuleTreeExplorer:
    def test_init(self):
        explorer = ModuleTreeExplorer("test.module", max_depth=3)
        assert explorer.root_module_path == "test.module"
        assert explorer.max_depth == 3
        assert explorer.tree == {}

    def test_init_defaults(self):
        explorer = ModuleTreeExplorer("test.module")
        assert explorer.max_depth == 2


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

    def test_get_tree_string_auto_explores(self):
        """Test that get_tree_string() automatically calls explore() if needed."""
        explorer = ModuleTreeExplorer("json", max_depth=1)

        # Should auto-explore when get_tree_string is called
        result = explorer.get_tree_string()

        # Verify it worked
        assert "📦 json" in result
        assert explorer.tree  # Tree should now be populated
