use serde::{Deserialize, Serialize};
use uuid::Uuid;
use williw_shared::*;

/// API基础URL
const API_BASE: &str = "http://localhost:8080/api";

/// API错误结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    /// 错误信息
    pub error: String,
}

/// API客户端
/// 用于与后端API通信
pub struct ApiClient {
    /// 基础URL
    base_url: String,
    /// JWT令牌
    token: Option<String>,
}

impl ApiClient {
    /// 创建新的API客户端
    pub fn new() -> Self {
        Self {
            base_url: API_BASE.to_string(),
            token: None,
        }
    }

    /// 创建带令牌的API客户端
    pub fn with_token(token: String) -> Self {
        Self {
            base_url: API_BASE.to_string(),
            token: Some(token),
        }
    }

    /// 设置JWT令牌
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    /// 发送GET请求
    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, String> {
        let url = format!("{}{}", self.base_url, path);
        let client = reqwest::Client::new();

        let mut req = client.get(&url);

        // 添加认证头
        if let Some(ref token) = self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let response = req.send().await.map_err(|e| e.to_string())?;

        if response.status().is_success() {
            response.json().await.map_err(|e| e.to_string())
        } else {
            Err(response.text().await.unwrap_or_else(|_| "Unknown error".to_string()))
        }
    }

    /// 发送POST请求
    async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(&self, path: &str, body: &B) -> Result<T, String> {
        let url = format!("{}{}", self.base_url, path);
        let client = reqwest::Client::new();

        let mut req = client.post(&url).json(body);

        // 添加认证头
        if let Some(ref token) = self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let response = req.send().await.map_err(|e| e.to_string())?;

        if response.status().is_success() {
            response.json().await.map_err(|e| e.to_string())
        } else {
            Err(response.text().await.unwrap_or_else(|_| "Unknown error".to_string()))
        }
    }

