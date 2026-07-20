use webauthn_rs::Webauthn;
use webauthn_rs::prelude::*;

pub fn build_webauthn(rp_id: &str, rp_origin_str: &str) -> Result<Webauthn, WebauthnError> {
    let rp_origin = Url::parse(rp_origin_str).map_err(|_| WebauthnError::Configuration)?;
    let builder = WebauthnBuilder::new(rp_id, &rp_origin)?;
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_webauthn() -> Result<(), WebauthnError> {
        let rp_id = std::env::var("TEST_RP_ID").unwrap_or_else(|_| "localhost".to_string());
        let rp_origin =
            std::env::var("TEST_RP_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let _webauthn = build_webauthn(&rp_id, &rp_origin)?;
        Ok(())
    }
}
