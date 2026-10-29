//! Atomarer Ersatz im selben Verzeichnis; eine beschädigte Datei bleibt erhalten.
use std::{fs, io::Write, path::Path};

pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let pending = path.with_extension("pending");
    let mut file = fs::File::create(&pending)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(pending, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ersetzt_vorhandene_datei_atomar() {
        let dir = std::env::temp_dir().join(format!("cnp-{}", uuid::Uuid::new_v4()));
        let path = dir.join("magazine.json");
        write_atomic(&path, b"old").unwrap();
        write_atomic(&path, b"new").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"new");
        assert!(!path.with_extension("pending").exists());
    }
}
