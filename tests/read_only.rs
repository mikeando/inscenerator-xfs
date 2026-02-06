use std::path::Path;
use inscenerator_xfs::{Xfs, XfsReadOnly, mockfs::MockFS};

#[test]
fn test_unsafe_clone_is_readonly() {
    let mut fs = MockFS::new();
    fs.add_file(Path::new("test.txt"), "hello").unwrap();

    let fs_ref: &dyn Xfs = &fs;

    // unsafe_clone on a &dyn Xfs returns Box<dyn XfsReadOnly>
    let fs_clone = fs_ref.unsafe_clone();

    // This should NOT compile if we try to call writer()
    // fs_clone.writer(Path::new("other.txt")).unwrap();

    assert!(fs_clone.exists(Path::new("test.txt")));
}

#[test]
fn test_unsafe_clone_mut_is_writable() {
    let mut fs = MockFS::new();
    fs.add_file(Path::new("test.txt"), "hello").unwrap();

    let fs_mut_ref: &mut dyn Xfs = &mut fs;

    // unsafe_clone_mut on a &mut dyn Xfs returns Box<dyn Xfs>
    let mut fs_clone = fs_mut_ref.unsafe_clone_mut();

    fs_clone.writer(Path::new("other.txt")).unwrap();

    assert!(fs.exists(Path::new("other.txt")));
}
