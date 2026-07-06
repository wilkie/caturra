//! An in-memory virtual filesystem.
//!
//! This backs `java.io.File` (and friends) for programs running in the
//! browser, where there is no real filesystem. Paths are Unix-style and
//! normalized relative to `/`. The WASM boundary exposes this to
//! TypeScript so the host page can seed input files and inspect output
//! files.

use std::collections::BTreeMap;

/// Errors returned by filesystem operations, mirroring the failure modes
/// `java.io` cares about.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum VfsError {
    #[error("no such file or directory: {0}")]
    NotFound(String),
    #[error("is a directory: {0}")]
    IsDirectory(String),
    #[error("not a directory: {0}")]
    NotADirectory(String),
    #[error("directory not empty: {0}")]
    DirectoryNotEmpty(String),
    #[error("file already exists: {0}")]
    AlreadyExists(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    File(Vec<u8>),
    Directory,
}

/// A flat-map filesystem: normalized absolute path → node.
///
/// The root directory `/` always exists. Parent directories are created
/// implicitly on write (like `mkdir -p`), which keeps the common
/// student workflow — "just write a file" — friction-free.
#[derive(Debug, Clone)]
pub struct VirtualFileSystem {
    nodes: BTreeMap<String, Node>,
}

impl Default for VirtualFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualFileSystem {
    #[must_use]
    pub fn new() -> Self {
        let mut nodes = BTreeMap::new();
        nodes.insert("/".to_owned(), Node::Directory);
        Self { nodes }
    }

