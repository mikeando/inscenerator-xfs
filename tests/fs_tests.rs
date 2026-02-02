use inscenerator_xfs::{OsFs, Xfs};
use inscenerator_xfs::mockfs::MockFS;
use std::path::Path;
use std::io::{Read, Write};

#[test]
fn test_mockfs_basic() {
    let mut fs = MockFS::new();
    let path = Path::new("test.txt");
    let content = "hello world";

    fs.add_file(path, content).unwrap();

    let mut reader = fs.reader(path).unwrap();
    let mut buf = String::new();
    reader.read_to_string(&mut buf).unwrap();
    assert_eq!(buf, content);
}

#[test]
fn test_mockfs_copy_recursive() {
    let mut fs1 = MockFS::new();
    fs1.add_file(Path::new("dir/a.txt"), "a").unwrap();
    fs1.add_file(Path::new("dir/subdir/b.txt"), "b").unwrap();

    let mut fs2 = MockFS::new();
    fs2.copy_recursive(&fs1, Path::new("dir"), Path::new("copied")).unwrap();

    assert!(fs2.is_file(Path::new("copied/a.txt")));
    assert!(fs2.is_file(Path::new("copied/subdir/b.txt")));

    let mut buf = String::new();
    fs2.reader(Path::new("copied/subdir/b.txt")).unwrap().read_to_string(&mut buf).unwrap();
    assert_eq!(buf, "b");
}

#[test]
fn test_mockfs_read_dir() {
    let mut fs = MockFS::new();
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
fn test_mockfs_read_dir_mut() {
    let mut fs = MockFS::new();
    fs.add_file(Path::new("a.txt"), "a").unwrap();

    // In the new iterator approach, we can just loop and use fs_mut
    let paths: Vec<_> = fs.read_dir(Path::new("")).unwrap()
        .map(|de| de.unwrap().path())
        .collect();

    for path in paths {
        if path == Path::new("a.txt") {
            let mut w = fs.writer(Path::new("b.txt")).unwrap();
            w.write_all(b"b").unwrap();
        }
    }

    assert!(fs.is_file(Path::new("b.txt")));
}

#[test]
fn test_mockfs_directories() {
    let mut fs = MockFS::new();
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
        let mut fs_mut = OsFs {};
        let mut writer = fs_mut.writer(&path).unwrap();
        writer.write_all(content.as_bytes()).unwrap();
    }

    let mut reader = fs.reader(&path).unwrap();
    let mut buf = String::new();
    reader.read_to_string(&mut buf).unwrap();
    assert_eq!(buf, content);
}
