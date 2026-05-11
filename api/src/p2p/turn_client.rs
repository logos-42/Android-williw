use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use williw_shared::TurnServer;

const TURN_HEADER_SIZE: usize = 20;
const CHANNEL_DATA: u16 = 0x0004;
const ALLOCATE_REQUEST: u16 = 0x0003;
const ALLOCATE_RESPONSE: u16 = 0x0103;
const CREATE_PERMISSION_REQUEST: u16 = 0x0008;
const CHANNEL_BIND_REQUEST: u16 = 0x0009;

pub struct TurnClient {
    socket: UdpSocket,
    local_addr: SocketAddr,
    server_addr: SocketAddr,
    username: Option<String>,
    password: Option<String>,
    relayed_addr: Option<SocketAddr>,
    timeout: Duration,
}

impl TurnClient {
    pub fn new(local_port: u16, server: &TurnServer) -> Result<Self, String> {
        let server_addr = parse_turn_url(&server.url)
            .ok_or_else(|| format!("Invalid TURN server URL: {}", server.url))?;

        let local_addr = SocketAddr::from(([0, 0, 0, 0], local_port));
        let socket = UdpSocket::bind(local_addr)
            .map_err(|e| format!("Failed to bind socket: {}", e))?;

        socket
            .set_nonblocking(false)
            .map_err(|e| format!("Failed to set non-blocking: {}", e))?;

        Ok(Self {
            socket,
            local_addr: socket.local_addr().map_err(|e| e.to_string())?,
            server_addr,
            username: server.username.clone(),
            password: server.password.clone(),
            relayed_addr: None,
            timeout: Duration::from_secs(5),
        })
    }

    pub fn allocate(&mut self) -> Result<SocketAddr, String> {
        self.send_allocate_request()?;

        let mut response_buf = [0u8; 1024];
        self.socket
            .set_read_timeout(Some(self.timeout))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        let (bytes_read, _) = self
            .socket
            .recv_from(&mut response_buf)
            .map_err(|e| format!("Failed to receive allocation response: {}", e))?;

        let relayed_addr = self.parse_allocation_response(&response_buf[..bytes_read])?;
        self.relayed_addr = Some(relayed_addr);

        Ok(relayed_addr)
    }

    fn send_allocate_request(&self) -> Result<(), String> {
        let mut request = vec![0u8; TURN_HEADER_SIZE + 24];

        request[0..2].copy_from_slice(&ALLOCATE_REQUEST.to_be_bytes());
        let msg_len: u16 = 24;
        request[2..4].copy_from_slice(&msg_len.to_be_bytes());

        let transaction_id = generate_transaction_id();
        request[4..8].copy_from_slice(&MAGIC_COOKIE.to_be_bytes());
        request[8..20].copy_from_slice(&transaction_id);

        let mut offset = TURN_HEADER_SIZE;

        if let (Some(username), Some(_)) = (&self.username, &self.password) {
            let user_bytes = username.as_bytes();
            request[offset..offset + 4].copy_from_slice(&SOFTWARE_ATTRIBUTE);
            request[offset + 4] = 0;
            request[offset + 5] = ((user_bytes.len() + 4) as u8).min(128) as u8;
            offset += 4;
            let len = (user_bytes.len() as u8).min(128) as usize;
            request[offset..offset + len].copy_from_slice(&user_bytes[..len]);
            offset += user_bytes.len();
            while offset % 4 != 0 {
                offset += 1;
            }
        }

        let total_len = offset - TURN_HEADER_SIZE;
        request[2..4].copy_from_slice(&(total_len as u16).to_be_bytes());

        self.socket
            .send_to(&request, self.server_addr)
            .map_err(|e| format!("Failed to send TURN allocate: {}", e))?;

        Ok(())
    }

