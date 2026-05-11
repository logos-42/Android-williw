use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;

/// JWT令牌中的声明结构体，包含用户身份信息
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// 用户唯一标识符
    pub sub: Uuid,
    /// 钱包地址
    pub wallet_address: String,
    /// 令牌过期时间戳
    pub exp: i64,
    /// 令牌签发时间戳
    pub iat: i64,
}

/// 创建JWT令牌
/// 根据用户ID和钱包地址生成24小时有效期的JSON Web Token
/// 
/// # 参数
/// * `user_id` - 用户的唯一UUID标识
/// * `wallet_address` - 用户的以太坊钱包地址
/// 
/// # 返回
/// 成功返回JWT令牌字符串，失败返回错误
pub fn create_token(user_id: Uuid, wallet_address: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    // 令牌有效期24小时
    let exp = now + Duration::hours(24);

    let claims = Claims {
        sub: user_id,
        wallet_address: wallet_address.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    // 使用HS256算法和默认Header编码令牌
    encode(
        &Header::default(),
        &claims,
        // 注意：生产环境应使用更安全的密钥
        &EncodingKey::from_secret(b"your-super-secret-jwt-key-change-in-production"),
    )
}

/// 验证JWT令牌
/// 解码并验证令牌的有效性，检查签名和过期时间
/// 
/// # 参数
/// * `token` - 待验证的JWT令牌字符串
/// 
/// # 返回
/// 成功返回解析后的Claims声明，失败返回错误
pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(b"your-super-secret-jwt-key-change-in-production"),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

pub fn extract_user_id(token: &str) -> Option<Uuid> {
    verify_token(token).ok().map(|claims| claims.sub)
}