use crate::supplied::SuppliedLesson;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LessonPreview {
    pub title: String,
    pub phrase: String,
    pub situations: Vec<String>,
    pub help: Option<String>,
    pub audio: String,
}

impl LessonPreview {
    pub fn parse(json: &str) -> Result<Self, String> {
        let lesson = SuppliedLesson::parse(json)?;
        Ok(Self {
            title: lesson.lesson.title,
            phrase: lesson.phrase.text,
            situations: lesson
                .phrase
                .situations
                .into_iter()
                .map(|s| s.text)
                .collect(),
            help: lesson.phrase.help.map(|h| h.meaning),
            audio: format!(
                "Original reference: {} ({})",
                lesson
                    .metadata
                    .performer
                    .as_deref()
                    .unwrap_or("Unspecified performer"),
                lesson.metadata.locale
            ),
        })
    }
}
