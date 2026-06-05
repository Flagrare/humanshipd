use rand::RngCore;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Directory holding the host's local key material (`~/.humanshipd`).
pub fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".humanshipd")
}

/// Load a 32-byte seed from `~/.humanshipd/<name>`, generating + persisting a
/// random one (0600) on first use.
pub fn load_or_create_seed(name: &str) -> io::Result<[u8; 32]> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join(name);

    if path.exists() {
        let bytes = fs::read(&path)?;
        return bytes.as_slice().try_into().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "stored seed is not 32 bytes")
        });
    }

    let mut seed = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut seed);
    fs::write(&path, seed)?;
    restrict_permissions(&path)?;
    Ok(seed)
}

#[cfg(unix)]
fn restrict_permissions(path: &std::path::Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &std::path::Path) -> io::Result<()> {
    Ok(())
}
