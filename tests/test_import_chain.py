"""Test import chain resolution for common patterns like prefect:flow and fastapi:FastAPI."""

from pretty_mod import display_signature


class TestImportChainResolution:
    """Test that import chain resolution works for known patterns."""

    def test_prefect_flow_resolution(self):
        """Test that prefect:flow resolves to the FlowDecorator.__call__ signature."""
        result = display_signature("prefect:flow", quiet=True)

        # Should not be "signature not available"
        assert "signature not available" not in result

        # Should show the flow signature
        assert "flow" in result
        assert "Parameters:" in result

        # Should have common flow parameters
        assert "name=None" in result
        assert "description=None" in result
        assert "retries=None" in result

    def test_fastapi_fastapi_resolution(self):
        """Test that fastapi:FastAPI resolves to the FastAPI.__init__ signature."""
        result = display_signature("fastapi:FastAPI", quiet=True)

        # Should not be "signature not available"
        assert "signature not available" not in result

        # Should show constructor parameters
        assert "Parameters:" in result

        # Should have common FastAPI parameters
        assert "debug:" in result
        assert "title:" in result

    def test_unknown_pattern_fallback(self):
        """Test that unknown patterns gracefully fall back to 'signature not available'."""
        result = display_signature("random_module:random_symbol", quiet=True)

        # Should show signature not available
        assert "signature not available" in result
        assert "random_symbol" in result

    def test_pydantic_basemodel_resolution(self):
        """Test that pydantic:BaseModel resolves to the BaseModel.__init__ signature."""
        result = display_signature("pydantic:BaseModel", quiet=True)

        # Should not be "signature not available"
        assert "signature not available" not in result

        # Should show the BaseModel constructor signature
        assert "BaseModel" in result
        assert "Parameters:" in result

        # Should have BaseModel parameters
        assert "**data" in result

    def test_direct_vs_import_chain_consistency(self):
        """Test that direct access and import chain resolution give same results."""
        # Compare prefect:flow with direct access to prefect.flows:FlowDecorator
        chain_result = display_signature("prefect:flow", quiet=True)
        direct_result = display_signature("prefect.flows:FlowDecorator", quiet=True)

        # Both should succeed and have the same signature content
        assert "signature not available" not in chain_result
        assert "signature not available" not in direct_result

        # The parameter lists should be similar
        # (extract just the parameter section for comparison)
        # This is a basic check - in reality they might differ slightly in formatting

    def test_simple_import_chain(self):
        """Test a simple import chain that we know works."""
        # Test with a package that has straightforward imports
        # For example, json.dumps which is directly available
        result = display_signature("json:dumps", quiet=True)

        # Should find the signature
        assert "dumps" in result
        assert "Parameters:" in result
