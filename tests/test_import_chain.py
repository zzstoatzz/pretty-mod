"""Test import chain resolution for common patterns like prefect:flow and fastapi:FastAPI."""

from pretty_mod import display_signature


class TestImportChainResolution:
    """Test that import chain resolution works for known patterns."""

    def test_prefect_flow_resolution(self):
        """Test that prefect:flow resolves to the FlowDecorator.__call__ signature."""
        result = display_signature("prefect:flow", quiet=True)

        # Should not be "signature not available"
        assert "signature not available" not in result

        # Should show the __call__ method signature
        assert "__call__" in result
        assert "Parameters:" in result

        # Should have common flow parameters
        assert "name:" in result
        assert "description:" in result
        assert "retries:" in result

    def test_fastapi_fastapi_resolution(self):
        """Test that fastapi:FastAPI resolves to the FastAPI.__init__ signature."""
        result = display_signature("fastapi:FastAPI", quiet=True)

        # Should not be "signature not available"
        assert "signature not available" not in result

        # Should show the FastAPI constructor signature
        assert "FastAPI" in result
        assert "Parameters:" in result

        # Should have common FastAPI parameters
        assert "title:" in result
        assert "debug:" in result
        assert "version:" in result

    def test_unknown_pattern_fallback(self):
        """Test that unknown patterns still try regular resolution."""
        # This should fall back to regular resolution and fail gracefully
        result = display_signature("unknown:pattern", quiet=True)

        # Should show "signature not available" for unknown patterns
        assert "signature not available" in result
        assert "pattern" in result

    def test_direct_vs_import_chain_consistency(self):
        """Test that direct access and import chain resolution give same results."""
        # Compare prefect:flow with direct access to prefect.flows:FlowDecorator
        chain_result = display_signature("prefect:flow", quiet=True)
        direct_result = display_signature("prefect.flows:FlowDecorator", quiet=True)

        # Both should succeed and have the same signature content
        assert "signature not available" not in chain_result
        assert "signature not available" not in direct_result

        # Should both show __call__ method
        assert "__call__" in chain_result
        assert "__call__" in direct_result

        # Should have the same parameter structure
        assert "name:" in chain_result and "name:" in direct_result
        assert "retries:" in chain_result and "retries:" in direct_result
