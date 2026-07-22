// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::fs::File;
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Clone)]
pub struct LogStorage {
    path: PathBuf,
}

impl LogStorage {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path: PathBuf = path.into();
        Self { path }
    }

    fn make_path(&self, id: u64) -> PathBuf {
        let mut path = self.path.clone();
        path.push(format!("{}", id / 1000 / 1000));
        path.push(format!("{:03}", id / 1000 % 1000));
        path.push(format!("{:03}.log", id % 1000));
        path
    }

    pub fn open(&self, id: u64) -> std::io::Result<File> {
        File::open(self.make_path(id))
    }

    pub fn create(&self, id: u64) -> std::io::Result<File> {
        let path = self.make_path(id);
        std::fs::create_dir_all(
            path.parent()
                .expect("generated path always contains directory"),
        )?;
        std::fs::File::create(path)
    }

    pub fn remove(&self, id: u64) -> std::io::Result<()> {
        match std::fs::remove_file(self.make_path(id)) {
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use std::io::{Read as _, Write as _};

    use super::*;

    const TEST_ID: u64 = 123_456_789;

    fn write_id(storage: &LogStorage, id: u64, data: &str) -> std::io::Result<()> {
        let mut file = storage.create(id).unwrap();
        write!(file, "{}", data).unwrap();
        Ok(())
    }

    fn read_id(storage: &LogStorage, id: u64) -> std::io::Result<String> {
        let mut file = storage.open(id).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        Ok(content)
    }

    #[test]
    fn test_read_write() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = LogStorage::new(temp_dir.path().join("storage"));

        write_id(&storage, TEST_ID, "foobar").unwrap();
        assert_eq!(read_id(&storage, TEST_ID).unwrap(), "foobar");
    }

    #[test]
    fn test_rewrite() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = LogStorage::new(temp_dir.path().join("storage"));

        write_id(&storage, TEST_ID, "foobar").unwrap();
        write_id(&storage, TEST_ID, "barbaz").unwrap();
        assert_eq!(read_id(&storage, TEST_ID).unwrap(), "barbaz");
    }

    #[test]
    fn test_read_nonexistent() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = LogStorage::new(temp_dir.path().join("storage"));

        let res = storage.open(TEST_ID);
        assert!(res.is_err());
        assert!(res.is_err_and(|e| e.kind() == ErrorKind::NotFound));
    }

    #[test]
    fn test_remove() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = LogStorage::new(temp_dir.path().join("storage"));

        write_id(&storage, TEST_ID, "foobar").unwrap();
        storage.remove(TEST_ID).unwrap();
        let res = storage.open(TEST_ID);
        assert!(res.is_err());
        assert!(res.is_err_and(|e| e.kind() == ErrorKind::NotFound));
    }

    #[test]
    fn test_remove_nonexistent() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = LogStorage::new(temp_dir.path().join("storage"));

        assert!(storage.remove(TEST_ID).is_ok());
    }
}
