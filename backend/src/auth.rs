use webauthn_rs::prelude::*;
use webauthn_rs::Webauthn;

pub fn build_webauthn() -> Result<Webauthn, WebauthnError> {
    let rp_id = std::env::var("RP_ID").unwrap_or_else(|_| "localhost".to_string());
    let rp_origin_str = std::env::var("RP_ORIGIN").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let rp_origin = Url::parse(&rp_origin_str).expect("Invalid RP_ORIGIN URL");
    
    let builder = WebauthnBuilder::new(&rp_id, &rp_origin)?;
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_webauthn() {
        let _webauthn = build_webauthn().expect("Failed to build Webauthn");
    }
}
