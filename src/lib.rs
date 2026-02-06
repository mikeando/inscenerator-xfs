use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use snafu::{ResultExt, Snafu};

pub mod mockfs;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum XfsError {
    #[snafu(display("IO error at {}: {}", path.display(), source))]
    IoError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("File not found at {}", path.display()))]
    NotFound { path: PathBuf },

    #[snafu(display("File or directory already exists at {}", path.display()))]
    AlreadyExists { path: PathBuf },

    #[snafu(display("Path is not a directory: {}", path.display()))]
    NotADirectory { path: PathBuf },

    #[snafu(display("Path is not a file: {}", path.display()))]
    NotAFile { path: PathBuf },

    #[snafu(display("Path steps outside the sandbox: {}", path.display()))]
    PathOutsideSandbox { path: PathBuf },

    #[snafu(display("Invalid UTF-8 in file {}", path.display()))]
    InvalidUtf8 { path: PathBuf },

    #[snafu(display("General error: {}", message))]
    GeneralError { message: String },

    #[snafu(display("User error: {}", source))]
    UserError {
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

pub type Result<T> = std::result::Result<T, XfsError>;

/// A result type for a single directory entry.
pub type XfsEntryResult = Result<Box<dyn XfsDirEntry>>;

/// An iterator over directory entries.
pub type XfsReadDir = Box<dyn Iterator<Item = XfsEntryResult>>;

pub trait XfsDirEntry {
    fn path(&self) -> PathBuf;
    fn metadata(&self) -> Result<Box<dyn XfsMetadata>>;
}

pub trait XfsMetadata {
    fn is_dir(&self) -> bool;
    fn is_file(&self) -> bool;
}

pub trait Xfs: Send {
    /// Creates a new handle to the same underlying filesystem.
    ///
    /// # Safety
    ///
    /// This is named `unsafe_clone` because it breaks the normal Rust expectation
    /// that a clone is an independent copy. Here, any mutation performed on the
    /// clone will be visible to the original and all other clones.
    ///
    /// The returned object is `Send`, allowing it to be moved to another thread
    /// to perform concurrent operations on the same filesystem.
    fn unsafe_clone(&self) -> Box<dyn Xfs + Send>;

    /// Returns an iterator over the entries within a directory.
    ///
    /// The iterator does not borrow the filesystem object, allowing
    /// for mutation of the filesystem during iteration.
    ///
    /// # Errors
    ///
    /// Returns an error if the path does not exist or is not a directory.
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::Path;
    /// # use inscenerator_xfs::{Xfs, mockfs::MockFS};
    /// # let mut fs = MockFS::new();
    /// # fs.add_file(Path::new("a.txt"), "content").unwrap();
    /// for entry in fs.read_dir(Path::new(".")).unwrap() {
    ///     let entry = entry.unwrap();
    ///     println!("{:?}", entry.path());
    ///     // Mutation is allowed during iteration
    ///     fs.create_dir_all(Path::new("new_dir")).unwrap();
    /// }
    /// ```
    fn read_dir(&self, p: &Path) -> Result<XfsReadDir>;

    fn reader(&self, p: &Path) -> Result<Box<dyn Read>>;
    fn writer(&mut self, p: &Path) -> Result<Box<dyn Write>>;

    fn create_dir(&mut self, p: &Path) -> Result<()>;

    fn create_dir_all(&mut self, p: &Path) -> Result<()>;

    /// Deletes a single file.
    ///
    /// # Errors
    ///
    /// Returns an error if the path does not exist, is a directory, or
    /// if there is an IO error.
    fn remove_file(&mut self, p: &Path) -> Result<()>;

    /// Deletes a directory and all its contents.
    ///
    /// # Errors
    ///
    /// Returns an error if the path does not exist, is not a directory, or
    /// if there is an IO error.
    fn remove_dir_all(&mut self, p: &Path) -> Result<()>;

    /// Renames or moves a file or directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the source path does not exist, or if there is
    /// an IO error.
    fn rename(&mut self, from: &Path, to: &Path) -> Result<()>;

    fn read_all_lines(&self, p: &Path) -> Result<Vec<String>>;

    fn metadata(&self, p: &Path) -> Result<Box<dyn XfsMetadata>>;

    /// IO Errors are treated as-if the file does not exist.
    fn exists(&self, p: &Path) -> bool {
        self.metadata(p).is_ok()
    }

    /// IO Errors are treated as-if the path is not a directory.
    fn is_dir(&self, p: &Path) -> bool {
        self.metadata(p).map(|md| md.is_dir()).unwrap_or(false)
    }

    /// IO Errors are treated as-if the path is not a file.
    fn is_file(&self, p: &Path) -> bool {
        self.metadata(p).map(|md| md.is_file()).unwrap_or(false)
    }
}

pub struct OsFs {}

impl XfsDirEntry for std::fs::DirEntry {
    fn path(&self) -> PathBuf {
        std::fs::DirEntry::path(self)
    }

    fn metadata(&self) -> Result<Box<dyn XfsMetadata>> {
        let md = std::fs::DirEntry::metadata(self).context(IoSnafu { path: self.path() })?;
        Ok(Box::new(md))
    }
}

impl XfsMetadata for std::fs::Metadata {
    fn is_dir(&self) -> bool {
        std::fs::Metadata::is_dir(self)
    }

    fn is_file(&self) -> bool {
        std::fs::Metadata::is_file(self)
    }
}

impl Xfs for OsFs {
    fn unsafe_clone(&self) -> Box<dyn Xfs + Send> {
        Box::new(OsFs {})
    }

    fn read_dir(&self, p: &Path) -> Result<XfsReadDir> {
        let path_buf = p.to_path_buf();
        let read_dir = std::fs::read_dir(p).context(IoSnafu { path: p })?;
        let iter = read_dir.map(move |entry| {
            let entry = entry.context(IoSnafu { path: &path_buf })?;
            let entry: Box<dyn XfsDirEntry> = Box::new(entry);
            Ok(entry)
        });
        Ok(Box::new(iter))
    }

    fn writer(&mut self, p: &Path) -> Result<Box<dyn Write>> {
        let file = std::fs::File::create(p).context(IoSnafu { path: p })?;
        Ok(Box::new(BufWriter::new(file)))
    }

    fn reader(&self, p: &Path) -> Result<Box<dyn Read>> {
        let file = std::fs::File::open(p).context(IoSnafu { path: p })?;
        Ok(Box::new(BufReader::new(file)))
    }

    fn create_dir(&mut self, p: &Path) -> Result<()> {
        std::fs::create_dir(p).context(IoSnafu { path: p })?;
        Ok(())
    }

    fn create_dir_all(&mut self, p: &Path) -> Result<()> {
        std::fs::create_dir_all(p).context(IoSnafu { path: p })?;
        Ok(())
    }

    fn remove_file(&mut self, p: &Path) -> Result<()> {
        std::fs::remove_file(p).context(IoSnafu { path: p })?;
        Ok(())
    }

    fn remove_dir_all(&mut self, p: &Path) -> Result<()> {
        std::fs::remove_dir_all(p).context(IoSnafu { path: p })?;
        Ok(())
    }

    fn rename(&mut self, from: &Path, to: &Path) -> Result<()> {
        std::fs::rename(from, to).context(IoSnafu { path: from })?;
        Ok(())
    }

    fn read_all_lines(&self, p: &Path) -> Result<Vec<String>> {
        let file = std::fs::File::open(p).context(IoSnafu { path: p })?;
        let lines: std::io::Result<Vec<_>> = BufReader::new(file).lines().collect();
        lines.context(IoSnafu { path: p })
    }

    fn metadata(&self, p: &Path) -> Result<Box<dyn XfsMetadata>> {
        let m = std::fs::metadata(p).context(IoSnafu { path: p })?;
        Ok(Box::new(m))
    }
}
