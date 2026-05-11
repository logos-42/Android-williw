use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub wallet_address: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn create_token(user_id: Uuid, wallet_address: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::hours(24);

    let claims = Claims {
        sub: user_id,
        wallet_address: wallet_address.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"your-super-secret-jwt-key-change-in-production"),
    )
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(b"your-super-secret-jwt-key-change-in-production"),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}
