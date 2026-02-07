# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.1.3]
### Added
- `MockFS::get_str` method for reading file contents as UTF-8 strings.

### Changed
- Internal refactoring of `MockFS` to improve thread safety and simplify path resolution.
- `MockFSEntry` now uses internal mutability (Arc/RwLock) more consistently, reducing the need for top-level locks.
- Refactored `resolve_path` to be a method of `MockFS` rather than a static function.

## [0.1.2]
### Added
- `XfsReadOnly` trait to provide a read-only interface to the filesystem.
- `unsafe_clone` (in `XfsReadOnly`) and `unsafe_clone_mut` (in `Xfs`) methods for obtaining new handles to the same underlying filesystem.
- `remove_file`, `remove_dir_all`, and `rename` methods to the `Xfs` trait.
- Dedicated `tests/read_only.rs` for verifying read-only constraints.
- `Send` bound added to `XfsReadOnly` and `Xfs` traits.

### Changed
- Refactored `Xfs` trait to inherit from `XfsReadOnly`, separating mutation methods from read-only operations.
- Enhanced documentation and added comprehensive usage examples in README.
- Improved test organization and granularity in `tests/fs_tests.rs`.
- `MockFS` internal implementation now uses `Arc` and `RwLock` to enable thread-safe shared access.

## [0.1.1]
### Added
- Comprehensive `XfsError` variants using `snafu` for better error context (e.g., `IoError`, `NotFound`, `NotADirectory`).
- Iterator-based `read_dir` API, replacing the previous callback-based approach.

### Changed
- Switched error handling from `thiserror` to `snafu`.
- Refined trait methods to return `Result<T>` instead of relying on `anyhow`.

## [0.1.0]
- Initial release featuring the `Xfs` trait with basic operations.
- Implementations for the real operating system filesystem (`OsFs`) and an in-memory mock filesystem (`MockFS`).
