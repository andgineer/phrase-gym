use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::profiles::PersonalProfile;

use crate::error::CoreError;

pub(super) fn validate(profile: &PersonalProfile) -> Result<(), CoreError> {
    if profile.learning_timezone.parse::<chrono_tz::Tz>().is_err() {
        return Err(CoreError::Invalid("Unknown learning timezone".into()));
    }
    if profile.languages.len() != 3
        || ![Language::En, Language::De, Language::Sr]
            .iter()
            .all(|language| profile.languages.contains_key(language))
    {
        return Err(CoreError::Invalid(
            "A complete profile requires English, German and Serbian preferences".into(),
        ));
    }
    for (language, preferences) in &profile.languages {
        if preferences
            .proficiency
            .as_ref()
            .is_some_and(|value| !matches!(value.as_str(), "A1" | "A2" | "B1" | "B2" | "C1" | "C2"))
            || preferences.preparation_guidance.len() > 8_000
            || (*language != Language::Sr
                && (preferences.proficiency.is_none()
                    || preferences.help_language.is_none()
                    || preferences.translation_with_answer.is_none()))
        {
            return Err(CoreError::Invalid(
                "Invalid or unresolved required language preferences".into(),
            ));
        }
    }
    Ok(())
}
