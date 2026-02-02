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

    #[snafu(display("Invalid type: {}", message))]
    InvalidType { message: String },

    #[snafu(display("Error at {}: {}", path.display(), message))]
    PathError { path: PathBuf, message: String },

    #[snafu(display("General error: {}", message))]
    GeneralError { message: String },

    #[snafu(display("User error: {}", source))]
    UserError {
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

pub type Result<T> = std::result::Result<T, XfsError>;

pub trait XfsDirEntry {
    fn path(&self) -> PathBuf;
    fn metadata(&self) -> Result<Box<dyn XfsMetadata>>;
}

pub trait XfsMetadata {
    fn is_dir(&self) -> bool;
    fn is_file(&self) -> bool;
}

pub trait Xfs {
    fn read_dir<'a>(
        &'a self,
        p: &Path,
    ) -> Result<Box<dyn Iterator<Item = Result<Box<dyn XfsDirEntry>>> + 'a>>;

    fn reader(&self, p: &Path) -> Result<Box<dyn Read>>;
    fn writer(&mut self, p: &Path) -> Result<Box<dyn Write>>;

    fn create_dir(&mut self, p: &Path) -> Result<()>;

    fn create_dir_all(&mut self, p: &Path) -> Result<()>;

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
    fn read_dir<'a>(
        &'a self,
        p: &Path,
    ) -> Result<Box<dyn Iterator<Item = Result<Box<dyn XfsDirEntry>>> + 'a>> {
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
