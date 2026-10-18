use std::{ffi::OsString, path::Path};

use super::{FileStore, sha256_hex};

#[test]
fn hashes_known_bytes() {
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[tokio::test]
async fn put_stores_the_bytes_under_their_hash() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStore::new(dir.path().join("nested"));
    let sha = store.put(b"hello").await.unwrap();
    assert_eq!(sha, sha256_hex(b"hello"));
    assert_eq!(std::fs::read(store.path(&sha)).unwrap(), b"hello");
}

#[tokio::test]
async fn putting_the_same_bytes_twice_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStore::new(dir.path());
    let first = store.put(b"same").await.unwrap();
    let second = store.put(b"same").await.unwrap();
    assert_eq!(first, second);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[tokio::test]
async fn put_leaves_no_partial_file_behind() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStore::new(dir.path());
    store.put(b"x").await.unwrap();
    let names: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(names.iter().all(|name| !name.contains(".new")), "{names:?}");
}

#[tokio::test]
async fn remove_deletes_and_tolerates_a_missing_file() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStore::new(dir.path());
    let sha = store.put(b"bye").await.unwrap();
    assert!(store.contains(&sha).await.unwrap());
    store.remove(&sha).await.unwrap();
    assert!(!store.contains(&sha).await.unwrap());
    store.remove(&sha).await.unwrap();
}

#[test]
fn the_folder_comes_from_the_env_then_stage_then_home() {
    let home = Path::new("/h");
    let dir = |dir: Option<&str>, stage| {
        FileStore::resolve(dir.map(OsString::from), Some(home), stage)
            .root()
            .to_owned()
    };
    assert_eq!(dir(Some("/x"), true), Path::new("/x"));
    assert_eq!(
        dir(None, false),
        Path::new("/h/Library/Application Support/Vitarchy/statements")
    );
    assert!(dir(None, true).starts_with(std::env::temp_dir()));
    assert_eq!(dir(Some(""), false), dir(None, false));
    assert_eq!(
        FileStore::resolve(None, None, false).root(),
        Path::new("statements")
    );
}
