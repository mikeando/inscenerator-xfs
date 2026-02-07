# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.1.3]
### Added
- `MockFS::get_str` for easier reading of UTF-8 file contents.

### Changed
- Improved thread safety and internal path resolution in `MockFS`.

## [0.1.2]
### Added
- `XfsReadOnly` trait to provide a read-only interface, with `Xfs` now inheriting from it to enforce mutability constraints.
- `unsafe_clone` and `unsafe_clone_mut` methods for sharing filesystem handles across threads.
- `remove_file`, `remove_dir_all`, and `rename` operations to the `Xfs` trait.
- `Send` bounds on traits to support multi-threaded applications.

### Changed
- Major documentation update in README with comprehensive usage examples.
- Internal refactoring of `MockFS` to use shared state (Arc/RwLock), enabling concurrent access.

## [0.1.1]
### Added
- Descriptive error variants and context using `snafu` (e.g., `NotFound`, `NotADirectory`).
- Iterator-based `read_dir` API for easier directory traversal, replacing the previous callback-based approach.

### Changed
- Migrated error handling from `thiserror` to `snafu` and standardized on a custom `Result` type.

## [0.1.0]
- Initial release featuring the `Xfs` filesystem abstraction.
- Includes `OsFs` for real disk access and `MockFS` for in-memory testing.
