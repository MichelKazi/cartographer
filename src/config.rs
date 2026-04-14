use core_graphics::event::CGEventFlags;
use serde::Deserialize;
use std::time::Duration;

use crate::ffi;

// -- top-level config --

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub hotkey: HotkeyConfig,
    pub grid: GridConfig,
    pub appearance: AppearanceConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: HotkeyConfig::default(),
            grid: GridConfig::default(),
            appearance: AppearanceConfig::default(),
        }
    }
}

// -- hotkey --

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct HotkeyConfig {
    pub trigger_key: String,
    pub modifiers: Vec<String>,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            trigger_key: "t".into(),
            modifiers: vec!["alt".into(), "cmd".into()],
        }
    }
}

impl HotkeyConfig {
    /// Resolve key name + modifier names into (keycode, flags) for the event tap.
    pub fn resolve(&self) -> Result<(i64, CGEventFlags), String> {
        let keycode = key_name_to_keycode(&self.trigger_key)
            .ok_or_else(|| format!("unknown trigger key: {:?}", self.trigger_key))?
            as i64;

        let mut flags = CGEventFlags::empty();
        for name in &self.modifiers {
            let flag = modifier_name_to_flag(name)
                .ok_or_else(|| format!("unknown modifier: {:?}", name))?;
            flags |= flag;
        }

        Ok((keycode, flags))
    }
}

// -- grid --

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct GridConfig {
    pub cols: usize,
    pub rows: usize,
    pub keys: Vec<Vec<String>>,
    pub selection_timeout_ms: u64,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            cols: 4,
            rows: 3,
            keys: vec![
                vec!["q".into(), "w".into(), "e".into(), "r".into()],
                vec!["a".into(), "s".into(), "d".into(), "f".into()],
                vec!["z".into(), "x".into(), "c".into(), "v".into()],
            ],
            selection_timeout_ms: 1000,
        }
    }
}

impl GridConfig {
    pub fn selection_timeout(&self) -> Duration {
        Duration::from_millis(self.selection_timeout_ms)
    }

    /// Convert the key name grid into a runtime (keycode, col, row) map.
    pub fn build_keycode_map(&self) -> Result<Vec<(u16, usize, usize)>, String> {
        let mut map = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for (row, row_keys) in self.keys.iter().enumerate() {
            for (col, key_name) in row_keys.iter().enumerate() {
                let keycode = key_name_to_keycode(key_name)
                    .ok_or_else(|| format!("unknown key name: {:?}", key_name))?;
                if !seen.insert(keycode) {
                    return Err(format!("duplicate key: {:?}", key_name));
                }
                map.push((keycode, col, row));
            }
        }

        Ok(map)
    }

    /// Build the "Q,W,E,R;A,S,D,F;Z,X,C,V" label string for Swift.
    pub fn build_label_string(&self) -> String {
        self.keys
            .iter()
            .map(|row| {
                row.iter()
                    .map(|k| k.to_uppercase())
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join(";")
    }
}

// -- appearance --

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    pub background_opacity: f64,
    pub border_color: [f64; 4],
    pub fill_color: [f64; 4],
    pub highlight_color: [f64; 4],
    pub text_color: [f64; 4],
    pub font_size_ratio: f64,
    pub border_width: f64,
    pub cell_gap: f64,
    pub corner_radius: f64,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            background_opacity: 0.55,
            border_color: [0.5, 0.5, 1.0, 0.4],
            fill_color: [0.5, 0.5, 1.0, 0.08],
            highlight_color: [0.5, 0.5, 1.0, 0.3],
            text_color: [0.5, 0.5, 1.0, 0.9],
            font_size_ratio: 0.4,
            border_width: 1.0,
            cell_gap: 8.0,
            corner_radius: 8.0,
        }
    }
}

impl AppearanceConfig {
    pub fn to_ffi(&self) -> ffi::OverlayAppearance {
        let [br, bg, bb, ba] = self.border_color;
        let [fr, fg, fb, fa] = self.fill_color;
        let [hr, hg, hb, ha] = self.highlight_color;
        let [tr, tg, tb, ta] = self.text_color;
        ffi::OverlayAppearance {
            background_opacity: self.background_opacity,
            border_r: br, border_g: bg, border_b: bb, border_a: ba,
            fill_r: fr, fill_g: fg, fill_b: fb, fill_a: fa,
            highlight_r: hr, highlight_g: hg, highlight_b: hb, highlight_a: ha,
            text_r: tr, text_g: tg, text_b: tb, text_a: ta,
            font_size_ratio: self.font_size_ratio,
            border_width: self.border_width,
            cell_gap: self.cell_gap,
            corner_radius: self.corner_radius,
        }
    }
}

