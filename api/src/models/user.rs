use williw_shared::*;

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub wallet_address: String,
    pub signature: String,
    pub message: String,
}

#[derive(serde::Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

#[derive(serde::Deserialize)]
pub struct VerifyRequest {
    pub wallet_address: String,
    pub signature: String,
}

#[derive(serde::Deserialize)]
pub struct ProfileResponse {
    pub user: User,
    pub total_orders: i64,
    pub total_spent: f64,
}
