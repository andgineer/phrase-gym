use icu_casemap::{CaseMapper, CaseMapperBorrowed};

// Full Unicode case folding rather than lowercasing, so "STRASSE" and "strasse" find "Straße".
const CASE: CaseMapperBorrowed<'static> = CaseMapper::new();

pub(crate) struct Search {
    folded: String,
}

impl Search {
    pub(crate) fn new(query: &str) -> Self {
        Self {
            folded: CASE.fold_string(query).into_owned(),
        }
    }

    pub(crate) fn matches(&self, text: &str) -> bool {
        self.folded.is_empty() || CASE.fold_string(text).contains(self.folded.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::Search;

    #[test]
    fn sharp_s_matches_its_full_case_folding_in_both_directions() {
        for query in ["strasse", "STRASSE", "Straße", "STRAẞE"] {
            assert!(
                Search::new(query).matches("Nach der Straße fragen"),
                "{query}"
            );
        }
        assert!(Search::new("straße").matches("NACH DER STRASSE FRAGEN"));
    }

    #[test]
    fn folding_covers_cyrillic_learning_text() {
        assert!(Search::new("ПРИВЕТ").matches("Привет, как дела?"));
        assert!(Search::new("ЋАО").matches("Ћао, шта има?"));
    }

    #[test]
    fn empty_query_matches_everything_and_other_text_does_not_match() {
        assert!(Search::new("").matches("anything"));
        assert!(!Search::new("strase").matches("Straße"));
    }
}
