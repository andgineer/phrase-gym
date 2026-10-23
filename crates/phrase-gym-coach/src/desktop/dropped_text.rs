use std::path::Path;

use crate::platform::files;
use crate::transfer::input_problem::InputProblem;
use crate::transfer::items::MAX_TEXT_BYTES;

// Any regular file whose bytes are plain UTF-8 text qualifies, whatever its name or extension;
// the bytes, not the label, decide.
pub(super) fn read(path: &Path) -> Result<String, InputProblem> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| InputProblem::Unreadable)?;
    if metadata.file_type().is_symlink() {
        return Err(InputProblem::Symlink);
    }
    if metadata.is_dir() {
        return Err(InputProblem::Folder);
    }
    if !metadata.is_file() {
        return Err(InputProblem::Binary);
    }
    if metadata.len() > MAX_TEXT_BYTES as u64 {
        return Err(InputProblem::FileTooLong);
    }
    let bytes =
        files::read_selected(path, MAX_TEXT_BYTES as u64).map_err(|_| InputProblem::Unreadable)?;
    text(&bytes)
}

fn text(bytes: &[u8]) -> Result<String, InputProblem> {
    // A byte-order mark encodes the file, not the learner's words.
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    if bytes.starts_with(b"{\\rtf") {
        return Err(InputProblem::RichText);
    }
    if bytes.contains(&0) {
        return Err(InputProblem::Binary);
    }
    let text = String::from_utf8(bytes.to_vec()).map_err(|_| InputProblem::NotUtf8)?;
    if text
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        return Err(InputProblem::Binary);
    }
    if text.trim().is_empty() {
        return Err(InputProblem::Empty);
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dropped(name: &str, bytes: &[u8]) -> Result<String, InputProblem> {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        read(&path)
    }

    #[test]
    fn plain_text_is_accepted_by_content_whatever_its_name() {
        for name in [
            "phrases.txt",
            "phrases.text",
            "notes.md",
            "phrases",
            "Phrases.TXT",
        ] {
            assert_eq!(
                dropped(name, "  Let's get started.\nimpediment\n".as_bytes()).unwrap(),
                "  Let's get started.\nimpediment\n",
                "{name}"
            );
        }
        assert_eq!(
            dropped("bom.txt", b"\xEF\xBB\xBFimpediment\r\n").unwrap(),
            "impediment\r\n"
        );
        assert_eq!(
            dropped("cyrillic", "Ћао, ђаче!\n".as_bytes()).unwrap(),
            "Ћао, ђаче!\n"
        );
    }

    #[test]
    fn each_unusable_file_names_its_own_problem() {
        let cases: [(&str, &[u8], InputProblem); 6] = [
            (
                "letter.rtf",
                b"{\\rtf1\\ansi impediment}",
                InputProblem::RichText,
            ),
            (
                "image.png",
                b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR",
                InputProblem::Binary,
            ),
            ("legacy.txt", b"caf\xe9 au lait", InputProblem::NotUtf8),
            ("blank.txt", b" \n\t\n", InputProblem::Empty),
            ("bell.txt", b"impediment\x07", InputProblem::Binary),
            ("long.txt", &[b'x'; 4001], InputProblem::FileTooLong),
        ];
        for (name, bytes, problem) in cases {
            assert_eq!(dropped(name, bytes), Err(problem), "{name}");
        }
        let directory = tempfile::tempdir().unwrap();
        assert_eq!(read(directory.path()), Err(InputProblem::Folder));
        assert_eq!(
            read(&directory.path().join("missing.txt")),
            Err(InputProblem::Unreadable)
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinks_without_reading_their_target() {
        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("target");
        std::fs::write(&target, "Let's get started.").unwrap();
        let link = directory.path().join("phrase.txt");
        std::os::unix::fs::symlink(target, &link).unwrap();
        assert_eq!(read(&link), Err(InputProblem::Symlink));
    }
}
