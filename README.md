# inscenerator-xfs

`inscenerator-xfs` is a Rust library providing a filesystem abstraction layer. It defines a common interface for
filesystem operations, allowing you to write code that can run against the real operating system filesystem or an in-memory 
mock filesystem.

This is particularly useful for unit testing code that performs filesystem operations without actually hitting the disk.

## Traits

The library provides two main traits:

- **`XfsReadOnly`**: Contains methods for read-only operations like `reader`, `read_dir`, and `metadata`. It also provides `unsafe_clone()` which returns a new read-only handle.
- **`Xfs`**: Inherits from `XfsReadOnly` and adds methods for mutations like `writer`, `create_dir`, and `remove_file`. It provides `unsafe_clone_mut()` which returns a new mutable handle.

This separation ensures that a read-only reference (`&dyn XfsReadOnly` or `&dyn Xfs`) cannot be used to obtain a mutable handle at compile-time.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
inscenerator-xfs = { git = "https://github.com/mikeando/inscenerator-xfs" }
```

## Usage

### The Problem: Hard-to-test Filesystem Code

Consider a function that reads a configuration file and performs some action:

```rust
use std::fs;
use std::path::Path;

fn process_config(path: &Path) -> Result<(), std::io::Error> {
    let content = fs::read_to_string(path)?;
    println!("Config content: {}", content);
    // ... do something with content
    Ok(())
}
```

Testing this function requires creating an actual file on disk, which can be slow, prone to side effects, and requires cleanup.

### The Solution: Using `inscenerator-xfs`

Refactor your function to accept an implementation of the `XfsReadOnly` trait:

```rust
use inscenerator_xfs::XfsReadOnly;
use std::path::Path;

fn process_config(fs: &dyn XfsReadOnly, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let lines = fs.read_all_lines(path)?;
    for line in lines {
        println!("Config line: {}", line);
    }
    Ok(())
}

// In production:
// use inscenerator_xfs::OsFs;
// let fs = OsFs {};
// process_config(&fs, Path::new("config.txt")).unwrap();
```

Now you can easily test it using `MockFS`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use inscenerator_xfs::mockfs::MockFS;
    use std::path::Path;

    #[test]
    fn test_process_config() {
        let mut fs = MockFS::new();
        fs.add_file(Path::new("test_config.txt"), "line1\nline2").unwrap();

        let result = process_config(&fs, Path::new("test_config.txt"));
        assert!(result.is_ok());
    }
}
```

### Thread Safety and Cloning

Both `OsFs` and `MockFS` support cloning to provide multiple handles to the same underlying filesystem. This is useful for multi-threaded operations.

- Use `fs.unsafe_clone()` to get a read-only handle from a shared reference.
- Use `fs.unsafe_clone_mut()` to get a mutable handle from a mutable reference.

```rust
use std::thread;
use inscenerator_xfs::{Xfs, mockfs::MockFS};
use std::path::Path;
use std::io::Write;

let mut fs = MockFS::new();
let mut fs_clone = fs.unsafe_clone_mut();

thread::spawn(move || {
    let mut w = fs_clone.writer(Path::new("file.txt")).unwrap();
    w.write_all(b"hello from thread").unwrap();
}).join().unwrap();
```

Note: These are called "unsafe" because they do not provide protection against concurrent mutations to the *same* paths. It is up to the user to coordinate access to disjoint parts of the directory tree.

## Features

- **Trait-based Abstraction**: `XfsReadOnly` and `Xfs` traits for flexible filesystem access.
- **`OsFs`**: A wrapper around `std::fs` for real filesystem access.
- **`MockFS`**: An in-memory filesystem implementation for testing.

## Future Plans

- Better support for in-memory filesystems.
- Support for archive filesystems (e.g., ZIP, TAR).

## About

`inscenerator-xfs` is a component of the Inscenerator project.

## License

This project is licensed under the MIT License.
