use std::collections::BTreeMap;

use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::profiles::{LanguagePreferences, PersonalProfile};

pub(super) fn current_learner() -> PersonalProfile {
    let language = |proficiency: Option<&str>,
                    card_language,
                    help_language,
                    translation_with_answer| LanguagePreferences {
        proficiency: proficiency.map(str::to_owned),
        card_language,
        help_language,
        translation_with_answer,
        preparation_guidance: String::new(),
    };
    PersonalProfile {
        native_language: Language::Ru,
        learning_timezone: "Europe/Belgrade".into(),
        languages: BTreeMap::from([
            (
                Language::En,
                language(Some("B2"), Language::En, Some(Language::En), Some(false)),
            ),
            (
                Language::De,
                language(Some("A1"), Language::Ru, Some(Language::Ru), Some(true)),
            ),
            (Language::Sr, language(None, Language::Sr, None, None)),
        ]),
    }
}
