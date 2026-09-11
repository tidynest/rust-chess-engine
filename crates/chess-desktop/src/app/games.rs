//! Games saved as PGN files in the user's data directory, named by the
//! moment they were saved.

use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::settings::app_dir;

/// `$XDG_DATA_HOME/rust-chess-engine/games`, or the platform's stand-in.
pub fn dir() -> Option<PathBuf> {
    Some(app_dir("XDG_DATA_HOME", "LOCALAPPDATA", ".local/share")?.join("games"))
}

/// Write `pgn` as `YYYY-MM-DD_HHMMSS.pgn` in the games directory.
pub fn save(pgn: &str) -> io::Result<PathBuf> {
    let dir = dir().ok_or_else(|| io::Error::other("no home directory"))?;
    save_in(&dir, pgn)
}

/// The saved games, newest first, at most `limit` of them.
pub fn list(limit: usize) -> Vec<PathBuf> {
    dir().map_or_else(Vec::new, |dir| list_in(&dir, limit))
}

/// Today as the PGN Date tag, `YYYY.MM.DD`.
pub fn today() -> String {
    let (year, month, day, ..) = utc_now();
    format!("{year:04}.{month:02}.{day:02}")
}

fn save_in(dir: &Path, pgn: &str) -> io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let (year, month, day, hour, minute, second) = utc_now();
    let path = dir.join(format!(
        "{year:04}-{month:02}-{day:02}_{hour:02}{minute:02}{second:02}.pgn"
    ));
    std::fs::write(&path, pgn)?;
    Ok(path)
}

fn list_in(dir: &Path, limit: usize) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut games: Vec<PathBuf> = entries
        .filter_map(|entry| Some(entry.ok()?.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "pgn"))
        .collect();
    // The names sort by time, so the newest come last.
    games.sort_unstable();
    games.reverse();
    games.truncate(limit);
    games
}

/// Year, month, day, hour, minute and second, in UTC. The local zone would
/// cost a dependency; at worst a game saved around midnight is filed under
/// the neighbouring day.
fn utc_now() -> (i64, u32, u32, u32, u32, u32) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let (year, month, day) = civil_from_days((secs / 86_400) as i64);
    let day_secs = (secs % 86_400) as u32;
    (
        year,
        month,
        day,
        day_secs / 3600,
        day_secs % 3600 / 60,
        day_secs % 60,
    )
}

/// Howard Hinnant's days-to-civil, for days since 1970-01-01.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(month <= 2), month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
        assert_eq!(civil_from_days(11_016), (2000, 2, 29));
        assert_eq!(civil_from_days(19_000), (2022, 1, 8));
        assert_eq!(civil_from_days(20_707), (2026, 9, 11));
    }

    #[test]
    fn saved_games_are_pgn_files_listed_newest_first() {
        let dir = std::env::temp_dir().join(format!("chess-games-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for name in [
            "2026-01-01_000000.pgn",
            "2026-02-02_000000.pgn",
            "notes.txt",
        ] {
            std::fs::write(dir.join(name), "x").unwrap();
        }
        let saved = save_in(&dir, "1. e4 *").unwrap();
        assert_eq!(saved.extension().unwrap(), "pgn");
        assert_eq!(std::fs::read_to_string(&saved).unwrap(), "1. e4 *");

        let listed = list_in(&dir, 10);
        assert_eq!(listed.len(), 3);
        assert_eq!(listed[0], saved);
        assert!(listed[2].ends_with("2026-01-01_000000.pgn"));
        assert_eq!(list_in(&dir, 1), vec![saved]);

        std::fs::remove_dir_all(&dir).unwrap();
        assert!(list_in(&dir, 10).is_empty());
    }
}