    /// 钱包登录
    pub async fn wallet_login(&self, wallet_address: &str, signature: &str, message: &str) -> Result<LoginResponse, String> {
        let request = WalletLoginRequest {
            wallet_address: wallet_address.to_string(),
            signature: signature.to_string(),
            message: message.to_string(),
        };

        let response: ApiResponse<WalletLoginResponse> = self.post("/auth/wallet/login", &request).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 获取AI模型列表
    pub async fn get_models(&self, filter: Option<ModelFilter>) -> Result<Vec<AiModel>, String> {
        let mut path = "/compute/models".to_string();
        // 构建查询参数
        if let Some(f) = filter {
            let mut params = vec![];
            if let Some(ref cat) = f.category {
                params.push(format!("category={}", cat));
            }
            if let Some(ref provider) = f.provider {
                params.push(format!("provider={}", provider));
            }
            if let Some(min) = f.min_power {
                params.push(format!("min_power={}", min));
            }
            if let Some(max) = f.max_price {
                params.push(format!("max_price={}", max));
            }
            if let Some(ref search) = f.search {
                params.push(format!("search={}", search));
            }
            if !params.is_empty() {
                path = format!("{}?{}", path, params.join("&"));
            }
        }

        let response: ApiResponse<Vec<AiModel>> = self.get(&path).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 获取指定模型
    pub async fn get_model(&self, id: Uuid) -> Result<AiModel, String> {
        let response: ApiResponse<AiModel> = self.get(&format!("/compute/models/{}", id)).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 创建订单
    pub async fn create_order(&self, model_id: Uuid, amount: f64, method: PaymentMethod) -> Result<Order, String> {
        let request = CreateOrderRequest { model_id, amount, payment_method: method };
        let response: ApiResponse<Order> = self.post("/payment/create", &request).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 发起支付
    pub async fn initiate_payment(&self, order_id: Uuid, method: PaymentMethod) -> Result<PaymentResponse, String> {
        let request = PaymentRequest { order_id, method };
        let response: ApiResponse<PaymentResponse> = self.post("/payment/initiate", &request).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 获取订单状态
    pub async fn get_order_status(&self, order_id: Uuid) -> Result<Order, String> {
        let response: ApiResponse<Order> = self.get(&format!("/payment/status/{}", order_id)).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 获取用户订单列表
    pub async fn get_user_orders(&self) -> Result<Vec<Order>, String> {
        let response: ApiResponse<Vec<Order>> = self.get("/payment/orders").await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 获取用户资料
    pub async fn get_profile(&self) -> Result<ProfileResponse, String> {
        let response: ApiResponse<ProfileResponse> = self.get("/auth/profile").await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 创建计算请求
    pub async fn create_compute_request(&self, model_id: Uuid, amount: f64) -> Result<ComputeRequest, String> {
        let request = ComputeRequestCreate { model_id, amount };
        let response: ApiResponse<ComputeRequest> = self.post("/compute/request", &request).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 获取本地模型列表
    pub async fn get_local_models(&self) -> Result<Vec<LocalModel>, String> {
        let response: ApiResponse<Vec<LocalModel>> = self.get("/local/models").await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 获取指定本地模型
    pub async fn get_local_model(&self, id: Uuid) -> Result<LocalModel, String> {
        let response: ApiResponse<LocalModel> = self.get(&format!("/local/models/{}", id)).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 下载模型
    pub async fn download_model(&self, model_id: Uuid) -> Result<LocalModel, String> {
        let request = DownloadRequest { model_id };
        let response: ApiResponse<LocalModel> = self.post("/local/models/download", &request).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 删除本地模型
    pub async fn delete_local_model(&self, model_id: Uuid) -> Result<(), String> {
        let _: ApiResponse<()> = self.delete(&format!("/local/models/{}", model_id)).await?;
        Ok(())
    }

    /// 发送DELETE请求
    async fn delete<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, String> {
        let url = format!("{}{}", self.base_url, path);
        let client = reqwest::Client::new();

        let mut req = client.delete(&url);

        if let Some(ref token) = self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let response = req.send().await.map_err(|e| e.to_string())?;

        if response.status().is_success() {
            response.json().await.map_err(|e| e.to_string())
        } else {
            Err(response.text().await.unwrap_or_else(|_| "Unknown error".to_string()))
        }
    }

    /// 设置默认模型
    pub async fn set_default_model(&self, model_id: Uuid) -> Result<LocalModel, String> {
        let response: ApiResponse<LocalModel> = self.post(&format!("/local/models/{}/set-default", model_id), &()).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 获取模型清单
    pub async fn get_model_manifest(&self) -> Result<ModelManifest, String> {
        let response: ApiResponse<ModelManifest> = self.get("/local/manifest").await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 获取设备信息
    pub async fn get_device_info(&self) -> Result<DeviceInfo, String> {
        let response: ApiResponse<DeviceInfo> = self.get("/local/device-info").await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 启动本地服务器
    pub async fn start_local_server(&self) -> Result<String, String> {
        let response: ApiResponse<String> = self.post("/local/server/start", &()).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 停止本地服务器
    pub async fn stop_local_server(&self) -> Result<String, String> {
        let response: ApiResponse<String> = self.post("/local/server/stop", &()).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 获取服务器配置
    pub async fn get_server_config(&self) -> Result<LocalApiConfig, String> {
        let response: ApiResponse<LocalApiConfig> = self.get("/local/server/config").await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 更新服务器配置
    pub async fn update_server_config(&self, config: LocalApiConfig) -> Result<LocalApiConfig, String> {
        let response: ApiResponse<LocalApiConfig> = self.post("/local/server/config", &config).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 运行推理
    pub async fn run_inference(&self, model_id: Uuid, prompt: String, max_tokens: Option<u32>, temperature: Option<f32>) -> Result<InferenceResponse, String> {
        let request = InferenceRequest { model_id, prompt, max_tokens, temperature };
        let response: ApiResponse<InferenceResponse> = self.post("/local/inference", &request).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 获取P2P状态
    pub async fn get_p2p_status(&self) -> Result<P2pStatus, String> {
        let response: ApiResponse<P2pStatus> = self.get("/p2p/status").await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 获取P2P配置
    pub async fn get_p2p_config(&self) -> Result<P2pConfig, String> {
        let response: ApiResponse<P2pConfig> = self.get("/p2p/config").await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 更新P2P配置
    pub async fn update_p2p_config(&self, config: P2pConfig) -> Result<P2pConfig, String> {
        let response: ApiResponse<P2pConfig> = self.post("/p2p/config", &config).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// P2P上线
    pub async fn p2p_go_online(&self) -> Result<P2pConnectionInfo, String> {
        let response: ApiResponse<P2pConnectionInfo> = self.post("/p2p/online", &()).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// P2P下线
    pub async fn p2p_go_offline(&self) -> Result<String, String> {
        let response: ApiResponse<String> = self.post("/p2p/offline", &()).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 获取P2P连接信息
    pub async fn get_p2p_connection_info(&self) -> Result<P2pConnectionInfo, String> {
        let response: ApiResponse<P2pConnectionInfo> = self.get("/p2p/connection-info").await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 连接到对等节点
    pub async fn connect_to_peer(&self, peer_id: String) -> Result<P2pTunnelResponse, String> {
        let response: ApiResponse<P2pTunnelResponse> = self.post(&format!("/p2p/connect/{}", peer_id), &()).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 共享P2P连接
    pub async fn share_p2p_connection(&self, host_peer_id: String) -> Result<P2pTunnelResponse, String> {
        let request = P2pTunnelRequest { host_peer_id };
        let response: ApiResponse<P2pTunnelResponse> = self.post("/p2p/share", &request).await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// 断开P2P隧道
    pub async fn disconnect_p2p_tunnel(&self, tunnel_id: String) -> Result<(), String> {
        let _: ApiResponse<()> = self.delete(&format!("/p2p/tunnel/{}", tunnel_id)).await?;
        Ok(())
    }

    /// 测试P2P连接
    pub async fn test_p2p_connection(&self) -> Result<ConnectionQuality, String> {
        let response: ApiResponse<ConnectionQuality> = self.get("/p2p/test").await?;
        response.data.ok_or(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

/// 创建订单请求结构体
#[derive(Debug, Serialize)]
pub struct CreateOrderRequest {
    /// 模型ID
    pub model_id: Uuid,
    /// 数量
    pub amount: f64,
    /// 支付方式
    pub payment_method: PaymentMethod,
}

/// 用户资料响应结构体
#[derive(Debug, Deserialize)]
pub struct ProfileResponse {
    /// 用户信息
    pub user: User,
    /// 总订单数
    pub total_orders: i64,
    /// 总消费
    pub total_spent: f64,
}

/// 计算请求创建结构体
#[derive(Debug, Serialize)]
pub struct ComputeRequestCreate {
    /// 模型ID
    pub model_id: Uuid,
    /// 数量
    pub amount: f64,
}