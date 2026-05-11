use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use williw_shared::{
    ConnectionQuality, NatDiscoveryResult, NatType, PeerEndpoint, P2pTunnelMode, RelayServerConfig,
    StunServer, TurnServer,
};

use super::signaling::{SignalingClient, SignalingServer};
use super::stun_client::StunClient;
use super::turn_client::TurnClient;

pub struct TunnelManager {
    peer_id: String,
    connection_code: String,
    mode: P2pTunnelMode,
    stun_servers: Vec<StunServer>,
    turn_servers: Vec<TurnServer>,
    nat_info: Option<NatDiscoveryResult>,
    signaling_client: Option<SignalingClient>,
    active_tunnels: Arc<RwLock<HashMap<String, Tunnel>>>,
    is_online: Arc<RwLock<bool>>,
}

struct Tunnel {
    tunnel_id: String,
    peer_id: String,
    local_endpoint: SocketAddr,
    remote_endpoint: SocketAddr,
    relay_used: bool,
    established_at: std::time::Instant,
    last_activity: std::time::Instant,
}

impl TunnelManager {
    pub fn new(
        peer_id: String,
        connection_code: String,
        mode: P2pTunnelMode,
        stun_servers: Vec<StunServer>,
        turn_servers: Vec<TurnServer>,
    ) -> Self {
        Self {
            peer_id,
            connection_code,
            mode,
            stun_servers,
            turn_servers,
            nat_info: None,
            signaling_client: None,
            active_tunnels: Arc::new(RwLock::new(HashMap::new())),
            is_online: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn go_online(&mut self, relay_url: &str) -> Result<String, String> {
        self.nat_info = Some(self.discover_nat().await?);

        let mut signaling = SignalingClient::new(relay_url, self.peer_id.clone());
        signaling.connect().await?;

        signaling.register_connection_code(&self.connection_code).await?;

        if let Some(ref nat_info) = self.nat_info {
            signaling.send_nat_info(nat_info).await?;
        }

        self.signaling_client = Some(signaling);
        *self.is_online.write().await = true;

        Ok(self.connection_code.clone())
    }

    pub async fn go_offline(&mut self) -> Result<(), String> {
        if let Some(ref mut signaling) = self.signaling_client {
            signaling.disconnect().await?;
        }
        self.signaling_client = None;
        *self.is_online.write().await = false;

        let mut tunnels = self.active_tunnels.write().await;
        tunnels.clear();

        Ok(())
    }

    pub async fn is_online(&self) -> bool {
        *self.is_online.read().await
    }

    async fn discover_nat(&self) -> Result<NatDiscoveryResult, String> {
        if let Some(ref stun_server) = self.stun_servers.first() {
            let client = StunClient::new(0)?;
            client.discover(stun_server)
        } else {
            Err("No STUN servers configured".to_string())
        }
    }

    pub async fn connect_to_peer(&mut self, host_peer_id: &str) -> Result<TunnelConnection, String> {
        if !self.is_online().await {
            return Err("P2P service is offline".to_string());
        }

        let signaling = self.signaling_client.as_mut()
            .ok_or("Signaling client not initialized")?;

        let nat_info = self.nat_info.as_ref()
            .ok_or("NAT info not discovered")?;

        match nat_info.nat_type {
            NatType::Symmetric => {
                self.connect_via_relay(host_peer_id).await
            }
            NatType::PortRestricted | NatType::Restricted => {
                self.try_nat_traversal(host_peer_id).await
            }
            NatType::FullCone => {
                self.try_direct_connection(host_peer_id).await
            }
            NatType::OpenInternet => {
                self.try_direct_connection(host_peer_id).await
            }
            NatType::Unknown => {
                self.connect_via_relay(host_peer_id).await
            }
        }
    }

    async fn try_direct_connection(&mut self, host_peer_id: &str) -> Result<TunnelConnection, String> {
        let signaling = self.signaling_client.as_mut()
            .ok_or("Signaling client not initialized")?;

        match signaling.request_connection(host_peer_id).await {
            Ok(tunnel_info) => {
                let tunnel = Tunnel {
                    tunnel_id: tunnel_info.tunnel_id.clone(),
                    peer_id: host_peer_id.to_string(),
                    local_endpoint: "0.0.0.0:0".parse().unwrap(),
                    remote_endpoint: tunnel_info.remote_endpoint.parse().unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap()),
                    relay_used: tunnel_info.relay_used,
                    established_at: std::time::Instant::now(),
                    last_activity: std::time::Instant::now(),
                };

                let mut tunnels = self.active_tunnels.write().await;
                tunnels.insert(tunnel.tunnel_id.clone(), tunnel);

                Ok(TunnelConnection {
                    tunnel_id: tunnel_info.tunnel_id,
                    endpoint: tunnel_info.remote_endpoint,
                    relay_used: tunnel_info.relay_used,
                    quality: tunnel_info.connection_quality,
                })
            }
            Err(e) => {
                tracing::warn!("Direct connection failed, falling back to relay: {}", e);
                self.connect_via_relay(host_peer_id).await
            }
        }
    }

    async fn try_nat_traversal(&mut self, host_peer_id: &str) -> Result<TunnelConnection, String> {
        let signaling = self.signaling_client.as_mut()
            .ok_or("Signaling client not initialized")?;

        match signaling.request_connection(host_peer_id).await {
            Ok(tunnel_info) => {
                let tunnel = Tunnel {
                    tunnel_id: tunnel_info.tunnel_id.clone(),
                    peer_id: host_peer_id.to_string(),
                    local_endpoint: "0.0.0.0:0".parse().unwrap(),
                    remote_endpoint: tunnel_info.remote_endpoint.parse().unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap()),
                    relay_used: tunnel_info.relay_used,
                    established_at: std::time::Instant::now(),
                    last_activity: std::time::Instant::now(),
                };

                let mut tunnels = self.active_tunnels.write().await;
                tunnels.insert(tunnel.tunnel_id.clone(), tunnel);

                Ok(TunnelConnection {
                    tunnel_id: tunnel_info.tunnel_id,
                    endpoint: tunnel_info.remote_endpoint,
                    relay_used: tunnel_info.relay_used,
                    quality: tunnel_info.connection_quality,
                })
            }
            Err(e) => {
                tracing::warn!("NAT traversal failed, falling back to relay: {}", e);
                self.connect_via_relay(host_peer_id).await
            }
        }
    }

