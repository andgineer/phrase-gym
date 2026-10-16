use std::path::{Path, PathBuf};

use crate::tools::Failure;

const MAX_PRODUCTION_LINES: usize = 300;
const RUST: &[&str] = &["rs"];
const CLIENT: &[&str] = &["ts", "vue"];

pub fn oversized_modules(root: &Path) -> Result<Vec<String>, Failure> {
    let mut sources = Vec::new();
    let crates = root.join("crates");
    if crates.is_dir() {
        for entry in read_dir(&crates)? {
            collect(&entry.join("src"), RUST, &mut sources)?;
        }
    }
    collect(&root.join("xtask/src"), RUST, &mut sources)?;
    collect(&root.join("frontend/src"), CLIENT, &mut sources)?;
    let mut problems = Vec::new();
    for path in sources {
        let text = std::fs::read_to_string(&path)
            .map_err(|error| Failure::new(format!("reading {}: {error}", path.display())))?;
        if text.lines().count() > MAX_PRODUCTION_LINES {
            problems.push(format!(
                "{}: exceeds {MAX_PRODUCTION_LINES} lines; split responsibilities",
                path.display()
            ));
        }
    }
    Ok(problems)
}

fn read_dir(directory: &Path) -> Result<Vec<PathBuf>, Failure> {
    let entries = std::fs::read_dir(directory)
        .map_err(|error| Failure::new(format!("reading {}: {error}", directory.display())))?;
    let mut paths: Vec<_> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    paths.sort();
    Ok(paths)
}

fn handwritten(path: &Path, extensions: &[&str]) -> bool {
    let generated = path
        .file_name()
        .is_some_and(|name| name.to_string_lossy().contains(".generated."));
    !generated
        && path
            .extension()
            .is_some_and(|extension| extensions.iter().any(|allowed| extension == *allowed))
}

fn collect(
    directory: &Path,
    extensions: &[&str],
    sources: &mut Vec<PathBuf>,
) -> Result<(), Failure> {
    if !directory.is_dir() {
        return Ok(());
    }
    for path in read_dir(directory)? {
        if path.is_dir() {
            collect(&path, extensions, sources)?;
        } else if handwritten(&path, extensions) {
            sources.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_lines(path: &Path, count: usize) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "// line\n".repeat(count)).unwrap();
    }

    #[test]
    fn flags_production_modules_over_the_review_boundary() {
        let root = tempfile::tempdir().unwrap();
        write_lines(&root.path().join("crates/example/src/nested/large.rs"), 301);
        write_lines(&root.path().join("crates/example/src/nested/small.rs"), 300);
        write_lines(&root.path().join("crates/example/tests/long.rs"), 400);
        write_lines(&root.path().join("frontend/src/components/Large.vue"), 301);
        write_lines(&root.path().join("frontend/src/api.generated.ts"), 900);
        let problems = oversized_modules(root.path()).unwrap();
        assert_eq!(problems.len(), 2, "{problems:?}");
        assert!(problems[0].contains("large.rs"));
        assert!(problems[1].contains("Large.vue"));
    }
}
