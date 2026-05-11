use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;
use williw_shared::{
    ConnectionQuality, NatDiscoveryResult, NatType, PeerEndpoint, SignalingMessage,
    SignalingMessageType, TunnelEstablished,
};
use futures_util::{SinkExt, StreamExt};

/// WebSocket信令客户端
/// 用于P2P连接建立时的节点发现和协商
pub struct SignalingClient {
    /// WebSocket服务器URL
    ws_url: String,
    /// 本节点ID
    peer_id: String,
    /// 连接码
    connection_code: Option<String>,
    /// 连接状态
    connected: Arc<RwLock<bool>>,
    /// 消息发送通道
    sender: Arc<RwLock<Option<mpsc::Sender<SignalingMessage>>>>,
    /// 消息接收通道
    receiver: Arc<RwLock<Option<mpsc::Receiver<SignalingMessage>>>>,
}

impl SignalingClient {
    /// 创建新的信令客户端
    /// 
    /// # 参数
    /// * `relay_url` - 中继服务器URL
    /// * `peer_id` - 本节点ID
    pub fn new(relay_url: &str, peer_id: String) -> Self {
        Self {
            ws_url: format!("{}/api/p2p/ws", relay_url.trim_end_matches('/')),
            peer_id,
            connection_code: None,
            connected: Arc::new(RwLock::new(false)),
            sender: Arc::new(RwLock::new(None)),
            receiver: Arc::new(RwLock::new(None)),
        }
    }

    /// 连接到信令服务器
    /// 建立WebSocket连接并启动消息处理循环
    pub async fn connect(&mut self) -> Result<(), String> {
        let url = Url::parse(&self.ws_url)
            .map_err(|e| format!("Invalid WebSocket URL: {}", e))?;

        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| format!("WebSocket connection failed: {}", e))?;

        let (write, read) = ws_stream.split();

        // 创建双向消息通道
        let (tx, rx) = mpsc::channel::<SignalingMessage>(100);
        let (out_tx, mut out_rx) = mpsc::channel::<SignalingMessage>(100);

