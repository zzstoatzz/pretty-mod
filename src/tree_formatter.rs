use pyo3::prelude::*;
use std::collections::HashMap;
use crate::config::{DisplayConfig, colorize};

/// Format tree display for wrapped format (with api/submodules structure)
pub fn format_tree_display(
    py: Python,
    tree: &PyObject,
    module_name: &str,
) -> PyResult<String> {
    let tree_dict: HashMap<String, PyObject> = tree.extract(py)?;
    let config = DisplayConfig::get();

    let mut result = format!("{} {}\n", 
        colorize(&config.module_icon, &config.color_scheme.module_color, config),
        colorize(module_name, &config.color_scheme.module_color, config)
    );

    // Check if there are submodules
    let has_submodules = tree_dict
        .get("submodules")
        .and_then(|s| s.extract::<HashMap<String, PyObject>>(py).ok())
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    // Extract the api dict
    if let Some(api) = tree_dict.get("api") {
        let api_dict: HashMap<String, PyObject> = api.extract(py)?;

        let mut items: Vec<String> = Vec::new();

        // Add __all__ if present
        if let Some(all_exports) = api_dict.get("all") {
            let exports: Vec<String> = all_exports.extract(py)?;
            if !exports.is_empty() {
                items.push(format!("{} __all__: {}", 
                    colorize(&config.exports_icon, &config.color_scheme.exports_color, config),
                    exports.join(", ")
                ));
            }
        }

        // functions
        if let Some(functions) = api_dict.get("functions") {
            let funcs: Vec<String> = functions.extract(py)?;
            if !funcs.is_empty() {
                items.push(format!("{} functions: {}", 
                    colorize(&config.function_icon, &config.color_scheme.function_color, config),
                    funcs.join(", ")
                ));
            }
        }

        // classes
        if let Some(classes) = api_dict.get("classes") {
            let cls: Vec<String> = classes.extract(py)?;
            if !cls.is_empty() {
                items.push(format!("{} classes: {}", 
                    colorize(&config.class_icon, &config.color_scheme.class_color, config),
                    cls.join(", ")
                ));
            }
        }

        // constants
        if let Some(constants) = api_dict.get("constants") {
            let consts: Vec<String> = constants.extract(py)?;
            if !consts.is_empty() {
                items.push(format!("{} constants: {}", 
                    colorize(&config.constant_icon, &config.color_scheme.constant_color, config),
                    consts.join(", ")
                ));
            }
        }

        // Print items
        for (i, item) in items.iter().enumerate() {
            let is_last = i == items.len() - 1 && !has_submodules;
            let prefix = if is_last { &config.tree_last } else { &config.tree_branch };
            result.push_str(&format!("{}{}\n", 
                colorize(prefix, &config.color_scheme.tree_color, config),
                item
            ));
        }
    }

    // submodules
    if let Some(submodules) = tree_dict.get("submodules") {
        let submods: HashMap<String, PyObject> = submodules.extract(py)?;
        let mut submod_names: Vec<_> = submods.keys().cloned().collect();
        submod_names.sort();

        if !submod_names.is_empty() {
            for (i, name) in submod_names.iter().enumerate() {
                let is_last = i == submod_names.len() - 1;
                let prefix = if is_last { &config.tree_last } else { &config.tree_branch };
                result.push_str(&format!("{}{} {}\n", 
                    colorize(prefix, &config.color_scheme.tree_color, config),
                    colorize(&config.module_icon, &config.color_scheme.module_color, config),
                    colorize(name, &config.color_scheme.module_color, config)
                ));

                if let Some(submod_tree) = submods.get(name) {
                    let submod_content = format_tree_recursive(
                        py,
                        submod_tree,
                        if is_last { &config.tree_empty } else { &config.tree_vertical },
                    )?;
                    result.push_str(&submod_content);
                }
            }
        }
    }

    Ok(result)
}

fn format_tree_recursive(py: Python, tree: &PyObject, prefix: &str) -> PyResult<String> {
    let tree_dict: HashMap<String, PyObject> = tree.extract(py)?;
    let config = DisplayConfig::get();

    let mut result = String::new();

    // Extract the api dict
    if let Some(api) = tree_dict.get("api") {
        let api_dict: HashMap<String, PyObject> = api.extract(py)?;

        let mut items: Vec<String> = Vec::new();

        // Add __all__ if present
        if let Some(all_exports) = api_dict.get("all") {
            let exports: Vec<String> = all_exports.extract(py)?;
            if !exports.is_empty() {
                items.push(format!("{} __all__: {}", 
                    colorize(&config.exports_icon, &config.color_scheme.exports_color, config),
                    exports.join(", ")
                ));
            }
        }

        // functions
        if let Some(functions) = api_dict.get("functions") {
            let funcs: Vec<String> = functions.extract(py)?;
            if !funcs.is_empty() {
                items.push(format!("{} functions: {}", 
                    colorize(&config.function_icon, &config.color_scheme.function_color, config),
                    funcs.join(", ")
                ));
            }
        }

        // classes
        if let Some(classes) = api_dict.get("classes") {
            let cls: Vec<String> = classes.extract(py)?;
            if !cls.is_empty() {
                items.push(format!("{} classes: {}", 
                    colorize(&config.class_icon, &config.color_scheme.class_color, config),
                    cls.join(", ")
                ));
            }
        }

        // constants
        if let Some(constants) = api_dict.get("constants") {
            let consts: Vec<String> = constants.extract(py)?;
            if !consts.is_empty() {
                items.push(format!("{} constants: {}", 
                    colorize(&config.constant_icon, &config.color_scheme.constant_color, config),
                    consts.join(", ")
                ));
            }
        }

        // Check if there are submodules
        let has_submodules = tree_dict
            .get("submodules")
            .and_then(|s| s.extract::<HashMap<String, PyObject>>(py).ok())
            .map(|s| !s.is_empty())
            .unwrap_or(false);

        // Print items
        for (i, item) in items.iter().enumerate() {
            let is_last = i == items.len() - 1 && !has_submodules;
            let item_prefix = if is_last { &config.tree_last } else { &config.tree_branch };
            result.push_str(&format!("{}{}{}\n", prefix, 
                colorize(item_prefix, &config.color_scheme.tree_color, config), 
                item
            ));
        }
    }

    // Process submodules recursively
    if let Some(submodules) = tree_dict.get("submodules") {
        let submods: HashMap<String, PyObject> = submodules.extract(py)?;
        let mut submod_names: Vec<_> = submods.keys().cloned().collect();
        submod_names.sort();

        for (i, name) in submod_names.iter().enumerate() {
            let is_last = i == submod_names.len() - 1;
            let submod_prefix = if is_last { &config.tree_last } else { &config.tree_branch };

            result.push_str(&format!("{}{}{} {}\n", prefix, 
                colorize(submod_prefix, &config.color_scheme.tree_color, config),
                colorize(&config.module_icon, &config.color_scheme.module_color, config),
                colorize(name, &config.color_scheme.module_color, config)
            ));

            if let Some(submod_tree) = submods.get(name) {
                let submod_content = format_tree_recursive(
                    py,
                    submod_tree,
                    &format!("{}{}", prefix, if is_last { &config.tree_empty } else { &config.tree_vertical }),
                )?;
                result.push_str(&submod_content);
            }
        }
    }

    Ok(result)
}