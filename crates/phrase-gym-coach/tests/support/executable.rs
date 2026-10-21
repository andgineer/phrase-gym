use std::path::Path;

pub fn alias(source: &Path, destination: &Path) {
    // A concurrent fork can inherit a copied executable's writer and cause Linux ETXTBSY.
    // Alias the immutable Cargo artifact instead of opening its inode for writing.
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, destination).unwrap();
    #[cfg(windows)]
    std::fs::copy(source, destination).unwrap();
}
