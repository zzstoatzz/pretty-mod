use std::env;
use std::sync::OnceLock;

/// Configuration for display characters and styling
#[derive(Debug, Clone)]
pub struct DisplayConfig {
    // Tree display characters
    pub module_icon: String,
    pub function_icon: String,
    pub class_icon: String,
    pub constant_icon: String,
    pub exports_icon: String,

    // Signature display characters
    pub signature_icon: String,

    // Tree structure characters
    pub tree_branch: String,
    pub tree_last: String,
    pub tree_vertical: String,
    pub tree_empty: String,

    // Color configuration
    pub use_color: bool,
    pub color_scheme: ColorScheme,
}

#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub module_color: String,
    pub function_color: String,
    pub class_color: String,
    pub constant_color: String,
    pub exports_color: String,
    pub signature_color: String,
    pub tree_color: String,
    pub param_color: String,
    pub type_color: String,
    pub default_color: String,
    pub warning_color: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        // Earth tone / pastel colors
        Self {
            module_color: "#8B7355".to_string(),    // Saddle brown
            function_color: "#6B8E23".to_string(),  // Olive drab
            class_color: "#4682B4".to_string(),     // Steel blue
            constant_color: "#BC8F8F".to_string(),  // Rosy brown
            exports_color: "#9370DB".to_string(),   // Medium purple
            signature_color: "#5F9EA0".to_string(), // Cadet blue
            tree_color: "#696969".to_string(),      // Dim gray
            param_color: "#708090".to_string(),     // Slate gray
            type_color: "#778899".to_string(),      // Light slate gray
            default_color: "#8FBC8F".to_string(),   // Dark sea green
            warning_color: "#DAA520".to_string(),   // Goldenrod
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            // Default Unicode characters
            module_icon: "📦".to_string(),
            function_icon: "⚡".to_string(),
            class_icon: "🔷".to_string(),
            constant_icon: "📌".to_string(),
            exports_icon: "📜".to_string(),
            signature_icon: "📎".to_string(),

            // Tree structure
            tree_branch: "├── ".to_string(),
            tree_last: "└── ".to_string(),
            tree_vertical: "│   ".to_string(),
            tree_empty: "    ".to_string(),

            // Color enabled by default
            use_color: true,
            color_scheme: ColorScheme::default(),
        }
    }
}

static CONFIG: OnceLock<DisplayConfig> = OnceLock::new();

impl DisplayConfig {
    /// Get the global configuration instance
    pub fn get() -> &'static DisplayConfig {
        CONFIG.get_or_init(Self::from_env)
    }

    /// Create configuration from environment variables
    fn from_env() -> Self {
        let mut config = Self::default();

        // Check if we should use ASCII-only mode
        if env::var("PRETTY_MOD_ASCII").is_ok() {
            config.use_ascii_mode();
        }

        // Override individual characters from environment
        if let Ok(val) = env::var("PRETTY_MOD_MODULE_ICON") {
            config.module_icon = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_FUNCTION_ICON") {
            config.function_icon = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_CLASS_ICON") {
            config.class_icon = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_CONSTANT_ICON") {
            config.constant_icon = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_EXPORTS_ICON") {
            config.exports_icon = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_SIGNATURE_ICON") {
            config.signature_icon = val;
        }

        // Tree structure overrides
        if let Ok(val) = env::var("PRETTY_MOD_TREE_BRANCH") {
            config.tree_branch = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_TREE_LAST") {
            config.tree_last = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_TREE_VERTICAL") {
            config.tree_vertical = val;
        }

        // Color configuration
        if env::var("PRETTY_MOD_NO_COLOR").is_ok() || env::var("NO_COLOR").is_ok() {
            config.use_color = false;
        }

        // Color scheme overrides
        if let Ok(val) = env::var("PRETTY_MOD_MODULE_COLOR") {
            config.color_scheme.module_color = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_FUNCTION_COLOR") {
            config.color_scheme.function_color = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_CLASS_COLOR") {
            config.color_scheme.class_color = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_CONSTANT_COLOR") {
            config.color_scheme.constant_color = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_EXPORTS_COLOR") {
            config.color_scheme.exports_color = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_SIGNATURE_COLOR") {
            config.color_scheme.signature_color = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_TREE_COLOR") {
            config.color_scheme.tree_color = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_PARAM_COLOR") {
            config.color_scheme.param_color = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_TYPE_COLOR") {
            config.color_scheme.type_color = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_DEFAULT_COLOR") {
            config.color_scheme.default_color = val;
        }
        if let Ok(val) = env::var("PRETTY_MOD_WARNING_COLOR") {
            config.color_scheme.warning_color = val;
        }

        config
    }

    /// Switch to ASCII-only mode
    fn use_ascii_mode(&mut self) {
        self.module_icon = "[M]".to_string();
        self.function_icon = "[F]".to_string();
        self.class_icon = "[C]".to_string();
        self.constant_icon = "[K]".to_string();
        self.exports_icon = "[E]".to_string();
        self.signature_icon = "[S]".to_string();

        self.tree_branch = "|-- ".to_string();
        self.tree_last = "`-- ".to_string();
        self.tree_vertical = "|   ".to_string();
        self.tree_empty = "    ".to_string();
    }
}

/// helper to format text with color if enabled
pub fn colorize(text: &str, color: &str, config: &DisplayConfig) -> String {
    if !config.use_color {
        return text.to_string();
    }

    // Convert hex color to ANSI escape code
    if let Some(rgb) = parse_hex_color(color) {
        format!("\x1b[38;2;{};{};{}m{}\x1b[0m", rgb.0, rgb.1, rgb.2, text)
    } else {
        text.to_string()
    }
}

/// parse hex color string to RGB values
fn parse_hex_color(color: &str) -> Option<(u8, u8, u8)> {
    let color = color.trim_start_matches('#');
    if color.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&color[0..2], 16).ok()?;
    let g = u8::from_str_radix(&color[2..4], 16).ok()?;
    let b = u8::from_str_radix(&color[4..6], 16).ok()?;

    Some((r, g, b))
}
