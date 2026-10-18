//! The JSON the helper reads on stdin.

use std::{fmt, path::Path};

use serde::Serialize;

use super::contract::VERSION;

/// The request for one file. Its `Debug` hides the passwords.
#[derive(Serialize)]
pub struct Request<'a> {
    /// Contract version.
    pub version: u32,
    /// The file to read.
    pub file_path: &'a Path,
    /// Saved Statement passwords, in order.
    pub passwords: &'a [String],
    /// Where the decrypted copy goes.
    pub decrypted_out: &'a Path,
}

impl<'a> Request<'a> {
    /// A version 1 request.
    #[must_use]
    pub const fn new(
        file_path: &'a Path,
        passwords: &'a [String],
        decrypted_out: &'a Path,
    ) -> Self {
        Self {
            version: VERSION,
            file_path,
            passwords,
            decrypted_out,
        }
    }
}

impl fmt::Debug for Request<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Request")
            .field("version", &self.version)
            .field("file_path", &self.file_path)
            .field(
                "passwords",
                &format_args!("[{} hidden]", self.passwords.len()),
            )
            .field("decrypted_out", &self.decrypted_out)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Request;

    #[test]
    fn debug_never_shows_a_password() {
        let passwords = vec!["hunter2".to_owned(), "swordfish".to_owned()];
        let request = Request::new(Path::new("/a"), &passwords, Path::new("/b"));
        let shown = format!("{request:?}");
        assert!(
            !shown.contains("hunter2") && !shown.contains("swordfish"),
            "{shown}"
        );
        assert!(shown.contains("2 hidden"));
    }

    #[test]
    fn serialises_the_contract_fields() {
        let passwords = vec!["p".to_owned()];
        let request = Request::new(Path::new("/a"), &passwords, Path::new("/b"));
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"version":1,"file_path":"/a","passwords":["p"],"decrypted_out":"/b"})
        );
    }
}
