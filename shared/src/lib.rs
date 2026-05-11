use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// 用户结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// 用户唯一标识符
    pub id: Uuid,
    /// 钱包地址
    pub wallet_address: String,
    /// 邮箱（可选）
    pub email: Option<String>,
    /// 账户余额
    pub balance: f64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl User {
    /// 创建新用户
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

/// AI模型结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModel {
    /// 模型ID
    pub id: Uuid,
    /// 模型名称
    pub name: String,
    /// 提供商
    pub provider: String,
    /// 模型类别
    pub category: ModelCategory,
    /// 模型描述
    pub description: String,
    /// 算力值
    pub compute_power: f64,
    /// 单价
    pub price_per_unit: f64,
    /// 模型状态
    pub status: ModelStatus,
    /// 图片URL
    pub image_url: Option<String>,
}

/// 模型类别枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelCategory {
    /// 大语言模型
    Llm,
    /// 图像模型
    Image,
    /// 音频模型
    Audio,
    /// 视频模型
    Video,
    /// 多模态模型
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

/// 模型状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelStatus {
    /// 活跃
    Active,
    /// 维护中
    Maintenance,
    /// 已弃用
    Deprecated,
}

/// 计算请求结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeRequest {
    /// 请求ID
    pub id: Uuid,
    /// 用户ID
    pub user_id: Uuid,
    /// 模型ID
    pub model_id: Uuid,
    /// 计算数量
    pub amount: f64,
    /// 请求状态
    pub status: ComputeStatus,
    /// 结果（可选）
    pub result: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 计算状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ComputeStatus {
    /// 待处理
    Pending,
    /// 处理中
    Processing,
    /// 已完成
    Completed,
    /// 失败
    Failed,
}

/// 订单结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// 订单ID
    pub id: Uuid,
    /// 用户ID
    pub user_id: Uuid,
    /// 模型ID
    pub model_id: Uuid,
    /// 数量
    pub amount: f64,
    /// 支付方式
    pub payment_method: PaymentMethod,
    /// 订单状态
    pub status: OrderStatus,
    /// 加密货币数量（可选）
    pub crypto_amount: Option<f64>,
    /// 加密货币类型（可选）
    pub crypto_currency: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 支付方式枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    /// 微信支付
    Wechat,
    /// 支付宝
    Alipay,
    /// USDT
    Usdt,
    /// 以太坊
    Eth,
    /// 比特币
    Btc,
}

/// 订单状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// 待支付
    Pending,
    /// 已支付
    Paid,
    /// 处理中
    Processing,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已退款
    Refunded,
}

/// 模型过滤条件结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFilter {
    /// 类别过滤
    pub category: Option<ModelCategory>,
    /// 提供商过滤
    pub provider: Option<String>,
    /// 最小算力
    pub min_power: Option<f64>,
    /// 最大价格
    pub max_price: Option<f64>,
    /// 搜索关键词
    pub search: Option<String>,
}

/// 钱包登录请求结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletLoginRequest {
    /// 钱包地址
    pub wallet_address: String,
    /// 签名
    pub signature: String,
    /// 消息
    pub message: String,
}

/// 钱包登录响应结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletLoginResponse {
    /// JWT令牌
    pub token: String,
    /// 用户信息
    pub user: User,
}

/// 支付请求结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    /// 订单ID
    pub order_id: Uuid,
    /// 支付方式
    pub method: PaymentMethod,
}

/// 支付响应结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    /// 二维码链接
    pub qr_code: Option<String>,
    /// 支付链接
    pub payment_url: Option<String>,
    /// 加密货币地址信息
    pub crypto_address: Option<CryptoPaymentInfo>,
    /// 订单ID
    pub order_id: Uuid,
}

/// 加密货币支付信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoPaymentInfo {
    /// 收款地址
    pub address: String,
    /// 金额
    pub amount: f64,
    /// 货币类型
    pub currency: String,
    /// 网络类型
    pub network: String,
}

/// API响应封装结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// 是否成功
    pub success: bool,
    /// 响应数据
    pub data: Option<T>,
    /// 错误信息
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    /// 创建成功响应
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// 创建错误响应
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

