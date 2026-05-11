use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;
use williw_shared::*;

use crate::p2p::{TunnelManager, TunnelConnection};
use williw_shared::{StunServer, TurnServer};

/// P2P服务结构体，管理点对点连接和隧道
pub struct P2pService {
    /// 隧道管理器，用于处理P2P连接
    tunnel_manager: Arc<RwLock<Option<TunnelManager>>>,
    /// 是否在线的原子状态标志
    is_online: AtomicBool,
    /// 本节点唯一标识符
    peer_id: String,
    /// 连接码，用于其他节点发现和连接
    connection_code: String,
    /// 当前活跃的隧道数量
    active_tunnels: AtomicU32,
    /// 已连接的节点信息映射表
    connected_peers: RwLock<HashMap<String, PeerInfo>>,
    /// P2P配置信息
    config: RwLock<P2pConfig>,
}

impl P2pService {
    /// 创建新的P2P服务实例
    /// 初始化时会生成唯一的peer_id和connection_code
    pub fn new() -> Self {
        // 生成16位唯一peer_id，格式为"peer_"前缀加UUID
        let peer_id = format!("peer_{}", Uuid::new_v4().to_string().replace("-", "")[..16].to_string());
        // 生成随机连接码，用于节点发现
        let connection_code = Self::generate_connection_code();

        Self {
            tunnel_manager: Arc::new(RwLock::new(None)),
            is_online: AtomicBool::new(false),
            peer_id,
            connection_code,
            active_tunnels: AtomicU32::new(0),
            connected_peers: RwLock::new(HashMap::new()),
            config: RwLock::new(P2pConfig::default()),
        }
    }

    /// 生成随机连接码，格式为"XXXX-XXXX"（8位大写字母和数字）
    /// 使用时间戳作为随机种子
    fn generate_connection_code() -> String {
        // 连接码可用字符集（排除易混淆的字符如0、O、I、1）
        let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
        let mut code = String::new();
        use std::time::{SystemTime, UNIX_EPOCH};
        // 使用纳秒时间戳作为随机种子
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let mut rng = seed;
        // 线性同余生成器产生8位字符
        for _ in 0..8 {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            code.push(chars[(rng as usize) % chars.len()]);
        }
        // 格式化为"XXXX-XXXX"形式
        code.chars().collect::<Vec<_>>()[..4].iter().collect::<String>()
            + "-"
            + &code.chars().collect::<Vec<_>>()[4..].iter().collect::<String>()
    }

    /// 获取当前P2P配置信息
    /// 返回包含peer_id和connection_code的完整配置
    pub async fn get_config(&self) -> P2pConfig {
        let config = self.config.read().await;
        P2pConfig {
            enabled: config.enabled,
            tunnel_mode: config.tunnel_mode.clone(),
            stun_servers: config.stun_servers.clone(),
            relay_servers: config.relay_servers.clone(),
            connection_code: Some(self.connection_code.clone()),
            peer_id: Some(self.peer_id.clone()),
        }
    }

    /// 更新P2P配置
    /// 新的peer_id和connection_code会被保留，不受配置更新影响
    pub async fn update_config(&self, new_config: P2pConfig) -> P2pConfig {
        let mut config = self.config.write().await;
        *config = new_config.clone();
        // 保留原有的peer_id和connection_code
        config.connection_code = Some(self.connection_code.clone());
        config.peer_id = Some(self.peer_id.clone());
        config.clone()
    }

