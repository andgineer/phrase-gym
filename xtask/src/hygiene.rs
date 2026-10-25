use yaml_rust2::YamlLoader;

use crate::tools::{Failure, Workspace};

pub fn check(workspace: &Workspace) -> Result<(), Failure> {
    let tracked = workspace.git_output(&["ls-files", "-z"])?;
    let mut problems = Vec::new();
    for name in tracked.split('\0').filter(|name| !name.is_empty()) {
        let path = workspace.root.join(name);
        // Symlinks and tracked files deleted from the working tree have no content to check.
        let regular = std::fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.is_file());
        if !regular {
            continue;
        }
        let content = std::fs::read(&path)
            .map_err(|error| Failure::new(format!("reading {name}: {error}")))?;
        problems.extend(file_problems(name, &content));
    }
    if problems.is_empty() {
        println!("xtask: hygiene checks passed");
        Ok(())
    } else {
        Err(Failure::new(problems.join("\n")))
    }
}

fn has_extension(name: &str, extensions: &[&str]) -> bool {
    std::path::Path::new(name)
        .extension()
        .is_some_and(|extension| {
            extensions
                .iter()
                .any(|allowed| extension.eq_ignore_ascii_case(allowed))
        })
}

pub fn file_problems(name: &str, content: &[u8]) -> Vec<String> {
    let mut problems = Vec::new();
    if has_extension(name, &["yaml", "yml"]) {
        problems.extend(yaml_problem(content).map(|problem| format!("{name}: {problem}")));
    }
    if is_text(content) {
        for line in trailing_whitespace_lines(content) {
            problems.push(format!("{name}:{line}: trailing whitespace"));
        }
        problems.extend(end_of_file_problem(content).map(|problem| format!("{name}: {problem}")));
    }
    problems
}

fn yaml_problem(content: &[u8]) -> Option<String> {
    let Ok(text) = std::str::from_utf8(content) else {
        return Some("YAML is not valid UTF-8".to_owned());
    };
    match YamlLoader::load_from_str(text) {
        Ok(documents) if documents.len() > 1 => Some("YAML has more than one document".to_owned()),
        Ok(_) => None,
        Err(error) => Some(format!("invalid YAML: {error}")),
    }
}

// Control bytes other than common whitespace and escape in the first KiB mean binary content.
fn is_text(content: &[u8]) -> bool {
    content
        .iter()
        .take(1024)
        .all(|byte| matches!(byte, 7..=13 | 27 | 0x20..=0x7e | 0x80..=0xff))
}

fn trailing_whitespace_lines(content: &[u8]) -> Vec<usize> {
    content
        .split_inclusive(|byte| *byte == b'\n')
        .enumerate()
        .filter(|(_, line)| {
            let line = line
                .strip_suffix(b"\r\n")
                .or_else(|| line.strip_suffix(b"\n"))
                .unwrap_or(line);
            line.last()
                .is_some_and(|byte| byte.is_ascii_whitespace() || *byte == 0x0b)
        })
        .map(|(index, _)| index + 1)
        .collect()
}

fn end_of_file_problem(content: &[u8]) -> Option<&'static str> {
    if content.is_empty() {
        return None;
    }
    let breaks = content
        .iter()
        .rev()
        .take_while(|byte| matches!(byte, b'\n' | b'\r'))
        .count();
    match &content[content.len() - breaks..] {
        _ if breaks == content.len() => Some("contains only line breaks"),
        b"" => Some("missing newline at end of file"),
        b"\n" | b"\r\n" | b"\r" => None,
        _ => Some("blank lines at end of file"),
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    fn problems(name: &str, content: &str) -> Vec<String> {
        file_problems(name, content.as_bytes())
    }

    #[test]
    fn reports_trailing_whitespace_on_each_line_of_text_files() {
        assert_eq!(
            problems("docs/a.md", "title  \nbody\r\nend\t\n"),
            [
                "docs/a.md:1: trailing whitespace",
                "docs/a.md:3: trailing whitespace"
            ]
        );
        assert_eq!(
            problems("a.ts", "windows\r\nlast \r"),
            ["a.ts:2: trailing whitespace"]
        );
        assert!(problems("a.rs", "fn main() {}\r\n").is_empty());
    }

    #[test]
    fn requires_exactly_one_final_line_break() {
        assert_eq!(
            problems("a.txt", "text"),
            ["a.txt: missing newline at end of file"]
        );
        assert_eq!(
            problems("a.txt", "text\n\n"),
            ["a.txt: blank lines at end of file"]
        );
        assert_eq!(
            problems("a.txt", "\n\r\n"),
            ["a.txt: contains only line breaks"]
        );
        for valid in ["", "text\n", "text\r\n"] {
            assert!(problems("a.txt", valid).is_empty(), "{valid:?}");
        }
        assert_eq!(end_of_file_problem(b"old mac\r"), None);
    }

    #[test]
    fn rejects_invalid_duplicate_and_multi_document_yaml() {
        assert!(problems("ci.yml", "on:\n  push:\n    branches: [main]\n").is_empty());
        assert!(problems("a.yaml", "key: [1, 2\n")[0].starts_with("a.yaml: invalid YAML"));
        assert!(problems("a.yaml", "key: 1\nkey: 2\n")[0].contains("duplicated key"));
        assert_eq!(
            problems("a.yml", "a: 1\n---\nb: 2\n"),
            ["a.yml: YAML has more than one document"]
        );
    }

    #[test]
    fn skips_binary_files_and_checks_every_text_file() {
        assert!(file_problems("media/tone.wav", b"RIFF\0\0 \n\n").is_empty());
        assert_eq!(
            problems(".githooks/pre-commit", "#!/bin/sh\necho \n").len(),
            1
        );
        assert_eq!(problems("notes", "text  \n\n").len(), 2);
    }

    #[test]
    fn checks_only_tracked_files_of_the_repository() {
        let root = tempfile::tempdir().unwrap();
        let write =
            |name: &str, content: &str| std::fs::write(root.path().join(name), content).unwrap();
        write("clean.md", "clean\n");
        write("dirty.md", "dirty \n");
        write("removed.md", "removed\n");
        write("untracked.md", "untracked ");
        let git = |arguments: &[&str]| {
            let status = Command::new("git")
                .args(arguments)
                .current_dir(root.path())
                .status();
            assert!(status.unwrap().success(), "git {arguments:?}");
        };
        git(&["init", "--quiet"]);
        git(&["add", "clean.md", "dirty.md", "removed.md"]);
        std::fs::remove_file(root.path().join("removed.md")).unwrap();
        let workspace = Workspace {
            root: root.path().to_path_buf(),
        };
        let failure = check(&workspace).unwrap_err().to_string();
        assert_eq!(failure, "dirty.md:1: trailing whitespace");
        write("dirty.md", "dirty\n");
        assert!(check(&workspace).is_ok());
    }
}
