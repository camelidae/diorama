//! On-disk blob/manifest store under `{root}/oci-registry/`.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Blobstore {
    root: PathBuf,
}

impl Blobstore {
    pub fn open(root: impl Into<PathBuf>) -> io::Result<Self> {
        let root = root.into().join("oci-registry");
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn has(&self, key: &str) -> bool {
        self.path(key).is_file()
    }

    pub fn get(&self, key: &str) -> io::Result<Option<Vec<u8>>> {
        match fs::read(self.path(key)) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn put(&self, key: &str, data: &[u8]) -> io::Result<()> {
        let path = self.path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("tmp");
        {
            let mut file = fs::File::create(&tmp)?;
            file.write_all(data)?;
            file.sync_all()?;
        }
        fs::rename(tmp, path)?;
        Ok(())
    }

    pub fn delete(&self, key: &str) -> io::Result<()> {
        match fs::remove_file(self.path(key)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn list_prefix(&self, prefix: &str) -> io::Result<Vec<String>> {
        let mut out = Vec::new();
        self.walk(&self.root, "", prefix, &mut out)?;
        out.sort();
        Ok(out)
    }

    fn walk(&self, dir: &Path, rel: &str, prefix: &str, out: &mut Vec<String>) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let child_rel = if rel.is_empty() {
                name.into_owned()
            } else {
                format!("{rel}/{name}")
            };
            let path = entry.path();
            if path.is_dir() {
                if prefix.starts_with(&child_rel) || child_rel.starts_with(prefix) {
                    self.walk(&path, &child_rel, prefix, out)?;
                }
            } else if child_rel.starts_with(prefix) && !child_rel.ends_with(".tmp") {
                out.push(child_rel);
            }
        }
        Ok(())
    }

    fn path(&self, key: &str) -> PathBuf {
        let mut path = self.root.clone();
        for part in key.split('/') {
            if part.is_empty() || part == "." || part == ".." {
                continue;
            }
            path.push(part);
        }
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("oci-reg-wasip3-{nanos}"))
    }

    #[test]
    fn round_trip() {
        let dir = tmp();
        let store = Blobstore::open(&dir).unwrap();
        store.put("repo/blobs/sha256_abc", b"hello").unwrap();
        assert!(store.has("repo/blobs/sha256_abc"));
        assert_eq!(
            store.get("repo/blobs/sha256_abc").unwrap().as_deref(),
            Some(b"hello".as_slice())
        );
        let listed = store.list_prefix("repo/").unwrap();
        assert_eq!(listed, vec!["repo/blobs/sha256_abc".to_string()]);
        let _ = fs::remove_dir_all(dir);
    }
}
