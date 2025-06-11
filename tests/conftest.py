import pytest


@pytest.fixture(autouse=True)
def disable_colors_for_tests(monkeypatch):
    """Disable colors for all tests."""
    monkeypatch.setenv("PRETTY_MOD_NO_COLOR", "1")
