use crate::db;
use rusqlite::Connection;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Captures the primary screen and stores the PNG under data/shots/.
/// Returns the saved file path on success.
pub fn capture_screen(conn: &Connection) -> anyhow::Result<PathBuf> {
    let mut dir = std::env::current_exe()?;
    dir.pop();
    dir.push("data");
    dir.push("shots");
    std::fs::create_dir_all(&dir)?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let filename = format!("shot_{ts}.png");
    let path = dir.join(&filename);

    let screen = screenshots::Screen::all()?
        .into_iter()
        .find(|s| s.display_info.is_primary)
        .ok_or_else(|| anyhow::anyhow!("no primary screen found"))?;

    let image = screen.capture()?;
    image.save(&path)?;

    let ts_str = format!("{ts}");
    db::insert_capture(conn, path.to_str().unwrap_or(&filename), &ts_str, "")?;
    Ok(path)
}
