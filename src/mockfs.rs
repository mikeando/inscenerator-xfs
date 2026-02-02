use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::{Result, Xfs, XfsDirEntry, XfsError, XfsMetadata, XfsReadDir, AlreadyExistsSnafu, NotADirectorySnafu, NotAFileSnafu, GeneralSnafu};

pub struct MockWriter {
    data: Rc<RefCell<Vec<u8>>>,
}

impl Write for MockWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut data = self.data.borrow_mut();
        data.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct MockReader {
    index: usize,
    data: Rc<RefCell<Vec<u8>>>,
}

impl Read for MockReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let data = self.data.borrow();
        let read_slice = &(*data)[self.index..];
        let read_len = usize::min(buf.len(), read_slice.len());
        if read_len > 0 {
            buf[0..read_len].copy_from_slice(&read_slice[0..read_len]);
        }
        self.index += read_len;
        Ok(read_len)
    }
}

#[derive(Debug, Default)]
pub struct MockFSDirectoryEntry {
    pub entries: BTreeMap<OsString, MockFSEntry>,
}

impl MockFSDirectoryEntry {
    pub fn get_or_create_dir(&mut self, pc: &OsStr) -> Result<&mut MockFSDirectoryEntry> {
        self.entries
            .entry(OsString::from(pc))
            .or_insert_with(|| MockFSEntry::Directory(MockFSDirectoryEntry::default()))
            .as_dir_mut()
    }

    pub fn create_file(
        &mut self,
        pc: &OsStr,
        contents: Rc<RefCell<Vec<u8>>>,
    ) -> Result<&MockFSFileEntry> {
        if self.entries.contains_key(pc) {
            return AlreadyExistsSnafu {
                path: PathBuf::from(pc),
            }.fail();
        }

        self.entries.insert(
            OsString::from(pc),
            MockFSEntry::File(MockFSFileEntry { contents }),
        );
        self.entries.get(pc).unwrap().as_file()
    }

    pub fn create_dir(&mut self, pc: &OsStr) -> Result<&MockFSDirectoryEntry> {
        if self.entries.contains_key(pc) {
            return AlreadyExistsSnafu {
                path: PathBuf::from(pc),
            }.fail();
        }

        self.entries.insert(
            OsString::from(pc),
            MockFSEntry::Directory(MockFSDirectoryEntry::default()),
        );
        self.entries.get(pc).unwrap().as_dir()
    }
}

#[derive(Debug)]
pub struct MockFSFileEntry {
    pub contents: Rc<RefCell<Vec<u8>>>,
}

#[derive(Debug)]
pub enum MockFSEntry {
    Directory(MockFSDirectoryEntry),
    File(MockFSFileEntry),
}

impl MockFSEntry {
    pub fn as_dir(&self) -> Result<&MockFSDirectoryEntry> {
        match self {
            MockFSEntry::Directory(d) => Ok(d),
            MockFSEntry::File(_) => NotADirectorySnafu {
                path: PathBuf::from(""),
            }.fail(),
        }
    }

    pub fn as_dir_mut(&mut self) -> Result<&mut MockFSDirectoryEntry> {
        match self {
            MockFSEntry::Directory(d) => Ok(d),
            MockFSEntry::File(_) => NotADirectorySnafu {
                path: PathBuf::from(""),
            }.fail(),
        }
    }

    pub fn as_file(&self) -> Result<&MockFSFileEntry> {
        match self {
            MockFSEntry::Directory(_) => NotAFileSnafu {
                path: PathBuf::from(""),
            }.fail(),
            MockFSEntry::File(f) => Ok(f),
        }
    }

    pub fn child(&self, pc: &OsStr) -> Result<&MockFSEntry> {
        self.as_dir()?.entries.get(pc).ok_or_else(|| {
            XfsError::NotFound {
                path: PathBuf::from(pc),
            }
        })
    }

    pub fn child_mut(&mut self, pc: &OsStr) -> Result<&mut MockFSEntry> {
        self.as_dir_mut()?.entries.get_mut(pc).ok_or_else(|| {
            XfsError::NotFound {
                path: PathBuf::from(pc),
            }
        })
    }

    fn metadata(&self) -> MockMetadata {
        match self {
            MockFSEntry::Directory(_) => MockMetadata {
                is_file: false,
                is_dir: true,
            },
            MockFSEntry::File(_) => MockMetadata {
                is_file: true,
                is_dir: false,
            },
        }
    }
}

#[derive(Debug)]
pub struct MockFS {
    pub root: MockFSEntry,
}

impl MockFS {
    pub fn new() -> MockFS {
        MockFS {
            root: MockFSEntry::Directory(MockFSDirectoryEntry::default()),
        }
    }

