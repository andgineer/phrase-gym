use std::path::Path;

use crate::platform::files;

pub(super) fn read(path: &Path) -> Result<String, String> {
    if path
        .extension()
        .is_none_or(|extension| !extension.eq_ignore_ascii_case("txt"))
    {
        return Err(
            "Drop one plain .txt file containing one English phrase, or paste phrase text".into(),
        );
    }
    let bytes = files::read_selected(path, 4000)?;
    let phrase = String::from_utf8(bytes).map_err(|_| "The dropped phrase must be UTF-8 text")?;
    if phrase.trim().is_empty() {
        return Err("The dropped text is empty; paste or drop one English phrase".into());
    }
    Ok(phrase)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_bounded_plain_text_and_preserves_exact_input() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("phrase.txt");
        std::fs::write(&path, "  Let's get started.\n").unwrap();
        assert_eq!(read(&path).unwrap(), "  Let's get started.\n");
        for bytes in [vec![b'x'; 4001], vec![255], b" \n".to_vec()] {
            std::fs::write(&path, bytes).unwrap();
            assert!(read(&path).is_err());
        }
        let other = directory.path().join("prepared.json");
        std::fs::write(&other, "Let's get started.").unwrap();
        assert!(read(&other).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinks_without_reading_their_target() {
        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("target");
        std::fs::write(&target, "Let's get started.").unwrap();
        let link = directory.path().join("phrase.txt");
        std::os::unix::fs::symlink(target, &link).unwrap();
        assert!(read(&link).is_err());
    }
}