/// 本地模型结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModel {
    /// 模型ID
    pub id: Uuid,
    /// 模型名称
    pub name: String,
    /// 模型类型
    pub model_type: LocalModelType,
    /// 模型大小（GB）
    pub size_gb: f64,
    /// 下载状态
    pub status: LocalModelStatus,
    /// 下载进度
    pub download_progress: Option<f32>,
    /// 下载URL
    pub download_url: String,
    /// 本地路径
    pub local_path: Option<String>,
    /// 推理端点
    pub inference_endpoint: Option<String>,
    /// 是否默认模型
    pub is_default: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 本地模型类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LocalModelType {
    /// LF2.5
    Lf25,
    /// Gamma4
    Gamma4,
    /// Phi3.5
    Phi35,
    /// Qwen2.5
    Qwen25,
    /// Yi
    Yi,
    /// Deepseek
    Deepseek,
    /// Llama
    Llama,
    /// Mistral
    Mistral,
    /// Gemma
    Gemma,
    /// 自定义
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

/// 本地模型状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LocalModelStatus {
    /// 未下载
    NotDownloaded,
    /// 下载中
    Downloading,
    /// 已下载
    Downloaded,
    /// 安装中
    Installing,
    /// 就绪
    Ready,
    /// 错误
    Error,
}

/// 下载请求结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    /// 模型ID
    pub model_id: Uuid,
}

/// 下载响应结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResponse {
    /// 模型信息
    pub model: LocalModel,
    /// 下载URL
    pub download_url: String,
}

/// 推理请求结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// 模型ID
    pub model_id: Uuid,
    /// 提示词
    pub prompt: String,
    /// 最大token数
    pub max_tokens: Option<u32>,
    /// 温度参数
    pub temperature: Option<f32>,
}

/// 推理响应结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    /// 模型ID
    pub model_id: Uuid,
    /// 输出内容
    pub output: String,
    /// 使用的token数
    pub tokens_used: u32,
    /// 推理耗时（毫秒）
    pub inference_time_ms: u64,
}

/// 设备信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// 设备ID
    pub device_id: String,
    /// 设备名称
    pub device_name: String,
    /// 可用内存（GB）
    pub available_memory_gb: f64,
    /// 可用存储（GB）
    pub available_storage_gb: f64,
    /// 服务器是否运行
    pub is_server_running: bool,
    /// 服务器端口
    pub server_port: u16,
    /// 已安装模型列表
    pub installed_models: Vec<LocalModel>,
}

/// 本地API服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalApiConfig {
    /// 是否启用
    pub enabled: bool,
    /// 端口号
    pub port: u16,
    /// API密钥
    pub api_key: Option<String>,
    /// 是否允许外部访问
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

/// P2P配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pConfig {
    /// 是否启用
    pub enabled: bool,
    /// 隧道模式
    pub tunnel_mode: P2pTunnelMode,
    /// STUN服务器列表
    pub stun_servers: Vec<String>,
    /// 中继服务器列表
    pub relay_servers: Vec<RelayServerConfig>,
    /// 连接码
    pub connection_code: Option<String>,
    /// 节点ID
    pub peer_id: Option<String>,
}

/// P2P隧道模式枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum P2pTunnelMode {
    /// 仅STUN
    StunOnly,
    /// 中继回退
    RelayFallback,
    /// 中继优先
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

/// 中继服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayServerConfig {
    /// 服务器URL
    pub url: String,
    /// 区域
    pub region: String,
}

/// P2P连接信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pConnectionInfo {
    /// 节点ID
    pub peer_id: String,
    /// 连接码
    pub connection_code: String,
    /// 公网端点
    pub public_endpoint: Option<String>,
    /// 是否已连接
    pub is_connected: bool,
    /// 已连接节点列表
    pub connected_peers: Vec<PeerInfo>,
    /// 连接质量
    pub connection_quality: ConnectionQuality,
}

/// 节点信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// 节点ID
    pub peer_id: String,
    /// 设备名称
    pub device_name: String,
    /// 端点地址
    pub endpoint: String,
    /// 连接时间
    pub connected_at: DateTime<Utc>,
}

/// 连接质量枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionQuality {
    /// 优秀
    Excellent,
    /// 良好
    Good,
    /// 一般
    Fair,
    /// 较差
    Poor,
    /// 断开
    Disconnected,
}

/// P2P隧道请求结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pTunnelRequest {
    /// 主机节点ID
    pub host_peer_id: String,
}

/// P2P隧道响应结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pTunnelResponse {
    /// 隧道端点
    pub tunnel_endpoint: String,
    /// 认证令牌
    pub auth_token: String,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
}

/// P2P状态结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pStatus {
    /// 是否在线
    pub is_online: bool,
    /// 节点ID
    pub peer_id: String,
    /// 连接码
    pub connection_code: String,
    /// 活跃隧道数
    pub active_tunnels: u32,
    /// 总带宽（Mbps）
    pub total_bandwidth_mbps: f64,
    /// 中继使用百分比
    pub relay_usage_percent: Option<f64>,
}

