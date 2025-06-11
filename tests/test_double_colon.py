from pretty_mod import display_tree


class TestDoubleColonSyntax:
    def test_double_colon_tree(self):
        """Test that package::module syntax works for tree command."""
        # This should download pillow and explore PIL module
        # Just test that it doesn't raise an exception
        display_tree("pillow::PIL", max_depth=0, quiet=True)

    def test_double_colon_with_version(self):
        """Test that package::module@version syntax works."""
        # This should download a specific version of pillow
        display_tree("pillow::PIL@10.0.0", max_depth=0, quiet=True)

    def test_regular_syntax_still_works(self):
        """Test that regular module syntax still works."""
        # Regular module exploration should work as before
        display_tree("json", max_depth=0, quiet=True)
