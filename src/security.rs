use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn verify_signature(secret: &str, body: &[u8], signature_header: &str) -> bool {
    if signature_header.is_empty() {
        return false;
    }

    let signature = signature_header.trim_start_matches("sha256=");
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");

    mac.update(body);
    let result = mac.finalize().into_bytes();
    let expected = hex::encode(result);

    expected == signature
}