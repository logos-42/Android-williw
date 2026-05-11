use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub wallet_address: String,
    pub email: Option<String>,
    pub balance: f64,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(wallet_address: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            wallet_address,
            email: None,
            balance: 0.0,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModel {
    pub id: Uuid,
    pub name: String,
    pub provider: String,
    pub category: ModelCategory,
    pub description: String,
    pub compute_power: f64,
    pub price_per_unit: f64,
    pub status: ModelStatus,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelCategory {
    Llm,
    Image,
    Audio,
    Video,
    Multimodal,
}

impl std::fmt::Display for ModelCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelCategory::Llm => write!(f, "llm"),
            ModelCategory::Image => write!(f, "image"),
            ModelCategory::Audio => write!(f, "audio"),
            ModelCategory::Video => write!(f, "video"),
            ModelCategory::Multimodal => write!(f, "multimodal"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelStatus {
    Active,
    Maintenance,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub model_id: Uuid,
    pub amount: f64,
    pub status: ComputeStatus,
    pub result: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ComputeStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub model_id: Uuid,
    pub amount: f64,
    pub payment_method: PaymentMethod,
    pub status: OrderStatus,
    pub crypto_amount: Option<f64>,
    pub crypto_currency: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    Wechat,
    Alipay,
    Usdt,
    Eth,
    Btc,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Paid,
    Processing,
    Completed,
    Failed,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFilter {
    pub category: Option<ModelCategory>,
    pub provider: Option<String>,
    pub min_power: Option<f64>,
    pub max_price: Option<f64>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletLoginRequest {
    pub wallet_address: String,
    pub signature: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletLoginResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub order_id: Uuid,
    pub method: PaymentMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub qr_code: Option<String>,
    pub payment_url: Option<String>,
    pub crypto_address: Option<CryptoPaymentInfo>,
    pub order_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoPaymentInfo {
    pub address: String,
    pub amount: f64,
    pub currency: String,
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}
