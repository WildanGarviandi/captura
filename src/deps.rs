//! System dependency detection and installation.

/// Returns `true` if an executable named `name` is on the PATH.
pub fn cmd_exists(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Install `slurp` via the host package manager using `pkexec`.
///
/// Runs as a side-effect task; the caller dispatches the result into
/// [`crate::message::Message::SlurpInstalled`].
pub async fn install_slurp() -> Result<(), String> {
    let script = r#"
    if command -v pacman >/dev/null 2>&1; then
        pacman -S --noconfirm slurp
    elif command -v dnf >/dev/null 2>&1; then
        dnf install -y slurp
    elif command -v apt-get >/dev/null 2>&1; then
        apt-get update && apt-get install -y slurp
    elif command -v zypper >/dev/null 2>&1; then
        zypper install -y slurp
    elif command -v apk >/dev/null 2>&1; then
        apk add slurp
    else
        exit 1
    fi
    "#;

    tokio::process::Command::new("pkexec")
        .arg("sh")
        .arg("-c")
        .arg(script)
        .status()
        .await
        .map_err(|e| e.to_string())
        .and_then(|s| {
            if s.success() {
                Ok(())
            } else {
                Err("Installation failed or unsupported package manager".to_string())
            }
        })
}