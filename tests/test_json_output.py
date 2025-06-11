"""Test JSON output format functionality."""

import json
import os
import subprocess
import sys


def test_tree_json_output():
    """Test tree command with JSON output."""
    result = subprocess.run(
        [sys.executable, "-m", "pretty_mod", "tree", "json", "-o", "json"],
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0

    # Parse JSON output
    data = json.loads(result.stdout)

    # Check structure
    assert "module" in data
    assert data["module"] == "json"
    assert "tree" in data
    assert "api" in data["tree"]
    assert "submodules" in data["tree"]

    # Check some expected content
    api = data["tree"]["api"]
    assert "dump" in api["functions"]
    assert "loads" in api["functions"]
    assert "JSONEncoder" in api["all"]


def test_signature_json_output():
    """Test signature command with JSON output."""
    result = subprocess.run(
        [sys.executable, "-m", "pretty_mod", "sig", "json:dumps", "-o", "json"],
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0

    # Parse JSON output
    data = json.loads(result.stdout)

    # Check structure
    assert "name" in data
    assert data["name"] == "dumps"
    assert "parameters" in data
    assert "obj" in data["parameters"]
    assert "skipkeys=False" in data["parameters"]
    assert "return_type" in data


def test_signature_not_available_json():
    """Test signature not available in JSON format."""
    result = subprocess.run(
        [sys.executable, "-m", "pretty_mod", "sig", "sys:maxsize", "-o", "json"],
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0

    # Parse JSON output
    data = json.loads(result.stdout)

    # Check structure
    assert "name" in data
    assert data["name"] == "maxsize"
    assert "available" in data
    assert data["available"] is False
    assert "reason" in data


def test_default_output_unchanged():
    """Test that default output (without -o flag) remains unchanged."""
    # Test tree
    result_tree = subprocess.run(
        [sys.executable, "-m", "pretty_mod", "tree", "json"],
        capture_output=True,
        text=True,
        env={**os.environ, "PRETTY_MOD_NO_COLOR": "1"},
    )

    assert result_tree.returncode == 0
    assert "📦 json" in result_tree.stdout
    assert "├── ⚡ functions:" in result_tree.stdout

    # Test signature
    result_sig = subprocess.run(
        [sys.executable, "-m", "pretty_mod", "sig", "json:dumps"],
        capture_output=True,
        text=True,
        env={**os.environ, "PRETTY_MOD_NO_COLOR": "1"},
    )

    assert result_sig.returncode == 0
    assert "📎 dumps" in result_sig.stdout
    assert "├──  Parameters:" in result_sig.stdout
