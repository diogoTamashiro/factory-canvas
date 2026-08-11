use rusqlite::{Connection, Result};
use std::path::PathBuf;

/// Path to the local SQLite database, stored under data/ next to the executable.
fn db_path() -> PathBuf {
    let mut path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    path.push("data");
    std::fs::create_dir_all(&path).ok();
    path.push("softfactory.db");
    path
}

/// Opens (and initializes if needed) the local SQLite database.
pub fn init_db() -> Result<Connection> {
    let conn = Connection::open(db_path())?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS captures (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL,
            ts   TEXT NOT NULL,
            tag  TEXT
        );
        CREATE TABLE IF NOT EXISTS blueprints (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            name       TEXT NOT NULL,
            graph_json TEXT NOT NULL,
            ts         TEXT NOT NULL
        );",
    )?;
    Ok(conn)
}

/// Records a screen capture in the database.
pub fn insert_capture(conn: &Connection, path: &str, ts: &str, tag: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO captures (path, ts, tag) VALUES (?1, ?2, ?3)",
        (path, ts, tag),
    )?;
    Ok(())
}

/// Records a saved blueprint (factory graph) in the database.
pub fn insert_blueprint(conn: &Connection, name: &str, graph_json: &str, ts: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO blueprints (name, graph_json, ts) VALUES (?1, ?2, ?3)",
        (name, graph_json, ts),
    )?;
    Ok(())
}

/// A row of the blueprints table.
pub struct BlueprintRow {
    pub id: i64,
    pub name: String,
    pub graph_json: String,
    pub ts: String,
}

/// A row of the captures table.
pub struct CaptureRow {
    pub id: i64,
    pub path: String,
    pub ts: String,
    pub tag: String,
}

/// Lists all recorded captures, newest first.
pub fn list_captures(conn: &Connection) -> Result<Vec<CaptureRow>> {
    let mut stmt = conn.prepare("SELECT id, path, ts, tag FROM captures ORDER BY id DESC")?;
    let rows = stmt.query_map([], |r| {
        Ok(CaptureRow {
            id: r.get(0)?,
            path: r.get(1)?,
            ts: r.get(2)?,
            tag: r.get(3)?,
        })
    })?;
    rows.collect()
}

/// Lists all saved blueprints, newest first.
pub fn list_blueprints(conn: &Connection) -> Result<Vec<BlueprintRow>> {
    let mut stmt =
        conn.prepare("SELECT id, name, graph_json, ts FROM blueprints ORDER BY id DESC")?;
    let rows = stmt.query_map([], |r| {
        Ok(BlueprintRow {
            id: r.get(0)?,
            name: r.get(1)?,
            graph_json: r.get(2)?,
            ts: r.get(3)?,
        })
    })?;
    rows.collect()
}

/// Gets a single blueprint by name (most recent match).
pub fn get_blueprint(conn: &Connection, name: &str) -> Result<Option<BlueprintRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, graph_json, ts FROM blueprints WHERE name = ?1 ORDER BY id DESC LIMIT 1",
    )?;
    let mut rows = stmt.query_map((name,), |r| {
        Ok(BlueprintRow {
            id: r.get(0)?,
            name: r.get(1)?,
            graph_json: r.get(2)?,
            ts: r.get(3)?,
        })
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blueprint_sqlite_roundtrip() {
        // DB temporário em memória.
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE blueprints (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                graph_json TEXT NOT NULL,
                ts TEXT NOT NULL
            );",
        )
        .unwrap();
        insert_blueprint(&conn, "teste", "{\"w\":2,\"h\":2,\"tiles\":[]}", "123").unwrap();
        let row = get_blueprint(&conn, "teste").unwrap().expect("deve achar");
        assert_eq!(row.name, "teste");
        assert_eq!(row.graph_json, "{\"w\":2,\"h\":2,\"tiles\":[]}");
        // Nome inexistente => None.
        assert!(get_blueprint(&conn, "naoexiste").unwrap().is_none());
    }
}
