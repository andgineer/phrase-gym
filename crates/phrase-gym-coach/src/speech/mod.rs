mod piper;
mod request;
mod runtime;
mod wav;

pub use piper::{StagedSpeech, stage};
pub use request::{PiperVoice, SpeechRequest};

pub const PREPARATION_VERSION: &str = "coach-piper-pcm-v1";
pub const MAX_AUDIO_BYTES: u64 = 2_000_000;