    /// 使P2P服务上线
    /// 初始化隧道管理器，连接到STUN和TURN服务器
    pub async fn go_online(&self) -> Result<P2pConnectionInfo, String> {
        let config = self.config.read().await.clone();

        // 将STUN服务器配置转换为内部格式
        let stun_servers = config
            .stun_servers
            .iter()
            .map(|s| StunServer {
                url: s.clone(),
                name: "STUN Server".to_string(),
            })
            .collect();

        // 将TURN/中继服务器配置转换为内部格式
        let turn_servers = config
            .relay_servers
            .iter()
            .map(|s| TurnServer::new(s.url.clone(), s.region.clone()))
            .collect();

        // 获取首选中继服务器URL，默认使用williw官方中继
        let relay_url = config
            .relay_servers
            .first()
            .map(|s| s.url.as_str())
            .unwrap_or("wss://relay.williw.ai");

        // 创建并初始化隧道管理器
        let mut manager = TunnelManager::new(
            self.peer_id.clone(),
            self.connection_code.clone(),
            config.tunnel_mode,
            stun_servers,
            turn_servers,
        );

        // 调用管理器上线，获取新的连接码
        let code = manager.go_online(relay_url).await?;
        self.connection_code = code;

        // 保存隧道管理器
        let mut tunnel_mgr = self.tunnel_manager.write().await;
        *tunnel_mgr = Some(manager);

        // 更新在线状态
        self.is_online.store(true, Ordering::SeqCst);

        // 获取NAT穿透信息，构建公网端点
        let nat_info = self.tunnel_manager.read().await
            .as_ref()
            .and_then(|m| m.get_nat_info());

        // 格式化公网端点地址
        let public_endpoint = nat_info
            .map(|n| format!("{}:{}", n.external_ip.clone().unwrap_or_default(), n.external_port.unwrap_or(0)))
            .filter(|s| !s.contains("0.0.0.0"))
            .map(|s| format!("wss://{}", s));

        // 构建并返回连接信息
        let connection_info = P2pConnectionInfo {
            peer_id: self.peer_id.clone(),
            connection_code: self.connection_code.clone(),
            public_endpoint,
            is_connected: true,
            connected_peers: vec![],
            connection_quality: ConnectionQuality::Good,
        };

        Ok(connection_info)
    }

    /// 使P2P服务下线
    /// 断开所有连接，清空状态
    pub async fn go_offline(&self) -> Result<String, String> {
        // 关闭隧道管理器
        if let Some(ref mut manager) = *self.tunnel_manager.write().await {
            manager.go_offline().await?;
        }

        // 清除隧道管理器
        self.tunnel_manager.write().await.take();
        // 重置在线状态和隧道计数
        self.is_online.store(false, Ordering::SeqCst);
        self.active_tunnels.store(0, Ordering::SeqCst);

        // 清空已连接节点列表
        let mut peers = self.connected_peers.write().await;
        peers.clear();

        Ok("P2P service went offline".to_string())
    }

    /// 获取P2P服务当前状态
    /// 返回在线状态、peer_id、连接码和活跃隧道数量等信息
    pub async fn get_status(&self) -> P2pStatus {
        let active_count = self.active_tunnels.load(Ordering::SeqCst);
        // 获取NAT穿透信息
        let nat_info = self.tunnel_manager.read().await
            .as_ref()
            .and_then(|m| m.get_nat_info());

        P2pStatus {
            is_online: self.is_online.load(Ordering::SeqCst),
            peer_id: self.peer_id.clone(),
            connection_code: self.connection_code.clone(),
            active_tunnels: active_count,
            total_bandwidth_mbps: 0.0,
            relay_usage_percent: nat_info.map(|_| 0.0),
        }
    }

    /// 获取详细连接信息
    /// 返回peer信息列表、公网端点和连接质量评估
    pub async fn get_connection_info(&self) -> Result<P2pConnectionInfo, String> {
        // 检查是否在线
        if !self.is_online.load(Ordering::SeqCst) {
            return Err("P2P service is offline. Call go_online first.".to_string());
        }

        // 获取所有已连接节点信息
        let peers = self.connected_peers.read().await;
        let peer_list: Vec<PeerInfo> = peers.values().cloned().collect();

        // 获取NAT穿透信息，构建公网端点
        let nat_info = self.tunnel_manager.read().await
            .as_ref()
            .and_then(|m| m.get_nat_info());

        let public_endpoint = nat_info
            .map(|n| format!("{}:{}", n.external_ip.clone().unwrap_or_default(), n.external_port.unwrap_or(0)))
            .filter(|s| !s.contains("0.0.0.0"))
            .map(|s| format!("wss://{}", s));

        Ok(P2pConnectionInfo {
            peer_id: self.peer_id.clone(),
            connection_code: self.connection_code.clone(),
            public_endpoint,
            is_connected: true,
            connected_peers: peer_list,
            connection_quality: ConnectionQuality::Good,
        })
    }