// -- loading --

pub fn load() -> Config {
    let path = dirs::home_dir().map(|h| h.join(".config").join("cartographer").join("config.toml"));

    let config = match path {
        Some(p) if p.exists() => {
            match std::fs::read_to_string(&p) {
                Ok(contents) => match toml::from_str::<Config>(&contents) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("[cartographer] config parse error: {e}, using defaults");
                        Config::default()
                    }
                },
                Err(e) => {
                    eprintln!("[cartographer] couldn't read config: {e}, using defaults");
                    Config::default()
                }
            }
        }
        _ => Config::default(),
    };

    if let Err(e) = validate(&config) {
        eprintln!("[cartographer] config validation failed: {e}, using defaults");
        return Config::default();
    }

    config
}

fn validate(config: &Config) -> Result<(), String> {
    if config.grid.cols == 0 || config.grid.rows == 0 {
        return Err("grid dimensions must be >= 1".into());
    }

    if config.grid.keys.len() != config.grid.rows {
        return Err(format!(
            "expected {} rows of keys, got {}",
            config.grid.rows,
            config.grid.keys.len()
        ));
    }

    for (i, row) in config.grid.keys.iter().enumerate() {
        if row.len() != config.grid.cols {
            return Err(format!(
                "row {} has {} keys, expected {}",
                i,
                row.len(),
                config.grid.cols
            ));
        }
    }

    // this also checks for unknown key names and duplicates
    config.grid.build_keycode_map()?;
    config.hotkey.resolve()?;

    if config.grid.selection_timeout_ms < 100 {
        return Err("selection_timeout_ms must be >= 100".into());
    }

    Ok(())
}

// -- keycode mapping --
// macOS virtual keycodes for QWERTY layout

pub fn key_name_to_keycode(name: &str) -> Option<u16> {
    match name.to_lowercase().as_str() {
        "a" => Some(0),
        "s" => Some(1),
        "d" => Some(2),
        "f" => Some(3),
        "h" => Some(4),
        "g" => Some(5),
        "z" => Some(6),
        "x" => Some(7),
        "c" => Some(8),
        "v" => Some(9),
        "b" => Some(11),
        "q" => Some(12),
        "w" => Some(13),
        "e" => Some(14),
        "r" => Some(15),
        "y" => Some(16),
        "t" => Some(17),
        "1" => Some(18),
        "2" => Some(19),
        "3" => Some(20),
        "4" => Some(21),
        "5" => Some(23),
        "6" => Some(22),
        "7" => Some(26),
        "8" => Some(28),
        "9" => Some(25),
        "0" => Some(29),
        "o" => Some(31),
        "u" => Some(32),
        "i" => Some(34),
        "p" => Some(35),
        "l" => Some(37),
        "j" => Some(38),
        "k" => Some(40),
        "n" => Some(45),
        "m" => Some(46),
        "space" => Some(49),
        "return" | "enter" => Some(36),
        "tab" => Some(48),
        "escape" | "esc" => Some(53),
        "backspace" | "delete" => Some(51),
        "minus" => Some(27),
        "equal" | "equals" => Some(24),
        "leftbracket" => Some(33),
        "rightbracket" => Some(30),
        "semicolon" => Some(41),
        "quote" => Some(39),
        "comma" => Some(43),
        "period" => Some(47),
        "slash" => Some(44),
        "backslash" => Some(42),
        "backtick" | "grave" => Some(50),
        _ => None,
    }
}

