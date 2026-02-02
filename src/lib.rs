// use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use thiserror::Error;

pub mod mockfs;

#[derive(Error, Debug)]
pub enum XfsError {
    #[error("IO error: {0}")]
    GeneralIOError(String, #[source] std::io::Error),

    #[error("Invalid type {0}")]
    InvalidType(String),

    #[error("unspecified error: {0}")]
    UnspecifiedError(String),

    #[error("general error: {0}")]
    GeneralError(
        String,
        #[source] Box<dyn std::error::Error + Send + Sync + 'static>,
    ),
}

pub trait XfsErrorContext<T> {
    fn xfs_error<C, F>(self, f: F) -> Result<T>
    where
        C: core::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> XfsErrorContext<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn xfs_error<C, F>(self, f: F) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(XfsError::GeneralError(format!("{}", f()), Box::new(e))),
        }
    }
}

pub trait XfsDirEntry {
    fn path(&self) -> PathBuf;
    fn metadata(&self) -> Result<Box<dyn XfsMetadata>>;
}

pub trait XfsMetadata {
    fn is_dir(&self) -> bool;
    fn is_file(&self) -> bool;
}

pub type Result<T> = std::result::Result<T, XfsError>;

pub trait Xfs {
    fn on_each_entry(
        &self,
        p: &Path,
        f: &mut dyn FnMut(&dyn Xfs, &dyn XfsDirEntry) -> anyhow::Result<()>,
    ) -> anyhow::Result<()>;

    fn on_each_entry_mut(
        &mut self,
        p: &Path,
        f: &mut dyn FnMut(&mut dyn Xfs, &dyn XfsDirEntry) -> anyhow::Result<()>,
    ) -> anyhow::Result<()>;

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
        let md = std::fs::DirEntry::metadata(self).map_err(|e| {
            XfsError::GeneralIOError(
                "XfsDirEntry::metadata unable to get metadata".to_string(),
                e,
            )
        })?;
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
    fn on_each_entry(
        &self,
        p: &Path,
        f: &mut dyn FnMut(&dyn Xfs, &dyn XfsDirEntry) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        for e in std::fs::read_dir(p).map_err(|e| {
            XfsError::GeneralIOError(format!("osfs::on_each_entry({}) failed", p.display()), e)
        })? {
            f(self, &e?)?
        }
        Ok(())
    }

    fn on_each_entry_mut(
        &mut self,
        p: &Path,
        f: &mut dyn FnMut(&mut dyn Xfs, &dyn XfsDirEntry) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        for e in std::fs::read_dir(p).map_err(|e| {
            XfsError::GeneralIOError(
                format!("osfs::on_each_entry_mut({}) failed", p.display()),
                e,
            )
        })? {
            f(self, &e?)?
        }
        Ok(())
    }

    fn writer(&mut self, p: &Path) -> Result<Box<dyn Write>> {
        Ok(Box::new(BufWriter::new(std::fs::File::create(p).map_err(
            |e| XfsError::GeneralIOError(format!("osfs::writer({}) failed", p.display()), e),
        )?)))
    }

    fn reader(&self, p: &Path) -> Result<Box<dyn Read>> {
        Ok(Box::new(BufReader::new(std::fs::File::open(p).map_err(
            |e| XfsError::GeneralIOError(format!("osfs::reader({}) failed", p.display()), e),
        )?)))
    }

    fn create_dir(&mut self, p: &Path) -> Result<()> {
        std::fs::create_dir(p).map_err(|e| {
            XfsError::GeneralIOError(format!("osfs::create_dir({}) failed", p.display()), e)
        })?;
        Ok(())
    }

    fn create_dir_all(&mut self, p: &Path) -> Result<()> {
        std::fs::create_dir_all(p).map_err(|e| {
            XfsError::GeneralIOError(format!("osfs::create_dir_all({}) failed", p.display()), e)
        })?;
        Ok(())
    }

    fn read_all_lines(&self, p: &Path) -> Result<Vec<String>> {
        let file = std::fs::File::open(p).map_err(|e| {
            XfsError::GeneralIOError(
                format!("osfs::read_all_lines({}) unable to open file", p.display()),
                e,
            )
        })?;
        let v: std::io::Result<Vec<_>> = BufReader::new(file).lines().collect();
        v.map_err(|e| {
            XfsError::GeneralIOError(
                format!(
                    "osfs::read_all_lines({}) error while reading lines",
                    p.display()
                ),
                e,
            )
        })
    }

    fn metadata(&self, p: &Path) -> Result<Box<dyn XfsMetadata>> {
        std::fs::metadata(p)
            .map(|m| {
                let b: Box<dyn XfsMetadata> = Box::new(m);
                b
            })
            .map_err(|e| {
                XfsError::GeneralIOError(format!("osfs::metadata({}) failed", p.display()), e)
            })
    }
}