    fn normalize_path(p: &Path) -> Result<Vec<&OsStr>> {
        let mut result = vec![];
        for pc in p.components() {
            match pc {
                std::path::Component::Prefix(_) => {}
                std::path::Component::RootDir => {
                    result = vec![];
                }
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    result.pop().ok_or_else(|| {
                        XfsError::PathOutsideSandbox {
                            path: p.to_path_buf(),
                        }
                    })?;
                }
                std::path::Component::Normal(c) => {
                    result.push(c);
                }
            }
        }
        Ok(result)
    }

    pub fn add_r(&mut self, p: &Path, contents: Vec<u8>) -> Result<()> {
        let p_comp: Vec<&OsStr> = Self::normalize_path(p)?;
        if p_comp.is_empty() {
            return Ok(());
        }

        let mut dir = self.root.as_dir_mut().map_err(|_| XfsError::NotADirectory { path: PathBuf::from("/") })?;
        for pc in &p_comp[..p_comp.len() - 1] {
            dir = dir.get_or_create_dir(pc)?;
        }
        let pc = p_comp[p_comp.len() - 1];
        let contents = Rc::new(RefCell::new(contents));
        dir.create_file(pc, contents)?;
        Ok(())
    }

    pub fn add_file<S: AsRef<str>>(&mut self, p: &Path, contents: S) -> Result<()> {
        self.add_r(p, contents.as_ref().as_bytes().to_vec())
    }

    pub fn get(&self, p: &Path) -> Result<Vec<u8>> {
        let f = self
            .resolve_path(p)?
            .as_file().map_err(|_| XfsError::NotAFile { path: p.to_path_buf() })?;
        Ok(f.contents.borrow().clone())
    }

    pub fn resolve_path(&self, p: &Path) -> Result<&MockFSEntry> {
        let mut result = &self.root;
        for pc in Self::normalize_path(p)? {
            result = result.child(pc).map_err(|_| XfsError::NotFound {
                path: p.to_path_buf(),
            })?;
        }
        Ok(result)
    }

    pub fn resolve_path_mut(&mut self, p: &Path) -> Result<&mut MockFSEntry> {
        let mut result = &mut self.root;
        for pc in Self::normalize_path(p)? {
            result = result.child_mut(pc).map_err(|_| XfsError::NotFound {
                path: p.to_path_buf(),
            })?;
        }
        Ok(result)
    }

    pub fn tree(&self) -> String {
        Self::tree_(&OsString::from("/"), &self.root, "")
    }

    fn tree_(pc: &OsStr, e: &MockFSEntry, prefix: &str) -> String {
        match e {
            MockFSEntry::Directory(d) => {
                let mut s = format!("{}{:?}/\n", prefix, pc);
                let new_prefix = format!("  {}", prefix);
                for (k, v) in &d.entries {
                    s = format!("{}{}", s, Self::tree_(k, v, &new_prefix));
                }
                s
            }
            MockFSEntry::File(f) => {
                let data = f.contents.borrow();
                match std::str::from_utf8(data.as_slice()) {
                    Ok(s) => format!("{}{:?} => {:?}\n", prefix, pc, s),
                    Err(_) => format!("{}{:?} => BINARY DATA\n", prefix, pc),
                }
            }
        }
    }

    pub fn copy_recursive(
        &mut self,
        other_fs: &dyn Xfs,
        other_path: &Path,
        self_path: &Path,
    ) -> Result<()> {
        let md = other_fs.metadata(other_path)?;

        let self_md = self.metadata(self_path);
        if md.is_file() {
            let mod_self_path = if let Ok(self_md) = self_md {
                if self_md.is_dir() {
                    self_path.join(other_path.file_name().unwrap())
                } else {
                    return AlreadyExistsSnafu {
                        path: self_path.to_path_buf(),
                    }.fail();
                }
            } else {
                // It doesn't exist we can just write to it
                PathBuf::from(self_path)
            };
            let mut r = other_fs.reader(other_path)?;
            let mut w = self.writer(&mod_self_path)?;
            std::io::copy(&mut r, &mut w).map_err(|e| {
                XfsError::IoError {
                    path: mod_self_path.clone(),
                    source: e,
                }
            })?;
        } else {
            if let Ok(self_md) = self_md {
                if !self_md.is_dir() {
                    return GeneralSnafu {
                        message: format!("mockfs::copy_recursive creating directory {} but already exists as file", self_path.display()),
                    }.fail();
                }
            } else {
                // If it doesn't exist we need to create it
                self.create_dir(self_path)?;
            };

            for de in other_fs.read_dir(other_path)? {
                let de = de?;
                let self_child_path = self_path.join(de.path().file_name().unwrap());
                self.copy_recursive(other_fs, &de.path(), &self_child_path)?;
            }
        }

        Ok(())
    }
}