fn modifier_name_to_flag(name: &str) -> Option<CGEventFlags> {
    match name.to_lowercase().as_str() {
        "alt" | "option" | "opt" => Some(CGEventFlags::CGEventFlagAlternate),
        "cmd" | "command" => Some(CGEventFlags::CGEventFlagCommand),
        "shift" => Some(CGEventFlags::CGEventFlagShift),
        "ctrl" | "control" => Some(CGEventFlags::CGEventFlagControl),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = Config::default();
        assert!(validate(&config).is_ok());
    }

    #[test]
    fn empty_toml_yields_defaults() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config.grid.cols, 4);
        assert_eq!(config.grid.rows, 3);
        assert_eq!(config.hotkey.trigger_key, "t");
        assert_eq!(config.hotkey.modifiers, vec!["alt", "cmd"]);
    }

    #[test]
    fn partial_toml_fills_defaults() {
        let config: Config = toml::from_str("[hotkey]\ntrigger_key = \"g\"").unwrap();
        assert_eq!(config.hotkey.trigger_key, "g");
        assert_eq!(config.hotkey.modifiers, vec!["alt", "cmd"]); // default preserved
        assert_eq!(config.grid.cols, 4); // default preserved
    }

    #[test]
    fn key_name_mapping_basics() {
        assert_eq!(key_name_to_keycode("t"), Some(17));
        assert_eq!(key_name_to_keycode("q"), Some(12));
        assert_eq!(key_name_to_keycode("a"), Some(0));
        assert_eq!(key_name_to_keycode("v"), Some(9));
        assert_eq!(key_name_to_keycode("space"), Some(49));
        assert_eq!(key_name_to_keycode("nope"), None);
    }

    #[test]
    fn key_name_case_insensitive() {
        assert_eq!(key_name_to_keycode("T"), Some(17));
        assert_eq!(key_name_to_keycode("Q"), Some(12));
        assert_eq!(key_name_to_keycode("SPACE"), Some(49));
    }

    #[test]
    fn modifier_mapping() {
        assert!(modifier_name_to_flag("alt").is_some());
        assert!(modifier_name_to_flag("cmd").is_some());
        assert!(modifier_name_to_flag("shift").is_some());
        assert!(modifier_name_to_flag("ctrl").is_some());
        assert!(modifier_name_to_flag("option").is_some());
        assert!(modifier_name_to_flag("command").is_some());
        assert!(modifier_name_to_flag("nope").is_none());
    }

    #[test]
    fn default_hotkey_resolves() {
        let hk = HotkeyConfig::default();
        let (keycode, flags) = hk.resolve().unwrap();
        assert_eq!(keycode, 17); // T
        assert!(flags.contains(CGEventFlags::CGEventFlagAlternate));
        assert!(flags.contains(CGEventFlags::CGEventFlagCommand));
    }

    #[test]
    fn default_grid_builds_keycode_map() {
        let gc = GridConfig::default();
        let map = gc.build_keycode_map().unwrap();
        assert_eq!(map.len(), 12);
        // Q -> (12, 0, 0)
        assert!(map.contains(&(12, 0, 0)));
        // V -> (9, 3, 2)
        assert!(map.contains(&(9, 3, 2)));
    }

    #[test]
    fn grid_key_count_mismatch_fails() {
        let config = Config {
            grid: GridConfig {
                cols: 5, // says 5 but keys have 4 per row
                ..GridConfig::default()
            },
            ..Config::default()
        };
        assert!(validate(&config).is_err());
    }

    #[test]
    fn duplicate_key_fails() {
        let config = Config {
            grid: GridConfig {
                keys: vec![
                    vec!["q".into(), "q".into(), "e".into(), "r".into()], // q twice
                    vec!["a".into(), "s".into(), "d".into(), "f".into()],
                    vec!["z".into(), "x".into(), "c".into(), "v".into()],
                ],
                ..GridConfig::default()
            },
            ..Config::default()
        };
        assert!(validate(&config).is_err());
    }

    #[test]
    fn unknown_key_fails_validation() {
        let config = Config {
            grid: GridConfig {
                keys: vec![
                    vec!["q".into(), "w".into(), "e".into(), "nope".into()],
                    vec!["a".into(), "s".into(), "d".into(), "f".into()],
                    vec!["z".into(), "x".into(), "c".into(), "v".into()],
                ],
                ..GridConfig::default()
            },
            ..Config::default()
        };
        assert!(validate(&config).is_err());
    }

    #[test]
    fn label_string_builds_correctly() {
        let gc = GridConfig::default();
        assert_eq!(gc.build_label_string(), "Q,W,E,R;A,S,D,F;Z,X,C,V");
    }

    #[test]
    fn timeout_too_low_fails() {
        let config = Config {
            grid: GridConfig {
                selection_timeout_ms: 50,
                ..GridConfig::default()
            },
            ..Config::default()
        };
        assert!(validate(&config).is_err());
    }
}
