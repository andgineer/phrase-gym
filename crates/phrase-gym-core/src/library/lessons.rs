use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::library::Lesson;
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::content::records;
use crate::error::CoreError;
use crate::library::search::Search;

type LessonRow = (String, String, String);

fn row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LessonRow> {
    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
}

fn lesson(connection: &Connection, (id, title, language): LessonRow) -> Result<Lesson, CoreError> {
    let phrase_count: i64 = connection.query_row(
        "SELECT count(*) FROM lesson_membership WHERE lesson_id = ?1",
        [&id],
        |row| row.get(0),
    )?;
    Ok(Lesson {
        id: records::uuid(&id)?,
        language: records::language(&language)?,
        title,
        phrase_count,
    })
}

pub(crate) fn of_target(connection: &Connection, target: Uuid) -> Result<Vec<Lesson>, CoreError> {
    let mut statement = connection.prepare(
        "SELECT l.id, l.title, l.language FROM lesson l \
         JOIN lesson_membership m ON m.lesson_id = l.id \
         WHERE m.target_id = ?1 ORDER BY l.title, l.id",
    )?;
    let rows: Vec<LessonRow> = statement
        .query_map([target.to_string()], row)?
        .collect::<Result<_, _>>()?;
    rows.into_iter()
        .map(|item| lesson(connection, item))
        .collect()
}

pub(crate) fn in_language(
    connection: &Connection,
    language: Language,
    search: &Search,
) -> Result<Vec<Lesson>, CoreError> {
    let mut statement = connection
        .prepare("SELECT id, title, language FROM lesson WHERE language = ?1 ORDER BY title, id")?;
    let rows: Vec<LessonRow> = statement
        .query_map([language.code()], row)?
        .collect::<Result<_, _>>()?;
    rows.into_iter()
        .filter(|(_, title, _)| search.matches(title))
        .map(|item| lesson(connection, item))
        .collect()
}

pub(crate) fn require(
    connection: &Connection,
    id: Uuid,
    language: Language,
) -> Result<(), CoreError> {
    let found: Option<String> = connection
        .query_row(
            "SELECT language FROM lesson WHERE id = ?1",
            params![id.to_string()],
            |row| row.get(0),
        )
        .optional()?;
    if found.as_deref() == Some(language.code()) {
        Ok(())
    } else {
        Err(CoreError::NotFound(
            "Lesson not found in this learning language".to_owned(),
        ))
    }
}
