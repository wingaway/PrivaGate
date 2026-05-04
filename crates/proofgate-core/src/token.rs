use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn canonical(value: &str) -> String {
    value.trim().to_lowercase()
}

pub fn hmac_token(secret: &[u8], field_type: &str, value: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts keys of any size");
    mac.update(field_type.as_bytes());
    mac.update(b":");
    mac.update(canonical(value).as_bytes());
    let digest = mac.finalize().into_bytes();
    let short = &hex::encode(digest)[..24];
    format!("<{}_{}>", field_type.to_ascii_uppercase(), short)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hmac_tokens_are_deterministic_and_canonicalized() {
        let a = hmac_token(b"secret", "person", " Alice ");
        let b = hmac_token(b"secret", "person", "alice");
        assert_eq!(a, b);
        assert!(a.starts_with("<PERSON_"));
    }

    #[test]
    fn different_field_types_have_distinct_tokens() {
        let a = hmac_token(b"secret", "person", "alice");
        let b = hmac_token(b"secret", "org", "alice");
        assert_ne!(a, b);
    }
}
