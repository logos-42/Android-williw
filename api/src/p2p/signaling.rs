use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;
use williw_shared::{
    ConnectionQuality, NatDiscoveryResult, NatType, PeerEndpoint, SignalingMessage,
    SignalingMessageType, TunnelEstablished,
};
use futures_util::{SinkExt, StreamExt};

pub struct SignalingClient {
    ws_url: String,
    peer_id: String,
    connection_code: Option<String>,
    connected: Arc<RwLock<bool>>,
    sender: Arc<RwLock<Option<mpsc::Sender<SignalingMessage>>>>,
    receiver: Arc<RwLock<Option<mpsc::Receiver<SignalingMessage>>>>,
}

impl SignalingClient {
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

    pub async fn connect(&mut self) -> Result<(), String> {
        let url = Url::parse(&self.ws_url)
            .map_err(|e| format!("Invalid WebSocket URL: {}", e))?;

        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| format!("WebSocket connection failed: {}", e))?;

        let (write, read) = ws_stream.split();

        let (tx, rx) = mpsc::channel::<SignalingMessage>(100);
        let (out_tx, mut out_rx) = mpsc::channel::<SignalingMessage>(100);

        let peer_id = self.peer_id.clone();
        tokio::spawn(async move {
            let mut read = read;
            let mut out_rx = out_rx;
            let mut tx = tx;

            loop {
                tokio::select! {
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

        self.send_register().await?;

        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), String> {
        *self.connected.write().await = false;
        *self.sender.write().await = None;
        *self.receiver.write().await = None;
        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    pub async fn send_register(&mut self) -> Result<(), String> {
        let msg = SignalingMessage {
            msg_type: SignalingMessageType::Register,
            from_peer_id: self.peer_id.clone(),
            to_peer_id: None,
            payload: None,
            timestamp: chrono::Utc::now(),
        };

        self.send(msg).await
    }

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

pub struct SignalingServer {
    pub peer_id: String,
    pub connection_code: String,
    pub nat_type: NatType,
    pub external_ip: Option<String>,
    pub external_port: Option<u16>,
    pub relay_enabled: bool,
}

impl SignalingServer {
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

    pub fn with_nat_info(mut self, nat_info: NatDiscoveryResult) -> Self {
        self.nat_type = nat_info.nat_type;
        self.external_ip = nat_info.external_ip;
        self.external_port = nat_info.external_port;
        self.relay_enabled = self.nat_type == NatType::Symmetric
            || self.nat_type == NatType::PortRestricted
            || self.nat_type == NatType::Restricted;
        self
    }

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
