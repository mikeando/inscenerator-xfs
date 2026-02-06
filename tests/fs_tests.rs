use inscenerator_xfs::{OsFs, Xfs};
use inscenerator_xfs::mockfs::MockFS;
use std::path::Path;
use std::io::{Read, Write};

#[test]
fn test_mockfs_basic() {
    let fs = MockFS::new();
    let path = Path::new("test.txt");
    let content = "hello world";

    fs.add_file(path, content).unwrap();

    let mut reader = fs.reader(path).unwrap();
    let mut buf = String::new();
    reader.read_to_string(&mut buf).unwrap();
    assert_eq!(buf, content);
}

#[test]
fn test_mockfs_multithreaded() {
    use std::sync::Arc;
    use std::thread;

    let fs = Arc::new(MockFS::new());
    let mut handles = vec![];

    for i in 0..10 {
        let fs_clone = Arc::clone(&fs);
        let handle = thread::spawn(move || {
            let path_str = format!("file_{}.txt", i);
            let path = Path::new(&path_str);
            let content = format!("content {}", i);
            fs_clone.add_file(path, &content).unwrap();

            let mut reader = fs_clone.reader(path).unwrap();
            let mut buf = String::new();
            reader.read_to_string(&mut buf).unwrap();
            assert_eq!(buf, content);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all files exist
    for i in 0..10 {
        let path_str = format!("file_{}.txt", i);
        assert!(fs.is_file(Path::new(&path_str)));
    }
}

#[test]
fn test_mockfs_remove_file() {
    let fs = MockFS::new();
    let path = Path::new("test.txt");
    fs.add_file(path, "hello").unwrap();
    assert!(fs.exists(path));
    fs.remove_file(path).unwrap();
    assert!(!fs.exists(path));
}

#[test]
fn test_mockfs_remove_dir_all() {
    let fs = MockFS::new();
    fs.add_file(Path::new("dir/a.txt"), "a").unwrap();
    fs.add_file(Path::new("dir/subdir/b.txt"), "b").unwrap();
    assert!(fs.is_dir(Path::new("dir")));
    fs.remove_dir_all(Path::new("dir")).unwrap();
    assert!(!fs.exists(Path::new("dir")));
    assert!(!fs.exists(Path::new("dir/a.txt")));
}

#[test]
fn test_mockfs_rename() {
    let fs = MockFS::new();
    fs.add_file(Path::new("old.txt"), "content").unwrap();
    fs.rename(Path::new("old.txt"), Path::new("new.txt")).unwrap();
    assert!(!fs.exists(Path::new("old.txt")));
    assert!(fs.is_file(Path::new("new.txt")));
    assert_eq!(fs.get(Path::new("new.txt")).unwrap(), b"content");
}

#[test]
fn test_osfs_remove_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let fs = OsFs {};
    let path = temp_dir.path().join("test.txt");

    fs.writer(&path).unwrap().write_all(b"hello").unwrap();
    assert!(fs.exists(&path));
    fs.remove_file(&path).unwrap();
    assert!(!fs.exists(&path));
}

#[test]
fn test_osfs_rename() {
    let temp_dir = tempfile::tempdir().unwrap();
    let fs = OsFs {};
    let old_path = temp_dir.path().join("old.txt");
    let new_path = temp_dir.path().join("new.txt");

    fs.writer(&old_path).unwrap().write_all(b"content").unwrap();
    fs.rename(&old_path, &new_path).unwrap();
    assert!(!fs.exists(&old_path));
    assert!(fs.exists(&new_path));
}

#[test]
fn test_osfs_remove_dir_all() {
    let temp_dir = tempfile::tempdir().unwrap();
    let fs = OsFs {};
    let dir_path = temp_dir.path().join("dir");

    fs.create_dir(&dir_path).unwrap();
    fs.writer(&dir_path.join("file.txt"))
        .unwrap()
        .write_all(b"a")
        .unwrap();
    fs.remove_dir_all(&dir_path).unwrap();
    assert!(!fs.exists(&dir_path));
}

#[test]
fn test_mockfs_remove_file_error() {
    let fs = MockFS::new();
    fs.create_dir(Path::new("dir")).unwrap();

    // remove_file on a directory should fail
    let res = fs.remove_file(Path::new("dir"));
    assert!(res.is_err());
}

#[test]
fn test_mockfs_remove_dir_all_error() {
    let fs = MockFS::new();
    fs.add_file(Path::new("file.txt"), "content").unwrap();

    // remove_dir_all on a file should fail
    let res = fs.remove_dir_all(Path::new("file.txt"));
    assert!(res.is_err());
}

#[test]
fn test_mockfs_rename_error() {
    let fs = MockFS::new();

    // rename non-existent should fail
    let res = fs.rename(Path::new("none"), Path::new("new"));
    assert!(res.is_err());
}

#[test]
fn test_mockfs_copy_recursive() {
    let fs1 = MockFS::new();
    fs1.add_file(Path::new("dir/a.txt"), "a").unwrap();
    fs1.add_file(Path::new("dir/subdir/b.txt"), "b").unwrap();

    let fs2 = MockFS::new();
    fs2.copy_recursive(&fs1, Path::new("dir"), Path::new("copied")).unwrap();

    assert!(fs2.is_file(Path::new("copied/a.txt")));
    assert!(fs2.is_file(Path::new("copied/subdir/b.txt")));

    let mut buf = String::new();
    fs2.reader(Path::new("copied/subdir/b.txt")).unwrap().read_to_string(&mut buf).unwrap();
    assert_eq!(buf, "b");
}

#[test]
fn test_mockfs_read_dir() {
    let fs = MockFS::new();
    fs.add_file(Path::new("a.txt"), "a").unwrap();
    fs.add_file(Path::new("b.txt"), "b").unwrap();

    let mut entries = Vec::new();
    for de in fs.read_dir(Path::new("")).unwrap() {
        entries.push(de.unwrap().path());
    }

    entries.sort();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0], Path::new("a.txt"));
    assert_eq!(entries[1], Path::new("b.txt"));
}

#[test]
fn test_mockfs_read_dir_mutation() {
    let fs = MockFS::new();
    fs.add_file(Path::new("a.txt"), "a").unwrap();

    // We can now iterate and mutate directly because the iterator
    // doesn't hold a borrow on the MockFS (it collected entries up front).
    for de in fs.read_dir(Path::new("")).unwrap() {
        let de = de.unwrap();
        if de.path() == Path::new("a.txt") {
            let mut w = fs.writer(Path::new("b.txt")).unwrap();
            w.write_all(b"b").unwrap();
        }
    }

    assert!(fs.is_file(Path::new("b.txt")));
}

#[test]
fn test_mockfs_directories() {
    let fs = MockFS::new();
    fs.create_dir_all(Path::new("a/b/c")).unwrap();
    assert!(fs.is_dir(Path::new("a")));
    assert!(fs.is_dir(Path::new("a/b")));
    assert!(fs.is_dir(Path::new("a/b/c")));
}

#[test]
fn test_osfs_basic() {
    let temp_dir = tempfile::tempdir().unwrap();
    let fs = OsFs {};
    let path = temp_dir.path().join("test.txt");
    let content = "hello osfs";

    {
        let fs_mut = OsFs {};
        let mut writer = fs_mut.writer(&path).unwrap();
        writer.write_all(content.as_bytes()).unwrap();
    }

    let mut reader = fs.reader(&path).unwrap();
    let mut buf = String::new();
    reader.read_to_string(&mut buf).unwrap();
    assert_eq!(buf, content);
}