    /// Normalize a path to absolute form: resolves `.` and `..`,
    /// collapses repeated slashes, and anchors relative paths at `/`.
    #[must_use]
    pub fn normalize(path: &str) -> String {
        let mut parts: Vec<&str> = Vec::new();
        for part in path.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    parts.pop();
                }
                other => parts.push(other),
            }
        }
        if parts.is_empty() {
            "/".to_owned()
        } else {
            format!("/{}", parts.join("/"))
        }
    }

    fn parent_of(path: &str) -> Option<&str> {
        if path == "/" {
            return None;
        }
        match path.rfind('/') {
            Some(0) => Some("/"),
            Some(i) => Some(&path[..i]),
            None => None,
        }
    }

    /// Write a file, creating parent directories as needed and
    /// overwriting any existing file at that path.
    pub fn write_file(&mut self, path: &str, contents: impl Into<Vec<u8>>) -> Result<(), VfsError> {
        let path = Self::normalize(path);
        if matches!(self.nodes.get(&path), Some(Node::Directory)) {
            return Err(VfsError::IsDirectory(path));
        }
        self.mkdir_parents(&path)?;
        self.nodes.insert(path, Node::File(contents.into()));
        Ok(())
    }

    /// Read a file's contents.
    pub fn read_file(&self, path: &str) -> Result<&[u8], VfsError> {
        let path = Self::normalize(path);
        match self.nodes.get(&path) {
            Some(Node::File(bytes)) => Ok(bytes),
            Some(Node::Directory) => Err(VfsError::IsDirectory(path)),
            None => Err(VfsError::NotFound(path)),
        }
    }

    /// Append to a file, creating it (and parents) if absent.
    pub fn append_file(&mut self, path: &str, contents: &[u8]) -> Result<(), VfsError> {
        let path = Self::normalize(path);
        match self.nodes.get_mut(&path) {
            Some(Node::File(bytes)) => {
                bytes.extend_from_slice(contents);
                Ok(())
            }
            Some(Node::Directory) => Err(VfsError::IsDirectory(path)),
            None => self.write_file(&path, contents),
        }
    }

    /// Create a directory (and any missing parents).
    pub fn mkdir(&mut self, path: &str) -> Result<(), VfsError> {
        let path = Self::normalize(path);
        if matches!(self.nodes.get(&path), Some(Node::File(_))) {
            return Err(VfsError::AlreadyExists(path));
        }
        self.mkdir_parents(&path)?;
        self.nodes.insert(path, Node::Directory);
        Ok(())
    }

    fn mkdir_parents(&mut self, path: &str) -> Result<(), VfsError> {
        let mut missing = Vec::new();
        let mut current = Self::parent_of(path);
        while let Some(dir) = current {
            match self.nodes.get(dir) {
                Some(Node::Directory) => break,
                Some(Node::File(_)) => return Err(VfsError::NotADirectory(dir.to_owned())),
                None => missing.push(dir.to_owned()),
            }
            current = Self::parent_of(dir);
        }
        for dir in missing {
            self.nodes.insert(dir, Node::Directory);
        }
        Ok(())
    }

    /// Whether a file or directory exists at the path.
    #[must_use]
    pub fn exists(&self, path: &str) -> bool {
        self.nodes.contains_key(&Self::normalize(path))
    }

    /// Whether the path names a directory.
    #[must_use]
    pub fn is_directory(&self, path: &str) -> bool {
        matches!(
            self.nodes.get(&Self::normalize(path)),
            Some(Node::Directory)
        )
    }

    /// Whether the path names a regular file.
    #[must_use]
    pub fn is_file(&self, path: &str) -> bool {
        matches!(self.nodes.get(&Self::normalize(path)), Some(Node::File(_)))
    }

    /// File length in bytes (`File.length()` semantics: 0 for
    /// directories and missing files).
    #[must_use]
    pub fn len(&self, path: &str) -> u64 {
        match self.nodes.get(&Self::normalize(path)) {
            Some(Node::File(bytes)) => bytes.len() as u64,
            _ => 0,
        }
    }

    /// Whether the filesystem contains only the root directory.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.len() == 1
    }

    /// List the immediate children of a directory, as absolute paths in
    /// sorted order.
    pub fn list_dir(&self, path: &str) -> Result<Vec<String>, VfsError> {
        let path = Self::normalize(path);
        match self.nodes.get(&path) {
            Some(Node::Directory) => {}
            Some(Node::File(_)) => return Err(VfsError::NotADirectory(path)),
            None => return Err(VfsError::NotFound(path)),
        }
        let prefix = if path == "/" {
            String::from("/")
        } else {
            format!("{path}/")
        };
        Ok(self
            .nodes
            .range(prefix.clone()..)
            .take_while(|(p, _)| p.starts_with(&prefix))
            .filter(|(p, _)| p.len() > prefix.len() && !p[prefix.len()..].contains('/'))
            .map(|(p, _)| p.clone())
            .collect())
    }

    /// Delete a file or an empty directory (`File.delete()` semantics).
    pub fn remove(&mut self, path: &str) -> Result<(), VfsError> {
        let path = Self::normalize(path);
        match self.nodes.get(&path) {
            None => return Err(VfsError::NotFound(path)),
            Some(Node::Directory) => {
                if !self.list_dir(&path)?.is_empty() {
                    return Err(VfsError::DirectoryNotEmpty(path));
                }
                if path == "/" {
                    return Err(VfsError::DirectoryNotEmpty(path));
                }
            }
            Some(Node::File(_)) => {}
        }
        self.nodes.remove(&path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_paths() {
        assert_eq!(VirtualFileSystem::normalize("foo/bar.txt"), "/foo/bar.txt");
        assert_eq!(VirtualFileSystem::normalize("/a//b/./c/../d"), "/a/b/d");
        assert_eq!(VirtualFileSystem::normalize("../.."), "/");
        assert_eq!(VirtualFileSystem::normalize(""), "/");
    }

    #[test]
    fn write_read_round_trip_creates_parents() {
        let mut vfs = VirtualFileSystem::new();
        vfs.write_file("/data/input.txt", "hello".as_bytes().to_vec())
            .unwrap();
        assert_eq!(vfs.read_file("data/input.txt").unwrap(), b"hello");
        assert!(vfs.is_directory("/data"));
    }

    #[test]
    fn append_creates_or_extends() {
        let mut vfs = VirtualFileSystem::new();
        vfs.append_file("/log.txt", b"a").unwrap();
        vfs.append_file("/log.txt", b"b").unwrap();
        assert_eq!(vfs.read_file("/log.txt").unwrap(), b"ab");
    }

    #[test]
    fn list_dir_returns_immediate_children_only() {
        let mut vfs = VirtualFileSystem::new();
        vfs.write_file("/a/one.txt", b"1".to_vec()).unwrap();
        vfs.write_file("/a/b/two.txt", b"2".to_vec()).unwrap();
        vfs.write_file("/top.txt", b"t".to_vec()).unwrap();
        assert_eq!(vfs.list_dir("/a").unwrap(), vec!["/a/b", "/a/one.txt"]);
        assert_eq!(vfs.list_dir("/").unwrap(), vec!["/a", "/top.txt"]);
    }

    #[test]
    fn remove_refuses_non_empty_directories() {
        let mut vfs = VirtualFileSystem::new();
        vfs.write_file("/a/one.txt", b"1".to_vec()).unwrap();
        assert_eq!(
            vfs.remove("/a"),
            Err(VfsError::DirectoryNotEmpty("/a".to_owned()))
        );
        vfs.remove("/a/one.txt").unwrap();
        vfs.remove("/a").unwrap();
        assert!(!vfs.exists("/a"));
    }

    #[test]
    fn file_over_directory_and_vice_versa_fail() {
        let mut vfs = VirtualFileSystem::new();
        vfs.mkdir("/dir").unwrap();
        assert_eq!(
            vfs.write_file("/dir", b"x".to_vec()),
            Err(VfsError::IsDirectory("/dir".to_owned()))
        );
        vfs.write_file("/file", b"x".to_vec()).unwrap();
        assert_eq!(
            vfs.write_file("/file/child", b"x".to_vec()),
            Err(VfsError::NotADirectory("/file".to_owned()))
        );
    }
}