impl Default for MockFS {
    fn default() -> Self {
        Self::new()
    }
}

struct MockDirEntry {
    path: PathBuf,
    metadata: MockMetadata,
}

impl XfsDirEntry for MockDirEntry {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn metadata(&self) -> Result<Box<dyn XfsMetadata>> {
        Ok(Box::new(self.metadata.clone()))
    }
}

#[derive(Clone, Debug)]
struct MockMetadata {
    is_file: bool,
    is_dir: bool,
}

impl XfsMetadata for MockMetadata {
    fn is_dir(&self) -> bool {
        self.is_dir
    }

    fn is_file(&self) -> bool {
        self.is_file
    }
}

impl Xfs for MockFS {
    fn read_dir(&self, p: &Path) -> Result<XfsReadDir> {
        let dir = self
            .resolve_path(p)
            .map_err(|_| XfsError::NotFound { path: p.to_path_buf() })?
            .as_dir().map_err(|_| XfsError::NotADirectory { path: p.to_path_buf() })?;

        let entries: Vec<Result<Box<dyn XfsDirEntry>>> = dir.entries.iter().map(|(k, v)| {
            let entry: Box<dyn XfsDirEntry> = Box::new(MockDirEntry {
                path: p.join(k),
                metadata: v.metadata(),
            });
            Ok(entry)
        }).collect();

        Ok(Box::new(entries.into_iter()))
    }

    fn reader(&self, p: &Path) -> Result<Box<dyn std::io::Read>> {
        let f = self
            .resolve_path(p)
            .map_err(|_| XfsError::NotFound { path: p.to_path_buf() })?
            .as_file().map_err(|_| XfsError::NotAFile { path: p.to_path_buf() })?;

        let r = MockReader {
            index: 0,
            data: f.contents.clone(),
        };
        Ok(Box::new(r))
    }

    fn writer(&mut self, p: &Path) -> Result<Box<dyn std::io::Write>> {
        let pp = p.parent().ok_or_else(|| {
            XfsError::NotFound {
                path: p.to_path_buf(),
            }
        })?;
        let parent_dir = self.resolve_path_mut(pp).map_err(|_| XfsError::NotFound { path: pp.to_path_buf() })?
            .as_dir_mut().map_err(|_| XfsError::NotADirectory { path: pp.to_path_buf() })?;

        let data = Rc::new(RefCell::new(Vec::new()));
        parent_dir.create_file(p.file_name().unwrap(), data.clone())?;

        let w = MockWriter { data };
        Ok(Box::new(w))
    }

    fn create_dir(&mut self, p: &Path) -> Result<()> {
        // The root always exists, so we can't create it
        if p.as_os_str().is_empty() || p == Path::new("/") {
            return AlreadyExistsSnafu {
                path: p.to_path_buf(),
            }.fail();
        }

        let pp = p.parent().ok_or_else(|| {
            XfsError::NotFound {
                path: p.to_path_buf(),
            }
        })?;
        let parent_dir = self
            .resolve_path_mut(pp)
            .map_err(|_| XfsError::NotFound { path: pp.to_path_buf() })?
            .as_dir_mut().map_err(|_| XfsError::NotADirectory { path: pp.to_path_buf() })?;
        parent_dir.create_dir(p.file_name().unwrap())?;
        Ok(())
    }

    fn create_dir_all(&mut self, p: &Path) -> Result<()> {
        let p_comp: Vec<&OsStr> = Self::normalize_path(p)?;
        let mut root = self.root.as_dir_mut().map_err(|_| XfsError::NotADirectory { path: PathBuf::from("/") })?;
        for pc in p_comp {
            root = root.get_or_create_dir(pc)?;
        }
        Ok(())
    }

    fn read_all_lines(&self, p: &Path) -> Result<Vec<String>> {
        let file = self
            .resolve_path(p)
            .map_err(|_| XfsError::NotFound { path: p.to_path_buf() })?
            .as_file().map_err(|_| XfsError::NotAFile { path: p.to_path_buf() })?;
        let data = file.contents.borrow();

        let s = std::str::from_utf8(data.as_slice())
            .map_err(|_| XfsError::InvalidUtf8 {
                path: p.to_path_buf(),
            })?;
        Ok(s.lines().map(|s| s.to_string()).collect())
    }

    fn metadata(&self, p: &Path) -> Result<Box<dyn XfsMetadata>> {
        let entry = self
            .resolve_path(p)
            .map_err(|_| XfsError::NotFound { path: p.to_path_buf() })?;
        Ok(Box::new(entry.metadata()))
    }
}
