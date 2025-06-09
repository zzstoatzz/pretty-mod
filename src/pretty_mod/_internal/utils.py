import importlib
from typing import Any


def import_object(import_path: str) -> Any:
    """
    Import an object from a module using an import path.

    Args:
        import_path: Import path like 'module.submodule:object' or 'module.object'

    Returns:
        The imported object
    """
    if ":" in import_path:
        module_path, obj_name = import_path.split(":", 1)
        module = importlib.import_module(module_path)
        return getattr(module, obj_name)
    else:
        parts = import_path.split(".")
        for i in range(len(parts), 0, -1):
            try:
                module_path = ".".join(parts[:i])
                obj_path = parts[i:]
                module = importlib.import_module(module_path)
                obj = module
                for attr in obj_path:
                    obj = getattr(obj, attr)
                return obj
            except (ImportError, AttributeError):
                continue
        raise ImportError(f"Cannot import {import_path}")