    fn parse_allocation_response(&self, data: &[u8]) -> Result<SocketAddr, String> {
        if data.len() < TURN_HEADER_SIZE {
            return Err("Response too short".to_string());
        }

        let (msg_type, msg_length, _) = parse_header(data);

        if msg_type != ALLOCATE_RESPONSE {
            return Err(format!("Expected ALLOCATE_RESPONSE, got 0x{:04x}", msg_type));
        }

        let mut offset = TURN_HEADER_SIZE;
        let end = TURN_HEADER_SIZE + msg_length as usize;

        while offset + 4 < end {
            let attr_type = u16::from_be_bytes([data[offset], data[offset + 1]]);
            let attr_len = u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize;
            offset += 4;

            if attr_type == XOR_RELAYED_ADDRESS && attr_len >= 8 {
                let family = data[offset];
                let xport = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
                let xored_port = xport ^ ((MAGIC_COOKIE >> 16) as u16);

                let mut xaddr = [0u8; 4];
                xaddr.copy_from_slice(&data[offset + 4..offset + 8]);
                let xored_ip = u32::from_be_bytes(xaddr) ^ MAGIC_COOKIE;

                let ip = format!(
                    "{}.{}.{}.{}",
                    (xored_ip >> 24) as u8,
                    (xored_ip >> 16) as u8,
                    (xored_ip >> 8) as u8,
                    xored_ip as u8
                );

                if family == 0x01 {
                    return Ok(SocketAddr::new(ip.parse().unwrap(), xored_port));
                }
            }

            offset += attr_len as usize;
            if attr_len % 4 != 0 {
                offset += 4 - (attr_len % 4);
            }
        }

        Err("XOR_RELAYED_ADDRESS not found".to_string())
    }

    pub fn create_permission(&self, peer_ip: &str) -> Result<(), String> {
        let peer_addr: SocketAddr = format!("{}:0", peer_ip)
            .parse()
            .map_err(|e| format!("Invalid peer IP: {}", e))?;

        let mut request = vec![0u8; TURN_HEADER_SIZE + 16];
        request[0..2].copy_from_slice(&CREATE_PERMISSION_REQUEST.to_be_bytes());
        request[2..4].copy_from_slice(&16u16.to_be_bytes());

        let transaction_id = generate_transaction_id();
        request[4..8].copy_from_slice(&MAGIC_COOKIE.to_be_bytes());
        request[8..20].copy_from_slice(&transaction_id);

        request[TURN_HEADER_SIZE..TURN_HEADER_SIZE + 4].copy_from_slice(&XOR_PEER_ADDRESS_ATTRIBUTE);
        request[TURN_HEADER_SIZE + 4..TURN_HEADER_SIZE + 8].copy_from_slice(&8u16.to_be_bytes());
        request[TURN_HEADER_SIZE + 8] = 0x01;

        let peer_port = 0u16 ^ ((MAGIC_COOKIE >> 16) as u16);
        request[TURN_HEADER_SIZE + 8..TURN_HEADER_SIZE + 10].copy_from_slice(&peer_port.to_be_bytes());

        let peer_u32: u32 = peer_addr.ip().to_string().parse().unwrap_or(0);
        let xored_ip = peer_u32 ^ MAGIC_COOKIE;
        request[TURN_HEADER_SIZE + 10..TURN_HEADER_SIZE + 14].copy_from_slice(&xored_ip.to_be_bytes());

        self.socket
            .send_to(&request, self.server_addr)
            .map_err(|e| format!("Failed to send permission: {}", e))?;

        Ok(())
    }