    async fn connect_via_relay(&mut self, host_peer_id: &str) -> Result<TunnelConnection, String> {
        if let Some(ref turn_server) = self.turn_servers.first() {
            let mut turn_client = TurnClient::new(0, turn_server)?;
            let relay_addr = turn_client.allocate()?;

            let signaling = self.signaling_client.as_mut()
                .ok_or("Signaling client not initialized")?;

            turn_client.create_permission("0.0.0.0")?;

            let tunnel_id = Uuid::new_v4().to_string();

            let tunnel = Tunnel {
                tunnel_id: tunnel_id.clone(),
                peer_id: host_peer_id.to_string(),
                local_endpoint: turn_client.local_addr,
                remote_endpoint: relay_addr,
                relay_used: true,
                established_at: std::time::Instant::now(),
                last_activity: std::time::Instant::now(),
            };

            let mut tunnels = self.active_tunnels.write().await;
            tunnels.insert(tunnel_id.clone(), tunnel);

            Ok(TunnelConnection {
                tunnel_id,
                endpoint: relay_addr.to_string(),
                relay_used: true,
                quality: ConnectionQuality::Good,
            })
        } else {
            Err("No TURN servers configured".to_string())
        }
    }

    pub async fn disconnect_tunnel(&mut self, tunnel_id: &str) -> Result<(), String> {
        let mut tunnels = self.active_tunnels.write().await;
        tunnels.remove(tunnel_id);
        Ok(())
    }

    pub async fn get_active_tunnels(&self) -> Vec<String> {
        let tunnels = self.active_tunnels.read().await;
        tunnels.keys().cloned().collect()
    }

    pub async fn get_nat_info(&self) -> Option<&NatDiscoveryResult> {
        self.nat_info.as_ref()
    }

    pub async fn keepalive(&mut self) -> Result<(), String> {
        if let Some(ref mut signaling) = self.signaling_client {
            signaling.send_keepalive().await?;
        }

        let mut tunnels = self.active_tunnels.write().await;
        for tunnel in tunnels.values_mut() {
            tunnel.last_activity = std::time::Instant::now();
        }

        Ok(())
    }

    pub async fn cleanup_stale_tunnels(&mut self, timeout: Duration) {
        let mut tunnels = self.active_tunnels.write().await;
        tunnels.retain(|_, tunnel| {
            tunnel.last_activity.elapsed() < timeout
        });
    }
}

pub struct TunnelConnection {
    pub tunnel_id: String,
    pub endpoint: String,
    pub relay_used: bool,
    pub quality: ConnectionQuality,
}

pub struct TunnelEndpoint {
    pub tunnel_id: String,
    pub peer_id: String,
    pub local_socket: SocketAddr,
    pub remote_socket: SocketAddr,
    pub relay_used: bool,
}

impl TunnelEndpoint {
    pub fn new(tunnel_id: String, peer_id: String) -> Self {
        Self {
            tunnel_id,
            peer_id,
            local_socket: SocketAddr::from(([0, 0, 0, 0], 0)),
            remote_socket: SocketAddr::from(([0, 0, 0, 0], 0)),
            relay_used: false,
        }
    }

    pub fn with_sockets(mut self, local: SocketAddr, remote: SocketAddr) -> Self {
        self.local_socket = local;
        self.remote_socket = remote;
        self
    }

    pub fn with_relay(mut self, used: bool) -> Self {
        self.relay_used = used;
        self
    }
}
