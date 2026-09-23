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
    /// The entry is there and its permissions say no. Worded as a JDK words
    /// it, because it reaches a program through the same
    /// `FileNotFoundException` an absent file does.
    #[error("{0} (Permission denied)")]
    PermissionDenied(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    File(Vec<u8>),
    Directory,
}

/// Which permission bit `setReadable`/`setWritable`/`setExecutable` names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Read,
    Write,
    Execute,
}

/// One entry, and what `java.io.File` can ask about it besides its bytes.
///
/// The permissions are real state rather than a fixed answer: a program that
/// calls `setReadOnly()` and then `canWrite()` is testing exactly this, and a
/// filesystem that always said "writable" would answer it wrongly.
///
/// They are also ENFORCED. That was worth measuring rather than assuming: the
/// first version of this comment said they were advisory, and a JDK capture
/// said otherwise — opening a read-only file for writing is
/// `FileNotFoundException: name (Permission denied)`, and so is reading one
/// that is not readable. What is NOT gated is metadata: `exists`, `length`
/// and `delete` all work on a file a program has locked itself out of.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    node: Node,
    readable: bool,
    writable: bool,
    executable: bool,
    /// Milliseconds since the epoch, as `File.lastModified` reports them. Zero
    /// until the host supplies a clock, which is also what a JDK answers for a
    /// file that is not there.
    modified: i64,
}

impl Entry {
    /// A newly written file or directory: readable and writable, and not
    /// executable — which is what a JDK reports for a file a program just
    /// made, and `true` for the execute bit only on a directory.
    fn new(node: Node) -> Self {
        let executable = matches!(node, Node::Directory);
        Self {
            node,
            readable: true,
            writable: true,
            executable,
            modified: 0,
        }
    }
}

