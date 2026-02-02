# inscenerator-xfs

`inscenerator-xfs` is a Rust library providing a filesystem abstraction layer. It defines a common interface (`Xfs` trait) for filesystem operations, allowing you to write code that can run against the real operating system filesystem or an in-memory mock filesystem.

This is particularly useful for unit testing code that performs filesystem operations without actually hitting the disk.

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

Refactor your function to accept an implementation of the `Xfs` trait:

```rust
use inscenerator_xfs::Xfs;
use std::path::Path;

fn process_config(fs: &dyn Xfs, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
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

## Features

- **`Xfs` Trait**: A common interface for filesystem operations like reading, writing, and directory traversal.
- **`OsFs`**: A wrapper around `std::fs` for real filesystem access.
- **`MockFS`**: An in-memory filesystem implementation for testing.

## Future Plans

- Better support for in-memory filesystems.
- Support for archive filesystems (e.g., ZIP, TAR).

## About

`inscenerator-xfs` is a component of the Inscenerator project.

## License

This project is licensed under the MIT License.
