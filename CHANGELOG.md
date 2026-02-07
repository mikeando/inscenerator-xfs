# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.1.3]
### Added
- `MockFS::get_str` for easier reading of UTF-8 file contents.

### Changed
- Refactored `MockFS` methods (including `resolve_path`, `as_dir`, and `as_file`) to return owned objects rather than references, simplifying multi-threaded access.
- Improved internal path resolution and thread safety across `MockFS`.

## [0.1.2]
### Added
- `XfsReadOnly` trait for read-only filesystem access, with `Xfs` inheriting from it to separate mutable operations.
- `unsafe_clone` and `unsafe_clone_mut` methods for sharing handles between threads.
- New `Xfs` operations: `remove_file`, `remove_dir_all`, and `rename`.
- `Send` bounds for traits to support concurrent applications.

### Changed
- Comprehensive documentation update in README with usage examples.
- Internal `MockFS` refactor using `Arc` and `RwLock` for thread-safe shared state.

## [0.1.1]
### Added
- Richer error context via `snafu`, introducing specific variants like `NotFound` and `NotADirectory`.
- **Iterator-based `read_dir` API**, replacing the previous callback-based approach for more idiomatic directory traversal.

### Changed
- Migrated from `thiserror` to `snafu` and introduced a project-wide `Result` type.

## [0.1.0]
- Initial release featuring the `Xfs` filesystem abstraction.
- Includes `OsFs` (standard library wrapper) and `MockFS` (in-memory implementation).