/// NAT类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NatType {
    /// 开放网络
    OpenInternet,
    /// 全锥型NAT
    FullCone,
    /// 受限锥型NAT
    Restricted,
    /// 端口受限锥型NAT
    PortRestricted,
    /// 对称型NAT
    Symmetric,
    /// 未知
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

/// 对等节点端点结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerEndpoint {
    /// 节点ID
    pub peer_id: String,
    /// 公网IP
    pub public_ip: Option<String>,
    /// 公网端口
    pub public_port: Option<u16>,
    /// NAT类型
    pub nat_type: NatType,
    /// 是否启用中继
    pub relay_enabled: bool,
    /// 本地IP
    pub local_ip: Option<String>,
    /// 本地端口
    pub local_port: Option<u16>,
}

impl PeerEndpoint {
    /// 创建新的端点
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

    /// 设置公网端点
    pub fn with_public(mut self, ip: String, port: u16) -> Self {
        self.public_ip = Some(ip);
        self.public_port = Some(port);
        self
    }

    /// 设置本地端点
    pub fn with_local(mut self, ip: String, port: u16) -> Self {
        self.local_ip = Some(ip);
        self.local_port = Some(port);
        self
    }

    /// 设置NAT类型
    pub fn with_nat_type(mut self, nat_type: NatType) -> Self {
        self.nat_type = nat_type;
        self
    }

    /// 设置是否启用中继
    pub fn with_relay(mut self, enabled: bool) -> Self {
        self.relay_enabled = enabled;
        self
    }
}

/// STUN服务器结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StunServer {
    /// 服务器URL
    pub url: String,
    /// 服务器名称
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

/// TURN服务器结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnServer {
    /// 服务器URL
    pub url: String,
    /// 用户名
    pub username: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 区域
    pub region: String,
}

impl TurnServer {
    /// 创建新的TURN服务器
    pub fn new(url: String, region: String) -> Self {
        Self {
            url,
            username: None,
            password: None,
            region,
        }
    }

    /// 设置凭证
    pub fn with_credentials(mut self, username: String, password: String) -> Self {
        self.username = Some(username);
        self.password = Some(password);
        self
    }
}

/// NAT发现结果结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatDiscoveryResult {
    /// 公网IP
    pub external_ip: Option<String>,
    /// 公网端口
    pub external_port: Option<u16>,
    /// NAT类型
    pub nat_type: NatType,
    /// 本地IP
    pub local_ip: String,
    /// 本地端口
    pub local_port: u16,
    /// 使用的STUN服务器
    pub stun_server_used: String,
}

/// 信令消息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalingMessage {
    /// 消息类型
    pub msg_type: SignalingMessageType,
    /// 发送者节点ID
    pub from_peer_id: String,
    /// 接收者节点ID
    pub to_peer_id: Option<String>,
    /// 负载数据
    pub payload: Option<serde_json::Value>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 信令消息类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SignalingMessageType {
    /// 注册
    Register,
    /// 注册确认
    RegisterAck,
    /// 查询
    Lookup,
    /// 查询结果
    LookupResult,
    /// NAT信息
    NatInfo,
    /// NAT信息确认
    NatInfoAck,
    /// 连接请求
    ConnectRequest,
    /// 连接接受
    ConnectAccept,
    /// 连接拒绝
    ConnectReject,
    /// 心跳
    KeepAlive,
    /// 断开
    Disconnect,
    /// 错误
    Error,
}

/// 隧道建立信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelEstablished {
    /// 隧道ID
    pub tunnel_id: String,
    /// 本地端点
    pub local_endpoint: String,
    /// 远端端点
    pub remote_endpoint: String,
    /// 是否使用中继
    pub relay_used: bool,
    /// 连接质量
    pub connection_quality: ConnectionQuality,
}

/// 模型清单结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManifest {
    /// 模型列表
    pub models: Vec<ModelManifestEntry>,
}

/// 模型清单条目结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManifestEntry {
    /// 模型ID
    pub id: Uuid,
    /// 模型名称
    pub name: String,
    /// 模型类型
    pub model_type: LocalModelType,
    /// 模型大小（GB）
    pub size_gb: f64,
    /// 模型描述
    pub description: String,
    /// 最低内存要求（GB）
    pub min_memory_gb: f64,
    /// 推荐内存（GB）
    pub recommended_memory_gb: f64,
    /// 量化方法
    pub quantization: String,
    /// 下载URL
    pub download_url: String,
    /// 校验和
    pub checksum: String,
}

impl ModelManifest {
    /// 获取默认模型清单
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