    pub fn channel_bind(&self, peer_addr: SocketAddr, channel_id: u16) -> Result<(), String> {
        let mut request = vec![0u8; TURN_HEADER_SIZE + 16];
        request[0..2].copy_from_slice(&CHANNEL_BIND_REQUEST.to_be_bytes());
        request[2..4].copy_from_slice(&16u16.to_be_bytes());

        let transaction_id = generate_transaction_id();
        request[4..8].copy_from_slice(&MAGIC_COOKIE.to_be_bytes());
        request[8..20].copy_from_slice(&transaction_id);

        request[TURN_HEADER_SIZE..TURN_HEADER_SIZE + 2].copy_from_slice(&channel_id.to_be_bytes());
        request[TURN_HEADER_SIZE + 2..TURN_HEADER_SIZE + 4].copy_from_slice(&12u16.to_be_bytes());
        request[TURN_HEADER_SIZE + 4] = 0x01;

        let peer_port = peer_addr.port() ^ ((MAGIC_COOKIE >> 16) as u16);
        request[TURN_HEADER_SIZE + 4..TURN_HEADER_SIZE + 6].copy_from_slice(&peer_port.to_be_bytes());

        let peer_u32: u32 = peer_addr.ip().to_string().parse().unwrap_or(0);
        let xored_ip = peer_u32 ^ MAGIC_COOKIE;
        request[TURN_HEADER_SIZE + 6..TURN_HEADER_SIZE + 10].copy_from_slice(&xored_ip.to_be_bytes());

        self.socket
            .send_to(&request, self.server_addr)
            .map_err(|e| format!("Failed to send channel bind: {}", e))?;

        Ok(())
    }

    pub fn send_to_relay(&self, data: &[u8], peer_addr: SocketAddr) -> Result<usize, String> {
        self.socket
            .send_to(data, self.server_addr)
            .map_err(|e| format!("Failed to send to relay: {}", e))
    }

    pub fn recv_from_relay(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), String> {
        self.socket
            .recv_from(buf)
            .map_err(|e| format!("Failed to recv from relay: {}", e))
    }

    pub fn relayed_address(&self) -> Option<SocketAddr> {
        self.relayed_addr
    }
}

const MAGIC_COOKIE: u32 = 0x2112A442;
const XOR_RELAYED_ADDRESS: u16 = 0x0016;
const XOR_PEER_ADDRESS_ATTRIBUTE: u16 = 0x0020;
const SOFTWARE_ATTRIBUTE: u16 = 0x8022;

fn parse_turn_url(url: &str) -> Option<SocketAddr> {
    let url = url.strip_prefix("turn:")?;
    let url = url.strip_prefix("turn://")?;

    let parts: Vec<&str> = url.rsplitn(2, ':').collect();
    if parts.len() == 2 {
        let port: u16 = parts[0].parse().ok()?;
        let host = parts[1];
        Some(SocketAddr::new(host.parse().ok()?, port))
    } else {
        let parts: Vec<&str> = url.split(':').collect();
        if parts.len() == 2 {
            let port: u16 = parts[1].parse().ok()?;
            Some(SocketAddr::new(parts[0].parse().ok()?, port))
        } else {
            Some(SocketAddr::new(url.parse().ok()?, 3478))
        }
    }
}

fn generate_transaction_id() -> [u8; 12] {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut tid = [0u8; 12];
    tid[0..8].copy_from_slice(&now.to_be_bytes()[0..8]);
    tid[8..12].copy_from_slice(&[
        (now as u32 >> 24) as u8,
        (now as u32 >> 16) as u8,
        (now as u32 >> 8) as u8,
        now as u8,
    ]);
    tid
}

fn parse_header(data: &[u8]) -> (u16, u16, [u8; 12]) {
    let msg_type = u16::from_be_bytes([data[0], data[1]]);
    let msg_length = u16::from_be_bytes([data[2], data[3]]);
    let mut transaction_id = [0u8; 12];
    transaction_id.copy_from_slice(&data[4..16]);
    (msg_type, msg_length, transaction_id)
}

pub fn create_turn_client(server: &TurnServer) -> Result<TurnClient, String> {
    TurnClient::new(0, server)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_turn_url() {
        let addr = parse_turn_url("turn:turn.williw.ai:3478");
        assert!(addr.is_some());

        let addr2 = parse_turn_url("turn://turn.williw.ai:3478");
        assert!(addr2.is_some());
    }
}
