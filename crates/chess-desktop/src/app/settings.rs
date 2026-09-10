//! Settings that survive a restart, kept as `key = value` lines in the
//! user's config directory. A dozen scalars do not need a serialiser.

use chess::Color as ChessColor;
use std::path::PathBuf;

use super::engine_comm::EngineMode;
use super::state::{CapturedPiecesStyle, ChessApp};
use crate::ui::theme::ThemeVariant;

/// Everything the user can set that is not part of a game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub theme: ThemeVariant,
    pub engine_mode: EngineMode,
    pub engine_depth: u32,
    pub engine_movetime: u64,
    pub engine_skill_level: i32,
    pub show_eval_bar: bool,
    pub captured_display_style: CapturedPiecesStyle,
    pub computer_color: ChessColor,
}

impl Settings {
    /// The saved settings, or the defaults when there is no file or a line
    /// cannot be read.
    pub fn load() -> Self {
        path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .map(|text| Self::parse(&text))
            .unwrap_or_default()
    }

    /// Write the settings; a failure is reported, not fatal.
    pub fn save(&self) {
        let Some(path) = path() else {
            return;
        };
        let written = path
            .parent()
            .map(std::fs::create_dir_all)
            .unwrap_or(Ok(()))
            .and_then(|()| std::fs::write(&path, self.to_string()));
        if let Err(e) = written {
            eprintln!("could not save settings to {}: {e}", path.display());
        }
    }

    pub fn from_app(app: &ChessApp) -> Self {
        Self {
            theme: app.theme_variant,
            engine_mode: app.engine_mode,
            engine_depth: app.engine_depth,
            engine_movetime: app.engine_movetime.unwrap_or(1000),
            engine_skill_level: app.engine_skill_level,
            show_eval_bar: app.show_eval_bar,
            captured_display_style: app.captured_display_style,
            computer_color: app.computer_color,
        }
    }

    pub fn apply(self, app: &mut ChessApp) {
        app.set_theme(self.theme);
        app.engine_mode = self.engine_mode;
        app.engine_depth = self.engine_depth;
        app.engine_movetime = Some(self.engine_movetime);
        app.engine_skill_level = self.engine_skill_level;
        app.show_eval_bar = self.show_eval_bar;
        app.captured_display_style = self.captured_display_style;
        app.computer_color = self.computer_color;
    }

    /// Read `key = value` lines; unknown keys and unreadable values keep
    /// their defaults.
    pub fn parse(text: &str) -> Self {
        let mut settings = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "theme" => {
                    if let Some(theme) = theme_from_key(value) {
                        settings.theme = theme;
                    }
                }
                "engine_mode" => match value {
                    "depth" => settings.engine_mode = EngineMode::Depth,
                    "time" => settings.engine_mode = EngineMode::TimeLimit,
                    _ => {}
                },
                "engine_depth" => {
                    if let Ok(depth) = value.parse() {
                        settings.engine_depth = depth;
                    }
                }
                "engine_movetime" => {
                    if let Ok(ms) = value.parse() {
                        settings.engine_movetime = ms;
                    }
                }
                "skill_level" => {
                    if let Ok(level) = value.parse::<i32>() {
                        settings.engine_skill_level = level.clamp(0, 20);
                    }
                }
                "show_eval_bar" => {
                    if let Ok(show) = value.parse() {
                        settings.show_eval_bar = show;
                    }
                }
                "captured_style" => match value {
                    "lichess" => settings.captured_display_style = CapturedPiecesStyle::Lichess,
                    "chesscom" => settings.captured_display_style = CapturedPiecesStyle::ChessCom,
                    _ => {}
                },
                "computer_color" => match value {
                    "white" => settings.computer_color = ChessColor::White,
                    "black" => settings.computer_color = ChessColor::Black,
                    _ => {}
                },
                _ => {}
            }
        }
        settings
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::from_app(&ChessApp::headless())
    }
}

impl std::fmt::Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "theme = {}", theme_key(self.theme))?;
        let mode = match self.engine_mode {
            EngineMode::Depth => "depth",
            EngineMode::TimeLimit => "time",
        };
        writeln!(f, "engine_mode = {mode}")?;
        writeln!(f, "engine_depth = {}", self.engine_depth)?;
        writeln!(f, "engine_movetime = {}", self.engine_movetime)?;
        writeln!(f, "skill_level = {}", self.engine_skill_level)?;
        writeln!(f, "show_eval_bar = {}", self.show_eval_bar)?;
        let style = match self.captured_display_style {
            CapturedPiecesStyle::Lichess => "lichess",
            CapturedPiecesStyle::ChessCom => "chesscom",
        };
        writeln!(f, "captured_style = {style}")?;
        let color = match self.computer_color {
            ChessColor::White => "white",
            ChessColor::Black => "black",
        };
        writeln!(f, "computer_color = {color}")
    }
}

fn theme_key(theme: ThemeVariant) -> &'static str {
    match theme {
        ThemeVariant::ClassicMonochrome => "classic-monochrome",
        ThemeVariant::WarmMinimal => "warm-minimal",
        ThemeVariant::ModernDark => "modern-dark",
    }
}

fn theme_from_key(key: &str) -> Option<ThemeVariant> {
    ThemeVariant::all()
        .into_iter()
        .find(|&theme| theme_key(theme) == key)
}

/// `$XDG_CONFIG_HOME/rust-chess-engine/settings`, or the platform's usual
/// stand-in for the first part.
fn path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("APPDATA").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("rust-chess-engine").join("settings"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_through_text() {
        let settings = Settings {
            theme: ThemeVariant::ModernDark,
            engine_mode: EngineMode::TimeLimit,
            engine_depth: 12,
            engine_movetime: 2500,
            engine_skill_level: 7,
            show_eval_bar: false,
            captured_display_style: CapturedPiecesStyle::ChessCom,
            computer_color: ChessColor::White,
        };
        assert_eq!(Settings::parse(&settings.to_string()), settings);
    }

    #[test]
    fn unreadable_lines_keep_defaults() {
        let parsed = Settings::parse("theme = neon\nengine_depth = deep\nskill_level = 99\njunk\n");
        let defaults = Settings::default();
        assert_eq!(parsed.theme, defaults.theme);
        assert_eq!(parsed.engine_depth, defaults.engine_depth);
        assert_eq!(parsed.engine_skill_level, 20);
    }
}
