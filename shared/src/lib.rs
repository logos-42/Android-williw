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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModel {
    pub id: Uuid,
    pub name: String,
    pub model_type: LocalModelType,
    pub size_gb: f64,
    pub status: LocalModelStatus,
    pub download_progress: Option<f32>,
    pub download_url: String,
    pub local_path: Option<String>,
    pub inference_endpoint: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LocalModelType {
    Lf25,
    Gamma4,
    Phi35,
    Qwen25,
    Yi,
    Deepseek,
    Llama,
    Mistral,
    Gemma,
    Custom,
}

impl std::fmt::Display for LocalModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalModelType::Lf25 => write!(f, "lf2.5"),
            LocalModelType::Gamma4 => write!(f, "gamma4"),
            LocalModelType::Phi35 => write!(f, "phi3.5"),
            LocalModelType::Qwen25 => write!(f, "qwen2.5"),
            LocalModelType::Yi => write!(f, "yi"),
            LocalModelType::Deepseek => write!(f, "deepseek"),
            LocalModelType::Llama => write!(f, "llama"),
            LocalModelType::Mistral => write!(f, "mistral"),
            LocalModelType::Gemma => write!(f, "gemma"),
            LocalModelType::Custom => write!(f, "custom"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LocalModelStatus {
    NotDownloaded,
    Downloading,
    Downloaded,
    Installing,
    Ready,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub model_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResponse {
    pub model: LocalModel,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub model_id: Uuid,
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub model_id: Uuid,
    pub output: String,
    pub tokens_used: u32,
    pub inference_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub available_memory_gb: f64,
    pub available_storage_gb: f64,
    pub is_server_running: bool,
    pub server_port: u16,
    pub installed_models: Vec<LocalModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalApiConfig {
    pub enabled: bool,
    pub port: u16,
    pub api_key: Option<String>,
    pub allow_external_access: bool,
}

impl Default for LocalApiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 8081,
            api_key: None,
            allow_external_access: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pConfig {
    pub enabled: bool,
    pub tunnel_mode: P2pTunnelMode,
    pub stun_servers: Vec<String>,
    pub relay_servers: Vec<RelayServerConfig>,
    pub connection_code: Option<String>,
    pub peer_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum P2pTunnelMode {
    StunOnly,
    RelayFallback,
    RelayPreferred,
}

impl Default for P2pConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tunnel_mode: P2pTunnelMode::RelayFallback,
            stun_servers: vec![
                "stun:stun.l.google.com:19302".to_string(),
                "stun:stun1.l.google.com:19302".to_string(),
            ],
            relay_servers: vec![
                RelayServerConfig {
                    url: "wss://relay.williw.ai".to_string(),
                    region: "auto".to_string(),
                },
            ],
            connection_code: None,
            peer_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayServerConfig {
    pub url: String,
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pConnectionInfo {
    pub peer_id: String,
    pub connection_code: String,
    pub public_endpoint: Option<String>,
    pub is_connected: bool,
    pub connected_peers: Vec<PeerInfo>,
    pub connection_quality: ConnectionQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub device_name: String,
    pub endpoint: String,
    pub connected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pTunnelRequest {
    pub host_peer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pTunnelResponse {
    pub tunnel_endpoint: String,
    pub auth_token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pStatus {
    pub is_online: bool,
    pub peer_id: String,
    pub connection_code: String,
    pub active_tunnels: u32,
    pub total_bandwidth_mbps: f64,
    pub relay_usage_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NatType {
    OpenInternet,
    FullCone,
    Restricted,
    PortRestricted,
    Symmetric,
    Unknown,
}

impl std::fmt::Display for NatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NatType::OpenInternet => write!(f, "open_internet"),
            NatType::FullCone => write!(f, "full_cone"),
            NatType::Restricted => write!(f, "restricted"),
            NatType::PortRestricted => write!(f, "port_restricted"),
            NatType::Symmetric => write!(f, "symmetric"),
            NatType::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerEndpoint {
    pub peer_id: String,
    pub public_ip: Option<String>,
    pub public_port: Option<u16>,
    pub nat_type: NatType,
    pub relay_enabled: bool,
    pub local_ip: Option<String>,
    pub local_port: Option<u16>,
}

impl PeerEndpoint {
    pub fn new(peer_id: String) -> Self {
        Self {
            peer_id,
            public_ip: None,
            public_port: None,
            nat_type: NatType::Unknown,
            relay_enabled: false,
            local_ip: None,
            local_port: None,
        }
    }

    pub fn with_public(mut self, ip: String, port: u16) -> Self {
        self.public_ip = Some(ip);
        self.public_port = Some(port);
        self
    }

    pub fn with_local(mut self, ip: String, port: u16) -> Self {
        self.local_ip = Some(ip);
        self.local_port = Some(port);
        self
    }

    pub fn with_nat_type(mut self, nat_type: NatType) -> Self {
        self.nat_type = nat_type;
        self
    }

    pub fn with_relay(mut self, enabled: bool) -> Self {
        self.relay_enabled = enabled;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StunServer {
    pub url: String,
    pub name: String,
}

impl Default for StunServer {
    fn default() -> Self {
        Self {
            url: "stun:stun.l.google.com:19302".to_string(),
            name: "Google STUN".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnServer {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub region: String,
}

impl TurnServer {
    pub fn new(url: String, region: String) -> Self {
        Self {
            url,
            username: None,
            password: None,
            region,
        }
    }

    pub fn with_credentials(mut self, username: String, password: String) -> Self {
        self.username = Some(username);
        self.password = Some(password);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatDiscoveryResult {
    pub external_ip: Option<String>,
    pub external_port: Option<u16>,
    pub nat_type: NatType,
    pub local_ip: String,
    pub local_port: u16,
    pub stun_server_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalingMessage {
    pub msg_type: SignalingMessageType,
    pub from_peer_id: String,
    pub to_peer_id: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SignalingMessageType {
    Register,
    RegisterAck,
    Lookup,
    LookupResult,
    NatInfo,
    NatInfoAck,
    ConnectRequest,
    ConnectAccept,
    ConnectReject,
    KeepAlive,
    Disconnect,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelEstablished {
    pub tunnel_id: String,
    pub local_endpoint: String,
    pub remote_endpoint: String,
    pub relay_used: bool,
    pub connection_quality: ConnectionQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManifest {
    pub models: Vec<ModelManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManifestEntry {
    pub id: Uuid,
    pub name: String,
    pub model_type: LocalModelType,
    pub size_gb: f64,
    pub description: String,
    pub min_memory_gb: f64,
    pub recommended_memory_gb: f64,
    pub quantization: String,
    pub download_url: String,
    pub checksum: String,
}

impl ModelManifest {
    pub fn default_manifest() -> Self {
        Self {
            models: vec![
                ModelManifestEntry {
                    id: Uuid::parse_str("11111111-1111-1111-1111-111111111101").unwrap(),
                    name: "LF2.5 7B".to_string(),
                    model_type: LocalModelType::Lf25,
                    size_gb: 4.2,
                    description: "LightFeather 2.5 - Efficient 7B instruction following model".to_string(),
                    min_memory_gb: 6.0,
                    recommended_memory_gb: 8.0,
                    quantization: "Q4_K_M".to_string(),
                    download_url: "https://models.williw.ai/lf25-7b-q4km.gguf".to_string(),
                    checksum: "sha256:abc123...".to_string(),
                },
                ModelManifestEntry {
                    id: Uuid::parse_str("11111111-1111-1111-1111-111111111102").unwrap(),
                    name: "LF2.5 14B".to_string(),
                    model_type: LocalModelType::Lf25,
                    size_gb: 8.5,
                    description: "LightFeather 2.5 - Powerful 14B instruction following model".to_string(),
                    min_memory_gb: 10.0,
                    recommended_memory_gb: 16.0,
                    quantization: "Q4_K_M".to_string(),
                    download_url: "https://models.williw.ai/lf25-14b-q4km.gguf".to_string(),
                    checksum: "sha256:def456...".to_string(),
                },
                ModelManifestEntry {
                    id: Uuid::parse_str("11111111-1111-1111-1111-111111111103").unwrap(),
                    name: "Gamma 4B".to_string(),
                    model_type: LocalModelType::Gamma4,
                    size_gb: 2.5,
                    description: "Gamma 4B - Fast and lightweight for mobile devices".to_string(),
                    min_memory_gb: 4.0,
                    recommended_memory_gb: 6.0,
                    quantization: "Q4_K_M".to_string(),
                    download_url: "https://models.williw.ai/gamma-4b-q4km.gguf".to_string(),
                    checksum: "sha256:ghi789...".to_string(),
                },
                ModelManifestEntry {
                    id: Uuid::parse_str("11111111-1111-1111-1111-111111111104").unwrap(),
                    name: "Gamma 7B".to_string(),
                    model_type: LocalModelType::Gamma4,
                    size_gb: 4.3,
                    description: "Gamma 7B - Balanced performance for smartphones".to_string(),
                    min_memory_gb: 6.0,
                    recommended_memory_gb: 8.0,
                    quantization: "Q4_K_M".to_string(),
                    download_url: "https://models.williw.ai/gamma-7b-q4km.gguf".to_string(),
                    checksum: "sha256:jkl012...".to_string(),
                },
                ModelManifestEntry {
                    id: Uuid::parse_str("11111111-1111-1111-1111-111111111105").unwrap(),
                    name: "Phi-3.5 Mini".to_string(),
                    model_type: LocalModelType::Phi35,
                    size_gb: 2.3,
                    description: "Microsoft Phi-3.5 Mini - High quality small model".to_string(),
                    min_memory_gb: 4.0,
                    recommended_memory_gb: 6.0,
                    quantization: "Q4_K_M".to_string(),
                    download_url: "https://models.williw.ai/phi35-mini-q4km.gguf".to_string(),
                    checksum: "sha256:mno345...".to_string(),
                },
                ModelManifestEntry {
                    id: Uuid::parse_str("11111111-1111-1111-1111-111111111106").unwrap(),
                    name: "Qwen 2.5 7B".to_string(),
                    model_type: LocalModelType::Qwen25,
                    size_gb: 4.4,
                    description: "Alibaba Qwen 2.5 7B - Strong multilingual support".to_string(),
                    min_memory_gb: 6.0,
                    recommended_memory_gb: 8.0,
                    quantization: "Q4_K_M".to_string(),
                    download_url: "https://models.williw.ai/qwen25-7b-q4km.gguf".to_string(),
                    checksum: "sha256:pqr678...".to_string(),
                },
                ModelManifestEntry {
                    id: Uuid::parse_str("11111111-1111-1111-1111-111111111107").unwrap(),
                    name: "Yi 6B".to_string(),
                    model_type: LocalModelType::Yi,
                    size_gb: 3.8,
                    description: "Yi 6B - Excellent English and Chinese".to_string(),
                    min_memory_gb: 6.0,
                    recommended_memory_gb: 8.0,
                    quantization: "Q4_K_M".to_string(),
                    download_url: "https://models.williw.ai/yi-6b-q4km.gguf".to_string(),
                    checksum: "sha256:stu901...".to_string(),
                },
                ModelManifestEntry {
                    id: Uuid::parse_str("11111111-1111-1111-1111-111111111108").unwrap(),
                    name: "DeepSeek 7B".to_string(),
                    model_type: LocalModelType::Deepseek,
                    size_gb: 4.1,
                    description: "DeepSeek 7B - Great for coding and math".to_string(),
                    min_memory_gb: 6.0,
                    recommended_memory_gb: 8.0,
                    quantization: "Q4_K_M".to_string(),
                    download_url: "https://models.williw.ai/deepseek-7b-q4km.gguf".to_string(),
                    checksum: "sha256:vwx234...".to_string(),
                },
                ModelManifestEntry {
                    id: Uuid::parse_str("11111111-1111-1111-1111-111111111109").unwrap(),
                    name: "Llama 3.2 3B".to_string(),
                    model_type: LocalModelType::Llama,
                    size_gb: 1.9,
                    description: "Meta Llama 3.2 3B - Efficient instruction following".to_string(),
                    min_memory_gb: 4.0,
                    recommended_memory_gb: 6.0,
                    quantization: "Q4_K_M".to_string(),
                    download_url: "https://models.williw.ai/llama32-3b-q4km.gguf".to_string(),
                    checksum: "sha256:yza567...".to_string(),
                },
                ModelManifestEntry {
                    id: Uuid::parse_str("11111111-1111-1111-1111-111111111110").unwrap(),
                    name: "Mistral 7B".to_string(),
                    model_type: LocalModelType::Mistral,
                    size_gb: 4.3,
                    description: "Mistral 7B - Well-rounded performance".to_string(),
                    min_memory_gb: 6.0,
                    recommended_memory_gb: 8.0,
                    quantization: "Q4_K_M".to_string(),
                    download_url: "https://models.williw.ai/mistral-7b-q4km.gguf".to_string(),
                    checksum: "sha256:bcd890...".to_string(),
                },
                ModelManifestEntry {
                    id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
                    name: "Gemma 2B".to_string(),
                    model_type: LocalModelType::Gemma,
                    size_gb: 1.4,
                    description: "Google Gemma 2B - Google's efficient model".to_string(),
                    min_memory_gb: 3.0,
                    recommended_memory_gb: 4.0,
                    quantization: "Q4_K_M".to_string(),
                    download_url: "https://models.williw.ai/gemma-2b-q4km.gguf".to_string(),
                    checksum: "sha256:efg123...".to_string(),
                },
            ],
        }
    }
}