/// A flat-map filesystem: normalized absolute path → node.
///
/// The root directory `/` always exists. Parent directories are created
/// implicitly on write (like `mkdir -p`), which keeps the common
/// student workflow — "just write a file" — friction-free.
#[derive(Debug, Clone)]
pub struct VirtualFileSystem {
    nodes: BTreeMap<String, Entry>,
    /// What the host's clock last said, in milliseconds since the epoch. Every
    /// write stamps the entry it touches from HERE rather than from a clock of
    /// its own, so no write path can forget to — the defect a per-call-site
    /// stamp would keep inviting.
    clock: i64,
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
        nodes.insert("/".to_owned(), Entry::new(Node::Directory));
        Self { nodes, clock: 0 }
    }

    /// Tell the filesystem what time it is. The interpreter does this on the
    /// way into every intrinsic call, so a write always has a current clock
    /// without every write path having to ask for one.
    pub fn set_clock(&mut self, millis: i64) {
        self.clock = millis;
    }

    /// Write a node, KEEPING the permissions and time an entry at that path
    /// already had. Rewriting a file does not make it writable again — a
    /// program that calls `setReadOnly()` and then writes anyway must still
    /// find `canWrite()` false, because that is what a JDK's file says.
    fn write_entry(&mut self, path: String, node: Node) {
        let clock = self.clock;
        if let Some(existing) = self.nodes.get_mut(&path) {
            existing.node = node;
            existing.modified = clock;
            return;
        }
        let mut entry = Entry::new(node);
        entry.modified = clock;
        self.nodes.insert(path, entry);
    }

    /// Whether a write to this path is allowed: the FILE's own bit when it is
    /// already there, and otherwise the DIRECTORY it would be created in — a
    /// JDK refuses `new PrintWriter(new File(readOnlyDir, "x"))` for the
    /// directory's sake, not the file's.
    fn check_writable(&self, path: &str) -> Result<(), VfsError> {
        if let Some(entry) = self.nodes.get(path) {
            return if entry.writable {
                Ok(())
            } else {
                Err(VfsError::PermissionDenied(path.to_owned()))
            };
        }
        match Self::parent_of(path).and_then(|parent| self.nodes.get(parent)) {
            Some(parent) if !parent.writable => Err(VfsError::PermissionDenied(path.to_owned())),
            _ => Ok(()),
        }
    }

    /// The three permission bits, as `canRead`/`canWrite`/`canExecute` answer
    /// them. A path that is not there has none of them, which is a JDK's
    /// answer too — the question is about a file, and there is no file.
    #[must_use]
    pub fn permissions(&self, path: &str) -> (bool, bool, bool) {
        self.nodes
            .get(&Self::normalize(path))
            .map_or((false, false, false), |entry| {
                (entry.readable, entry.writable, entry.executable)
            })
    }

    /// Set one of them, and say whether there was a file to set it on —
    /// which is what `setReadable` and its three siblings return.
    pub fn set_permission(&mut self, path: &str, which: Permission, value: bool) -> bool {
        let Some(entry) = self.nodes.get_mut(&Self::normalize(path)) else {
            return false;
        };
        match which {
            Permission::Read => entry.readable = value,
            Permission::Write => entry.writable = value,
            Permission::Execute => entry.executable = value,
        }
        true
    }

    /// Milliseconds since the epoch, or 0 for a path that is not there.
    #[must_use]
    pub fn modified(&self, path: &str) -> i64 {
        self.nodes
            .get(&Self::normalize(path))
            .map_or(0, |entry| entry.modified)
    }

    /// ...and set it, answering whether there was a file to set it on.
    pub fn set_modified(&mut self, path: &str, millis: i64) -> bool {
        match self.nodes.get_mut(&Self::normalize(path)) {
            Some(entry) => {
                entry.modified = millis;
                true
            }
            None => false,
        }
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

    /// Write a file, overwriting any existing file at that path — INTO a
    /// directory that already exists, which is the only place a JDK writes
    /// one. This used to make the parent chain, so every write path here
    /// silently succeeded where a JDK throws: `new PrintWriter("out/log.txt")`
    /// with no `out` directory wrote the file and made the directory, and
    /// `Files.writeString`, `Files.write`, `newBufferedWriter`, `copy`, `move`
    /// and `File.createNewFile` did the same — a program that "worked" here
    /// and failed on a JDK, which is the dangerous direction.
    ///
    /// [`Self::seed_file`] is the other half: a HOST putting files into the
    /// filesystem before the program runs has no directories to make them in.
    pub fn write_file(&mut self, path: &str, contents: impl Into<Vec<u8>>) -> Result<(), VfsError> {
        let path = Self::normalize(path);
        if matches!(self.nodes.get(&path), Some(e) if e.node == Node::Directory) {
            return Err(VfsError::IsDirectory(path));
        }
        self.check_writable(&path)?;
        // A JDK tells the two apart: the directory above is MISSING
        // (`NoSuchFileException: out/log.txt`, naming the file) or it is a
        // FILE (`FileSystemException: f.txt/child: Not a directory`).
        match Self::parent_of(&path) {
            Some(dir) if dir != "/" => match self.nodes.get(dir) {
                Some(Entry {
                    node: Node::Directory,
                    ..
                }) => {}
                Some(_) => return Err(VfsError::NotADirectory(dir.to_owned())),
                None => return Err(VfsError::NotFound(path)),
            },
            _ => {}
        }
        self.write_entry(path, Node::File(contents.into()));
        Ok(())
    }

    /// Write a file and MAKE the directories above it — for a host seeding the
    /// filesystem before a program runs (the playground's assets, a sweep's
    /// inputs, a test's fixture). No Java call takes this route: see
    /// [`Self::write_file`].
    pub fn seed_file(&mut self, path: &str, contents: impl Into<Vec<u8>>) -> Result<(), VfsError> {
        let path = Self::normalize(path);
        if matches!(self.nodes.get(&path), Some(e) if e.node == Node::Directory) {
            return Err(VfsError::IsDirectory(path));
        }
        self.check_writable(&path)?;
        self.mkdir_parents(&path)?;
        self.write_entry(path, Node::File(contents.into()));
        Ok(())
    }

    /// Read a file's contents.
    pub fn read_file(&self, path: &str) -> Result<&[u8], VfsError> {
        let path = Self::normalize(path);
        match self.nodes.get(&path) {
            Some(Entry {
                readable: false, ..
            }) => Err(VfsError::PermissionDenied(path)),
            Some(Entry {
                node: Node::File(bytes),
                ..
            }) => Ok(bytes),
            Some(Entry {
                node: Node::Directory,
                ..
            }) => Err(VfsError::IsDirectory(path)),
            // A path UNDER a file is not merely absent: a JDK says "Not a
            // directory" about it, on the read side as on the write side.
            None => Err(self.absent(path)),
        }
    }

    /// Why a path is not there: because nothing is at it, or because the
    /// directory it names is a FILE. One reader, so the read and the write
    /// answer the same question the same way.
    fn absent(&self, path: String) -> VfsError {
        match Self::parent_of(&path) {
            Some(dir)
                if dir != "/"
                    && matches!(
                        self.nodes.get(dir),
                        Some(Entry {
                            node: Node::File(_),
                            ..
                        })
                    ) =>
            {
                VfsError::NotADirectory(dir.to_owned())
            }
            _ => VfsError::NotFound(path),
        }
    }

    /// Append to a file, creating it (and parents) if absent.
    pub fn append_file(&mut self, path: &str, contents: &[u8]) -> Result<(), VfsError> {
        let path = Self::normalize(path);
        self.check_writable(&path)?;
        let clock = self.clock;
        match self.nodes.get_mut(&path) {
            // An append is a write: it stamps the time like any other, which
            // is what makes a file a program has just printed into report a
            // `lastModified` rather than the zero of a file that is not there.
            Some(Entry {
                node: Node::File(bytes),
                modified,
                ..
            }) => {
                bytes.extend_from_slice(contents);
                *modified = clock;
                Ok(())
            }
            Some(Entry {
                node: Node::Directory,
                ..
            }) => Err(VfsError::IsDirectory(path)),
            None => self.write_file(&path, contents),
        }
    }

    /// Create a directory (and any missing parents).
    pub fn mkdir(&mut self, path: &str) -> Result<(), VfsError> {
        let path = Self::normalize(path);
        if matches!(self.nodes.get(&path), Some(e) if matches!(e.node, Node::File(_))) {
            return Err(VfsError::AlreadyExists(path));
        }
        self.mkdir_parents(&path)?;
        self.write_entry(path, Node::Directory);
        Ok(())
    }

    fn mkdir_parents(&mut self, path: &str) -> Result<(), VfsError> {
        let mut missing = Vec::new();
        let mut current = Self::parent_of(path);
        while let Some(dir) = current {
            match self.nodes.get(dir) {
                Some(Entry {
                    node: Node::Directory,
                    ..
                }) => break,
                Some(Entry {
                    node: Node::File(_),
                    ..
                }) => return Err(VfsError::NotADirectory(dir.to_owned())),
                None => missing.push(dir.to_owned()),
            }
            current = Self::parent_of(dir);
        }
        for dir in missing {
            self.write_entry(dir, Node::Directory);
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
            Some(Entry {
                node: Node::Directory,
                ..
            })
        )
    }

    /// Whether the path names a regular file.
    #[must_use]
    pub fn is_file(&self, path: &str) -> bool {
        matches!(self.nodes.get(&Self::normalize(path)), Some(e) if matches!(e.node, Node::File(_)))
    }

    /// File length in bytes (`File.length()` semantics: 0 for
    /// directories and missing files).
    #[must_use]
    pub fn len(&self, path: &str) -> u64 {
        match self.nodes.get(&Self::normalize(path)) {
            Some(Entry {
                node: Node::File(bytes),
                ..
            }) => bytes.len() as u64,
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
            Some(Entry {
                node: Node::Directory,
                ..
            }) => {}
            Some(Entry {
                node: Node::File(_),
                ..
            }) => return Err(VfsError::NotADirectory(path)),
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

    /// Move a file or directory, with its whole subtree.
    ///
    /// `File.renameTo` on a Unix filesystem OVERWRITES an existing
    /// destination file rather than failing, which is the surprising half of
    /// it, and fails if the source is missing.
    pub fn rename(&mut self, from: &str, to: &str) -> Result<(), VfsError> {
        let from = Self::normalize(from);
        let to = Self::normalize(to);
        if !self.nodes.contains_key(&from) {
            return Err(VfsError::NotFound(from));
        }
        if from == to {
            return Ok(());
        }
        if to.starts_with(&format!("{from}/")) {
            return Err(VfsError::NotADirectory(to));
        }
        if matches!(self.nodes.get(&to), Some(e) if e.node == Node::Directory) {
            return Err(VfsError::IsDirectory(to));
        }
        // A directory moves with everything under it, so the paths to rewrite
        // are the subtree's, and they have to be collected before the map is
        // touched.
        let prefix = format!("{from}/");
        let moving: Vec<String> = self
            .nodes
            .range(from.clone()..)
            .take_while(|(p, _)| **p == from || p.starts_with(&prefix))
            .map(|(p, _)| p.clone())
            .collect();
        self.mkdir_parents(&to)?;
        for path in moving {
            let Some(node) = self.nodes.remove(&path) else {
                continue;
            };
            let moved = if path == from {
                to.clone()
            } else {
                format!("{to}{}", &path[from.len()..])
            };
            self.nodes.insert(moved, node);
        }
        Ok(())
    }

    /// Delete a file or an empty directory (`File.delete()` semantics).
    pub fn remove(&mut self, path: &str) -> Result<(), VfsError> {
        let path = Self::normalize(path);
        match self.nodes.get(&path) {
            None => return Err(VfsError::NotFound(path)),
            Some(Entry {
                node: Node::Directory,
                ..
            }) => {
                if !self.list_dir(&path)?.is_empty() {
                    return Err(VfsError::DirectoryNotEmpty(path));
                }
                if path == "/" {
                    return Err(VfsError::DirectoryNotEmpty(path));
                }
            }
            Some(Entry {
                node: Node::File(_),
                ..
            }) => {}
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
    fn rename_moves_a_whole_subtree() {
        let mut vfs = VirtualFileSystem::new();
        vfs.seed_file("/a/deep/one.txt", b"1".to_vec()).unwrap();
        vfs.seed_file("/a/two.txt", b"2".to_vec()).unwrap();
        vfs.rename("/a", "/b").unwrap();
        assert!(!vfs.exists("/a"));
        assert_eq!(vfs.read_file("/b/deep/one.txt").unwrap(), b"1");
        assert_eq!(vfs.read_file("/b/two.txt").unwrap(), b"2");
    }

    #[test]
    fn rename_overwrites_a_file_but_not_a_directory() {
        let mut vfs = VirtualFileSystem::new();
        vfs.write_file("/one.txt", b"1".to_vec()).unwrap();
        vfs.write_file("/two.txt", b"2".to_vec()).unwrap();
        vfs.mkdir("/dir").unwrap();
        // A Unix rename replaces an existing FILE silently…
        vfs.rename("/one.txt", "/two.txt").unwrap();
        assert_eq!(vfs.read_file("/two.txt").unwrap(), b"1");
        // …and refuses a directory, a missing source, and a move into itself.
        assert!(vfs.rename("/two.txt", "/dir").is_err());
        assert!(vfs.rename("/ghost.txt", "/x.txt").is_err());
        assert!(vfs.rename("/dir", "/dir/inner").is_err());
    }

    #[test]
    fn a_write_needs_the_directory_to_be_there() {
        let mut vfs = VirtualFileSystem::new();
        // A Java write does NOT make the directory above it, as a JDK's does
        // not: the complaint names the file.
        assert_eq!(
            vfs.write_file("/data/input.txt", "hello".as_bytes().to_vec()),
            Err(VfsError::NotFound("/data/input.txt".to_owned()))
        );
        vfs.mkdir("/data").unwrap();
        vfs.write_file("/data/input.txt", "hello".as_bytes().to_vec())
            .unwrap();
        assert_eq!(vfs.read_file("data/input.txt").unwrap(), b"hello");
        // ...and a HOST seeding the filesystem makes them, because there is
        // no program to have made them first.
        vfs.seed_file("/deep/down/here.txt", b"x".to_vec()).unwrap();
        assert!(vfs.is_directory("/deep/down"));
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
        vfs.seed_file("/a/one.txt", b"1".to_vec()).unwrap();
        vfs.seed_file("/a/b/two.txt", b"2".to_vec()).unwrap();
        vfs.write_file("/top.txt", b"t".to_vec()).unwrap();
        assert_eq!(vfs.list_dir("/a").unwrap(), vec!["/a/b", "/a/one.txt"]);
        assert_eq!(vfs.list_dir("/").unwrap(), vec!["/a", "/top.txt"]);
    }

    #[test]
    fn remove_refuses_non_empty_directories() {
        let mut vfs = VirtualFileSystem::new();
        vfs.seed_file("/a/one.txt", b"1".to_vec()).unwrap();
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