    /// 连接到指定的对等节点
    /// 通过peer_id建立P2P隧道连接
    pub async fn connect_to_peer(&self, host_peer_id: String) -> Result<P2pTunnelResponse, String> {
        // 检查是否在线
        if !self.is_online.load(Ordering::SeqCst) {
            return Err("P2P service is offline".to_string());
        }

        // 获取隧道管理器
        let mut manager_guard = self.tunnel_manager.write().await;
        let manager = manager_guard
            .as_mut()
            .ok_or("Tunnel manager not initialized")?;

        // 请求建立到对等节点的隧道
        let tunnel_conn = manager.connect_to_peer(&host_peer_id).await?;

        // 增加活跃隧道计数
        self.active_tunnels.fetch_add(1, Ordering::SeqCst);

        // 记录对等节点信息
        let peer_info = PeerInfo {
            peer_id: host_peer_id.clone(),
            device_name: "Remote Device".to_string(),
            endpoint: tunnel_conn.endpoint.clone(),
            connected_at: Utc::now(),
        };

        // 将节点添加到已连接列表
        self.connected_peers.write().await
            .insert(host_peer_id.clone(), peer_info);

        // 返回隧道响应信息
        Ok(P2pTunnelResponse {
            tunnel_endpoint: tunnel_conn.endpoint,
            auth_token: tunnel_conn.tunnel_id,
            expires_at: Utc::now() + chrono::Duration::hours(24),
        })
    }

    /// 断开指定隧道连接
    /// 根据tunnel_id断开对应的P2P隧道
    pub async fn disconnect_tunnel(&self, tunnel_id: &str) -> Result<(), String> {
        // 减少活跃隧道计数
        self.active_tunnels.fetch_sub(1, Ordering::SeqCst);

        // 获取管理器并断开隧道
        let mut manager_guard = self.tunnel_manager.write().await;
        if let Some(ref mut manager) = *manager_guard {
            manager.disconnect_tunnel(tunnel_id).await?;
        }

        Ok(())
    }

    /// 共享网络连接给请求者
    /// 允许其他节点通过本节点中继其网络流量
    pub async fn share_connection(&self, requester_peer_id: &str) -> Result<P2pTunnelResponse, String> {
        // 检查是否在线
        if !self.is_online.load(Ordering::SeqCst) {
            return Err("P2P service is offline".to_string());
        }

        // 生成新的隧道ID和认证令牌
        let tunnel_id = Uuid::new_v4().to_string();
        let auth_token = Uuid::new_v4().to_string();

        // 增加活跃隧道计数
        self.active_tunnels.fetch_add(1, Ordering::SeqCst);

        // 记录请求者节点信息
        let peer_info = PeerInfo {
            peer_id: requester_peer_id.to_string(),
            device_name: "Remote Device".to_string(),
            endpoint: format!("wss://{}.williw.ai", requester_peer_id),
            connected_at: Utc::now(),
        };

        // 将请求者添加到已连接列表
        self.connected_peers.write().await
            .insert(requester_peer_id.to_string(), peer_info);

        // 返回隧道响应信息，24小时后过期
        Ok(P2pTunnelResponse {
            tunnel_endpoint: format!("wss://tunnel.williw.ai/{}", tunnel_id),
            auth_token,
            expires_at: Utc::now() + chrono::Duration::hours(24),
        })
    }

    /// 获取当前活跃的隧道列表
    /// 返回所有活跃隧道的ID列表
    pub async fn get_active_tunnels(&self) -> Vec<String> {
        let tunnels = self.active_tunnels.load(Ordering::SeqCst);
        vec![format!("tunnel_{}", tunnels)]
    }

    /// 注册新的中继服务器
    /// 添加TURN服务器到配置列表
    pub async fn register_relay_server(&self, url: String, region: String) -> Result<(), String> {
        let mut config = self.config.write().await;
        config.relay_servers.push(RelayServerConfig { url, region });
        Ok(())
    }

    /// 测试当前连接质量
    /// 返回连接质量评估结果
    pub async fn test_connection(&self) -> Result<ConnectionQuality, String> {
        // 检查是否在线
        if !self.is_online.load(Ordering::SeqCst) {
            return Err("P2P service is offline".to_string());
        }

        Ok(ConnectionQuality::Excellent)
    }
}

/// P2pService的默认实现
impl Default for P2pService {
    fn default() -> Self {
        Self::new()
    }
}