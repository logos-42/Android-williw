use williw_shared::*;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct P2pService {
    config: RwLock<P2pConfig>,
    is_online: AtomicBool,
    peer_id: String,
    connection_code: String,
    active_tunnels: AtomicU32,
    connected_peers: RwLock<HashMap<String, PeerInfo>>,
}

impl P2pService {
    pub fn new() -> Self {
        let peer_id = format!("peer_{}", Uuid::new_v4().to_string().replace("-", "")[..16].to_string());
        let connection_code = Self::generate_connection_code();

        Self {
            config: RwLock::new(P2pConfig::default()),
            is_online: AtomicBool::new(false),
            peer_id,
            connection_code,
            active_tunnels: AtomicU32::new(0),
            connected_peers: RwLock::new(HashMap::new()),
        }
    }

    fn generate_connection_code() -> String {
        let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
        let mut code = String::new();
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let mut rng = seed;
        for _ in 0..8 {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            code.push(chars[(rng as usize) % chars.len()]);
        }
        code.chars().collect::<Vec<_>>()[..4].iter().collect::<String>()
            + "-" +
            &code.chars().collect::<Vec<_>>()[4..].iter().collect::<String>()
    }

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

    pub async fn update_config(&self, new_config: P2pConfig) -> P2pConfig {
        let mut config = self.config.write().await;
        *config = new_config.clone();
        config.connection_code = Some(self.connection_code.clone());
        config.peer_id = Some(self.peer_id.clone());
        config.clone()
    }

    pub async fn go_online(&self) -> Result<P2pConnectionInfo, String> {
        self.is_online.store(true, Ordering::SeqCst);

        let connection_info = P2pConnectionInfo {
            peer_id: self.peer_id.clone(),
            connection_code: self.connection_code.clone(),
            public_endpoint: Some(format!("wss://{}.williw.ai", self.peer_id)),
            is_connected: true,
            connected_peers: vec![],
            connection_quality: ConnectionQuality::Excellent,
        };

        Ok(connection_info)
    }

    pub async fn go_offline(&self) -> Result<String, String> {
        self.is_online.store(false, Ordering::SeqCst);
        self.active_tunnels.store(0, Ordering::SeqCst);

        let mut peers = self.connected_peers.write().await;
        peers.clear();

        Ok("P2P service went offline".to_string())
    }

    pub async fn get_status(&self) -> P2pStatus {
        P2pStatus {
            is_online: self.is_online.load(Ordering::SeqCst),
            peer_id: self.peer_id.clone(),
            connection_code: self.connection_code.clone(),
            active_tunnels: self.active_tunnels.load(Ordering::SeqCst),
            total_bandwidth_mbps: 0.0,
            relay_usage_percent: None,
        }
    }

    pub async fn get_connection_info(&self) -> Result<P2pConnectionInfo, String> {
        if !self.is_online.load(Ordering::SeqCst) {
            return Err("P2P service is offline. Call go_online first.".to_string());
        }

        let peers = self.connected_peers.read().await;
        let peer_list: Vec<PeerInfo> = peers.values().cloned().collect();

        Ok(P2pConnectionInfo {
            peer_id: self.peer_id.clone(),
            connection_code: self.connection_code.clone(),
            public_endpoint: Some(format!("wss://{}.williw.ai", self.peer_id)),
            is_connected: true,
            connected_peers: peer_list,
            connection_quality: ConnectionQuality::Good,
        })
    }

    pub async fn connect_to_peer(&self, host_peer_id: String) -> Result<P2pTunnelResponse, String> {
        if !self.is_online.load(Ordering::SeqCst) {
            return Err("P2P service is offline".to_string());
        }

        let tunnel_id = Uuid::new_v4().to_string();
        let auth_token = Uuid::new_v4().to_string();

        self.active_tunnels.fetch_add(1, Ordering::SeqCst);

        Ok(P2pTunnelResponse {
            tunnel_endpoint: format!("wss://tunnel.williw.ai/{}", tunnel_id),
            auth_token,
            expires_at: Utc::now() + chrono::Duration::hours(24),
        })
    }

    pub async fn disconnect_tunnel(&self, tunnel_id: &str) -> Result<(), String> {
        self.active_tunnels.fetch_sub(1, Ordering::SeqCst);
        Ok(())
    }

    pub async fn share_connection(&self, requester_peer_id: &str) -> Result<P2pTunnelResponse, String> {
        if !self.is_online.load(Ordering::SeqCst) {
            return Err("P2P service is offline".to_string());
        }

        let tunnel_id = Uuid::new_v4().to_string();
        let auth_token = Uuid::new_v4().to_string();

        self.active_tunnels.fetch_add(1, Ordering::SeqCst);

        let peer_info = PeerInfo {
            peer_id: requester_peer_id.to_string(),
            device_name: "Remote Device".to_string(),
            endpoint: format!("wss://{}.williw.ai", requester_peer_id),
            connected_at: Utc::now(),
        };

        let mut peers = self.connected_peers.write().await;
        peers.insert(requester_peer_id.to_string(), peer_info);

        Ok(P2pTunnelResponse {
            tunnel_endpoint: format!("wss://tunnel.williw.ai/{}", tunnel_id),
            auth_token,
            expires_at: Utc::now() + chrono::Duration::hours(24),
        })
    }

    pub async fn get_active_tunnels(&self) -> Vec<String> {
        let tunnels = self.active_tunnels.load(Ordering::SeqCst);
        vec![format!("tunnel_{}", tunnels)]
    }

    pub async fn register_relay_server(&self, url: String, region: String) -> Result<(), String> {
        let mut config = self.config.write().await;
        config.relay_servers.push(RelayServerConfig { url, region });
        Ok(())
    }

    pub async fn test_connection(&self) -> Result<ConnectionQuality, String> {
        if !self.is_online.load(Ordering::SeqCst) {
            return Err("P2P service is offline".to_string());
        }

        Ok(ConnectionQuality::Excellent)
    }
}

impl Default for P2pService {
    fn default() -> Self {
        Self::new()
    }
}
