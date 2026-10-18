use base64::Engine;
use base64::alphabet::STANDARD;
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};

// Standard alphabet with canonical padding; non-zero trailing bits are tolerated, as the browser
// encoders that produce uploads may emit them.
const DECODER: GeneralPurpose = GeneralPurpose::new(
    &STANDARD,
    GeneralPurposeConfig::new().with_decode_allow_trailing_bits(true),
);

pub(crate) fn decode_base64(encoded: &str) -> Option<Vec<u8>> {
    DECODER.decode(encoded).ok()
}

#[cfg(test)]
mod tests {
    use super::decode_base64;

    #[test]
    fn decodes_padded_standard_base64_only() {
        assert_eq!(decode_base64("aGk=").unwrap(), b"hi");
        assert_eq!(decode_base64("aGl=").unwrap(), b"hi");
        for invalid in ["aGk", "aG k=", "aGk=\n", "not base64", "aGk_"] {
            assert!(decode_base64(invalid).is_none(), "{invalid:?}");
        }
    }
}