        let peer_id = self.peer_id.clone();
        // 启动异步消息处理任务
        tokio::spawn(async move {
            let mut read = read;
            let mut out_rx = out_rx;
            let mut tx = tx;

            loop {
                tokio::select! {
                    // 处理接收到的WebSocket消息
                    msg = read.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                if let Ok(sig_msg) = serde_json::from_str::<SignalingMessage>(&text) {
                                    let _ = tx.send(sig_msg).await;
                                }
                            }
                            Some(Ok(Message::Close(_))) | None => {
                                break;
                            }
                            _ => {}
                        }
                    }
                    // 处理待发送的消息
                    msg = out_rx.recv() => {
                        if let Some(sig_msg) = msg {
                            let json = serde_json::to_string(&sig_msg).unwrap_or_default();
                            let _ = write.send(Message::Text(json)).await;
                        }
                    }
                }
            }
        });

        *self.sender.write().await = Some(out_tx);
        *self.receiver.write().await = Some(rx);
        *self.connected.write().await = true;

        // 发送注册消息
        self.send_register().await?;

        Ok(())
    }

    /// 断开与信令服务器的连接
    pub async fn disconnect(&mut self) -> Result<(), String> {
        *self.connected.write().await = false;
        *self.sender.write().await = None;
        *self.receiver.write().await = None;
        Ok(())
    }

    /// 检查是否已连接
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// 发送注册消息到信令服务器
    async fn send_register(&mut self) -> Result<(), String> {
        let msg = SignalingMessage {
            msg_type: SignalingMessageType::Register,
            from_peer_id: self.peer_id.clone(),
            to_peer_id: None,
            payload: None,
            timestamp: chrono::Utc::now(),
        };

        self.send(msg).await
    }

    /// 使用连接码注册到信令服务器
    /// 
    /// # 参数
    /// * `code` - 连接码
    pub async fn register_connection_code(&mut self, code: &str) -> Result<(), String> {
        self.connection_code = Some(code.to_string());

        let payload = serde_json::json!({
            "connection_code": code
        });

        let msg = SignalingMessage {
            msg_type: SignalingMessageType::Register,
            from_peer_id: self.peer_id.clone(),
            to_peer_id: None,
            payload: Some(payload),
            timestamp: chrono::Utc::now(),
        };

        self.send(msg).await
    }

    /// 根据连接码查找对等节点
    /// 
    /// # 参数
    /// * `connection_code` - 目标节点的连接码
    /// 
    /// # 返回
    /// 找到返回节点端点信息
    pub async fn lookup_peer(&mut self, connection_code: &str) -> Result<PeerEndpoint, String> {
        let payload = serde_json::json!({
            "connection_code": connection_code
        });

        let msg = SignalingMessage {
            msg_type: SignalingMessageType::Lookup,
            from_peer_id: self.peer_id.clone(),
            to_peer_id: None,
            payload: Some(payload),
            timestamp: chrono::Utc::now(),
        };

        self.send(msg).await?;

        if let Some(receiver) = self.receiver.write().await.as_mut() {
            if let Some(response) = receiver.recv().await {
                if response.msg_type == SignalingMessageType::LookupResult {
                    if let Some(payload) = response.payload {
                        let endpoint = serde_json::from_value::<PeerEndpoint>(payload)
                            .map_err(|e| format!("Failed to parse peer endpoint: {}", e))?;
                        return Ok(endpoint);
                    }
                }
            }
        }

        Err("No response from lookup".to_string())
    }

    /// 发送NAT穿透信息到信令服务器
    /// 
    /// # 参数
    /// * `nat_info` - NAT发现结果
    pub async fn send_nat_info(&mut self, nat_info: &NatDiscoveryResult) -> Result<(), String> {
        let payload = serde_json::json!({
            "nat_type": nat_info.nat_type.to_string(),
            "external_ip": nat_info.external_ip,
            "external_port": nat_info.external_port,
            "local_ip": nat_info.local_ip,
            "local_port": nat_info.local_port,
        });

        let msg = SignalingMessage {
            msg_type: SignalingMessageType::NatInfo,
            from_peer_id: self.peer_id.clone(),
            to_peer_id: None,
            payload: Some(payload),
            timestamp: chrono::Utc::now(),
        };

        self.send(msg).await
    }

    /// 向目标节点发起连接请求
    /// 
    /// # 参数
    /// * `host_peer_id` - 目标节点ID
    /// 
    /// # 返回
    /// 连接成功返回隧道建立信息
    pub async fn request_connection(&mut self, host_peer_id: &str) -> Result<TunnelEstablished, String> {
        let payload = serde_json::json!({
            "host_peer_id": host_peer_id
        });

        let msg = SignalingMessage {
            msg_type: SignalingMessageType::ConnectRequest,
            from_peer_id: self.peer_id.clone(),
            to_peer_id: Some(host_peer_id.to_string()),
            payload: Some(payload),
            timestamp: chrono::Utc::now(),
        };

        self.send(msg).await?;

        if let Some(receiver) = self.receiver.write().await.as_mut() {
            while let Some(response) = receiver.recv().await {
                match response.msg_type {
                    SignalingMessageType::ConnectAccept => {
                        if let Some(payload) = response.payload {
                            let tunnel = serde_json::from_value::<TunnelEstablished>(payload)
                                .map_err(|e| format!("Failed to parse tunnel: {}", e))?;
                            return Ok(tunnel);
                        }
                    }
                    SignalingMessageType::ConnectReject => {
                        return Err("Connection rejected by host".to_string());
                    }
                    _ => continue,
                }
            }
        }

        Err("Connection request timed out".to_string())
    }

    /// 接受来自客户端的连接请求
    /// 
    /// # 参数
    /// * `client_peer_id` - 客户端节点ID
    pub async fn accept_connection(&mut self, client_peer_id: &str) -> Result<TunnelEstablished, String> {
        let payload = serde_json::json!({
            "client_peer_id": client_peer_id,
            "relay_used": true,
            "connection_quality": "good"
        });

        let msg = SignalingMessage {
            msg_type: SignalingMessageType::ConnectAccept,
            from_peer_id: self.peer_id.clone(),
            to_peer_id: Some(client_peer_id.to_string()),
            payload: Some(payload),
            timestamp: chrono::Utc::now(),
        };

        self.send(msg).await
    }

    /// 发送心跳保活消息
    pub async fn send_keepalive(&mut self) -> Result<(), String> {
        let msg = SignalingMessage {
            msg_type: SignalingMessageType::KeepAlive,
            from_peer_id: self.peer_id.clone(),
            to_peer_id: None,
            payload: None,
            timestamp: chrono::Utc::now(),
        };

        self.send(msg).await
    }

    /// 发送消息到信令服务器
    async fn send(&mut self, msg: SignalingMessage) -> Result<(), String> {
        let sender = self.sender.read().await;
        let sender = sender
            .as_ref()
            .ok_or("WebSocket not connected")?;

        sender
            .send(msg)
            .await
            .map_err(|e| format!("Failed to send message: {}", e))
    }
}

/// 信令服务器（用于模拟或本地测试）
/// 维护节点信息和NAT类型
pub struct SignalingServer {
    /// 节点ID
    pub peer_id: String,
    /// 连接码
    pub connection_code: String,
    /// NAT类型
    pub nat_type: NatType,
    /// 公网IP地址
    pub external_ip: Option<String>,
    /// 公网端口
    pub external_port: Option<u16>,
    /// 是否启用中继
    pub relay_enabled: bool,
}

impl SignalingServer {
    /// 创建新的信令服务器实例
    /// 
    /// # 参数
    /// * `peer_id` - 节点ID
    /// * `connection_code` - 连接码
    pub fn new(peer_id: String, connection_code: String) -> Self {
        Self {
            peer_id,
            connection_code,
            nat_type: NatType::Unknown,
            external_ip: None,
            external_port: None,
            relay_enabled: true,
        }
    }

    /// 设置NAT信息
    /// 根据NAT类型决定是否启用中继
    pub fn with_nat_info(mut self, nat_info: NatDiscoveryResult) -> Self {
        self.nat_type = nat_info.nat_type;
        self.external_ip = nat_info.external_ip;
        self.external_port = nat_info.external_port;
        // 对称型NAT或端口受限NAT需要中继
        self.relay_enabled = self.nat_type == NatType::Symmetric
            || self.nat_type == NatType::PortRestricted
            || self.nat_type == NatType::Restricted;
        self
    }

    /// 转换为对等节点端点信息
    pub fn to_peer_endpoint(&self) -> PeerEndpoint {
        PeerEndpoint::new(self.peer_id.clone())
            .with_nat_type(self.nat_type.clone())
            .with_relay(self.relay_enabled)
            .with_public(
                self.external_ip.clone().unwrap_or_else(|| "0.0.0.0".to_string()),
                self.external_port.unwrap_or(0),
            )
    }
}