use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub grid: GridConfig,
    pub appearance: AppearanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GridConfig {
    pub cols: u32,
    pub rows: u32,
    pub gap: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    pub tile_color: u32,
    pub highlight_color: u32,
    pub background_color: u32,
    pub text_color: u32,
    pub alpha: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            grid: GridConfig::default(),
            appearance: AppearanceConfig::default(),
        }
    }
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            cols: 4,
            rows: 2,
            gap: 10,
        }
    }
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            tile_color: 0x00805030,      // Teal-ish
            highlight_color: 0x0000A0FF, // Orange
            background_color: 0x00302020, // Dark gray-brown
            text_color: 0x00FFFFFF,      // White
            alpha: 220,
        }
    }
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|p| p.join(".tactile-win.toml"))
    }

    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| {
                if path.exists() {
                    fs::read_to_string(&path).ok()
                } else {
                    None
                }
            })
            .and_then(|contents| toml::from_str(&contents).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = Self::config_path() {
            let contents = toml::to_string_pretty(self)?;
            fs::write(path, contents)?;
        }
        Ok(())
    }

    pub fn validate(&mut self) {
        // Clamp values to valid ranges
        self.grid.cols = self.grid.cols.clamp(1, 8);
        self.grid.rows = self.grid.rows.clamp(1, 4);
        self.grid.gap = self.grid.gap.clamp(0, 50);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.grid.cols, 4);
        assert_eq!(config.grid.rows, 2);
        assert_eq!(config.grid.gap, 10);
    }

    #[test]
    fn test_parse_toml() {
        let toml_str = r#"
[grid]
cols = 6
rows = 3
gap = 5

[appearance]
alpha = 200
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.grid.cols, 6);
        assert_eq!(config.grid.rows, 3);
        assert_eq!(config.grid.gap, 5);
        assert_eq!(config.appearance.alpha, 200);
    }

    #[test]
    fn test_validate() {
        let mut config = Config::default();
        config.grid.cols = 100;
        config.grid.rows = 0;
        config.validate();
        assert_eq!(config.grid.cols, 8);
        assert_eq!(config.grid.rows, 1);
    }
}
