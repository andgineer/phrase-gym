use std::collections::BTreeMap;
use std::path::PathBuf;

use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::media::{MediaMetadata, MediaOrigin, MediaRole, SettingValue};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PiperVoice {
    pub executable: PathBuf,
    pub runtime_version: String,
    pub model: PathBuf,
    pub model_sha256: String,
    pub config: PathBuf,
    pub config_sha256: String,
    pub voice: String,
    pub locale: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpeechRequest {
    pub language: Language,
    pub text: String,
    pub speech_text: String,
    pub source: String,
    pub role: MediaRole,
}

impl SpeechRequest {
    pub fn metadata(&self, voice: &PiperVoice) -> Result<MediaMetadata, String> {
        for value in [
            &self.text,
            &self.speech_text,
            &self.source,
            &voice.voice,
            &voice.locale,
        ] {
            if value.trim().is_empty()
                || value.trim() != value
                || value.len() > 800
                || value.chars().any(char::is_control)
            {
                return Err(
                    "Speech needs bounded exact text and explicit source/voice/locale".into(),
                );
            }
        }
        if voice.runtime_version != "1.8.0"
            || !voice.executable.is_absolute()
            || !voice.model.is_absolute()
            || !voice.config.is_absolute()
            || voice.locale.split('-').next() != Some(self.language.code())
            || voice.voice.contains("serbski_institut")
            || self.role == MediaRole::Cue
        {
            return Err("Select Piper 1.8.0 with absolute paths and a matching reference/partner voice; Lower Sorbian is not Serbian".into());
        }
        for hash in [&voice.model_sha256, &voice.config_sha256] {
            if hash.len() != 64
                || !hash
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err("Pin lowercase model and configuration SHA-256 hashes".into());
            }
        }
        Ok(MediaMetadata {
            language: self.language,
            text: self.text.clone(),
            role: self.role,
            origin: MediaOrigin::Generated,
            source: self.source.clone(),
            performer: None,
            engine: Some("piper".into()),
            voice: Some(voice.voice.clone()),
            locale: voice.locale.clone(),
            accent: None,
            speech_text: Some(self.speech_text.clone()),
            preparation_version: super::PREPARATION_VERSION.into(),
            settings: BTreeMap::from([
                ("speaker".into(), SettingValue::Integer(0)),
                ("normalize_audio".into(), SettingValue::Flag(true)),
                (
                    "runtime_version".into(),
                    SettingValue::Text(voice.runtime_version.clone()),
                ),
                (
                    "model_sha256".into(),
                    SettingValue::Text(voice.model_sha256.clone()),
                ),
                (
                    "config_sha256".into(),
                    SettingValue::Text(voice.config_sha256.clone()),
                ),
                ("length_scale".into(), SettingValue::Text("1".into())),
                ("noise_scale".into(), SettingValue::Text("0.667".into())),
                ("noise_w_scale".into(), SettingValue::Text("0.8".into())),
                ("volume".into(), SettingValue::Text("1".into())),
                ("sentence_silence".into(), SettingValue::Text("0.2".into())),
            ]),
        })
    }
}
