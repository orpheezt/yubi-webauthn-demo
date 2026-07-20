use webauthn_rs::prelude::*;
use webauthn_rs::Webauthn;

pub fn build_webauthn(rp_id: &str, rp_origin_str: &str) -> Result<Webauthn, WebauthnError> {
    let rp_origin = Url::parse(rp_origin_str).expect("Invalid RP_ORIGIN URL");
    let builder = WebauthnBuilder::new(rp_id, &rp_origin)?;
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_webauthn() {
        let _webauthn = build_webauthn("localhost", "http://localhost:8080").expect("Failed to build Webauthn");
    }
}
