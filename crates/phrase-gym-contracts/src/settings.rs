use serde::{Deserialize, Serialize};

use crate::base::Language;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum UiLanguageMode {
    #[default]
    Native,
    Learning,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Settings {
    pub native_language: Option<Language>,
    pub ui_language_mode: UiLanguageMode,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct SettingsUpdate {
    pub native_language: Language,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = "native"))]
    pub ui_language_mode: UiLanguageMode,
}

impl From<SettingsUpdate> for Settings {
    fn from(update: SettingsUpdate) -> Self {
        Self {
            native_language: Some(update.native_language),
            ui_language_mode: update.ui_language_mode,
        }
    }
}
