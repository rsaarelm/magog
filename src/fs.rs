use std::path::{Path};
use std::fs;

/// Temporary replacement for unstable library PathExt.
pub trait PathExt {
    fn exists(&self) -> bool;
}

impl<P: AsRef<Path>> PathExt for P {
    fn exists(&self) -> bool { fs::metadata(self).is_ok() }
}

#[cfg(test)]
mod test {
    use std::fs::{remove_file, File};
    use std::path::{Path, PathBuf};
    use super::PathExt;

    #[test]
    fn test_pathext() {
        // XXX: Unit test uses filesystem, expects file creation and
        // destruction to work right.
        File::create(".pathext_test").unwrap();
        assert!(Path::new(".pathext_test").exists());
        assert!(PathBuf::from(".pathext_test").exists());
        remove_file(".pathext_test").unwrap();
        assert!(!Path::new(".pathext_test").exists());
        assert!(!PathBuf::from(".pathext_test").exists());
    }
}
