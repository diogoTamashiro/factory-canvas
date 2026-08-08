use serde_json::Value;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Path to the Python executable inside the project venv.
fn python_path() -> PathBuf {
    let mut p = std::env::current_exe().ok();
    if let Some(exe) = p.as_mut() {
        exe.pop(); // remove executable name
    }
    let mut base = p.unwrap_or_else(|| PathBuf::from("."));
    base.push(".venv");
    base.push("Scripts");
    base.push("python.exe");
    base
}

/// Path to the solver script, relative to the executable's directory.
fn solver_path() -> PathBuf {
    let mut p = std::env::current_exe().ok();
    if let Some(exe) = p.as_mut() {
        exe.pop();
    }
    let mut base = p.unwrap_or_else(|| PathBuf::from("."));
    base.push("solver");
    base.push("solve.py");
    base
}

/// Runs the Python OR-Tools solver with the given JSON request and returns its
/// parsed JSON response. Errors are surfaced as `Err`.
pub fn run_solver(request: &Value) -> anyhow::Result<Value> {
    let python = python_path();
    let solver = solver_path();

    if !python.exists() {
        anyhow::bail!("python nao encontrado em {:?}", python);
    }
    if !solver.exists() {
        anyhow::bail!("solver nao encontrado em {:?}", solver);
    }

    let mut child = Command::new(&python)
        .arg(&solver)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let input = serde_json::to_string(request)?;
    child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("falha ao abrir stdin do solver"))?
        .write_all(input.as_bytes())?;

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("solver falhou: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout)?;
    Ok(parsed)
}
