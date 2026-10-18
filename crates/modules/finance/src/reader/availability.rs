//! Whether the helper's program can be found, for the status page.

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// True when `program` names an existing file: as given when it has a folder part, else
/// somewhere on `path_var` (the value of `PATH`).
#[must_use]
pub fn program_exists(program: &OsStr, path_var: Option<&OsStr>) -> bool {
    let named = Path::new(program);
    if named.components().count() > 1 {
        return named.is_file();
    }
    path_var
        .into_iter()
        .flat_map(std::env::split_paths)
        .map(|dir| dir.join(program))
        .any(|candidate: PathBuf| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::program_exists;

    #[test]
    fn finds_a_file_by_path_or_on_the_path_variable() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("reader");
        std::fs::write(&file, "").unwrap();
        assert!(program_exists(file.as_os_str(), None));
        let path_var = OsString::from(dir.path());
        assert!(program_exists("reader".as_ref(), Some(&path_var)));
    }

    #[test]
    fn a_missing_program_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path_var = OsString::from(dir.path());
        assert!(!program_exists("reader".as_ref(), Some(&path_var)));
        assert!(!program_exists(dir.path().join("nope").as_os_str(), None));
        assert!(!program_exists("reader".as_ref(), None));
    }
}
