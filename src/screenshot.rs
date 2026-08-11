use crate::db;
use anyhow::{Context as _, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Captures the primary screen and stores the PNG under data/shots/.
/// Retries a few times to work around intermittent GDI "Access denied" errors.
/// Returns the saved file path on success.
pub fn capture_screen(conn: &Connection) -> Result<PathBuf> {
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

    let image = capture_with_retry()
        .context("nao foi possivel capturar a tela (verifique permissoes/DPI)")?;
    image
        .save(&path)
        .with_context(|| format!("falha ao salvar {}", path.display()))?;

    let ts_str = format!("{ts}");
    db::insert_capture(conn, path.to_str().unwrap_or(&filename), &ts_str, "")?;
    Ok(path)
}

/// Tries to grab the primary display up to 3 times.
fn capture_with_retry() -> Option<screenshots::image::RgbaImage> {
    for attempt in 1..=3 {
        match try_capture() {
            Ok(img) => return Some(img),
            Err(e) => {
                eprintln!("capture attempt {attempt} failed: {e}");
                std::thread::sleep(std::time::Duration::from_millis(150));
            }
        }
    }
    None
}

fn try_capture() -> Result<screenshots::image::RgbaImage> {
    let screen = screenshots::Screen::all()?
        .into_iter()
        .find(|s| s.display_info.is_primary)
        .or_else(|| {
            screenshots::Screen::all()
                .ok()
                .and_then(|s| s.into_iter().next())
        })
        .ok_or_else(|| anyhow::anyhow!("nenhuma tela encontrada"))?;
    let image = screen.capture()?;
    Ok(image)
}
