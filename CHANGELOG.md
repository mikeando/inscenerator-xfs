# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.1.3]
### Added
- `MockFS::get_str` for easier reading of UTF-8 file contents.

### Changed
- `MockFS` interface update: key methods (such as `resolve_path`, `as_dir`, and `as_file`) now return owned objects rather than references.
- Improved internal path resolution and thread safety in `MockFS`.

## [0.1.2]
### Added
- `XfsReadOnly` trait to separate read-only access from mutable operations.
- `unsafe_clone` and `unsafe_clone_mut` methods for sharing filesystem handles.
- New `Xfs` operations: `remove_file`, `remove_dir_all`, and `rename`.
- `Send` bounds on traits to support multi-threaded use.

### Changed
- Major documentation update in README with comprehensive usage examples.
- `MockFS` now uses internal mutability to support thread-safe shared access.

## [0.1.1]
### Added
- Specific error variants and context using `snafu` (e.g., `NotFound`, `NotADirectory`).
- **Iterator-based `read_dir` API**, replacing the previous callback-based approach for directory traversal.

### Changed
- Migrated error handling to `snafu` and standardized on a project-wide `Result` type.

## [0.1.0]
- Initial release featuring the `Xfs` filesystem abstraction.
- Includes `OsFs` for real disk access and `MockFS` for in-memory testing.
