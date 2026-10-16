use std::path::Path;
use std::time::{Duration, SystemTime};

use serde::Deserialize;
use serde_json::Value;

use super::documents::scenario_bytes;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setup {
    #[serde(default)]
    sql: Vec<String>,
    #[serde(default)]
    files: Vec<DataFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataFile {
    path: String,
    bytes: Option<Value>,
    age_seconds: Option<u64>,
    #[serde(default)]
    absent: bool,
}

impl DataFile {
    fn content(&self) -> Vec<u8> {
        scenario_bytes(self.bytes.as_ref().expect("a present file names its bytes"))
    }
}

pub fn execute_sql(database: &Path, statements: &[String]) {
    if !statements.is_empty() {
        rusqlite::Connection::open(database)
            .unwrap()
            .execute_batch(&statements.join("\n"))
            .unwrap();
    }
}

pub fn prepare(setup: &Setup, data_dir: &Path, database: &Path) {
    execute_sql(database, &setup.sql);
    for file in &setup.files {
        let path = data_dir.join(&file.path);
        if file.absent {
            std::fs::remove_file(&path).unwrap();
            continue;
        }
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, file.content()).unwrap();
        if let Some(age) = file.age_seconds {
            let modified = SystemTime::now() - Duration::from_secs(age);
            std::fs::File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_modified(modified)
                .unwrap();
        }
    }
}

pub fn check_files(step: &str, files: &[DataFile], data_dir: &Path) {
    for file in files {
        let path = data_dir.join(&file.path);
        if file.absent {
            assert!(!path.exists(), "{step} {}", file.path);
        } else {
            assert_eq!(
                std::fs::read(&path).ok(),
                Some(file.content()),
                "{step} {}",
                file.path
            );
        }
    }
}
