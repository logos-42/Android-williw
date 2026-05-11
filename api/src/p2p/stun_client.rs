use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::time::Duration;
use williw_shared::{NatDiscoveryResult, NatType, StunServer};

/// STUN协议头部大小（字节）
const STUN_HEADER_SIZE: usize = 20;
/// STUN魔术cookie值，用于协议识别
const STUN_MAGIC_COOKIE: u32 = 0x2112A442;
/// STUN绑定请求消息类型
const BINDING_REQUEST: u16 = 0x0001;
/// STUN绑定响应消息类型
const BINDING_RESPONSE: u16 = 0x0101;
/// MAPPED_ADDRESS属性类型
const MAPPED_ADDRESS: u16 = 0x0001;
/// XOR_MAPPED_ADDRESS属性类型
const XOR_MAPPED_ADDRESS: u16 = 0x0020;
/// CHANGE_REQUEST属性类型
const CHANGE_REQUEST: u16 = 0x0003;

/// STUN客户端
/// 用于发现NAT穿透信息，获取公网IP和端口
pub struct StunClient {
    /// UDP套接字
    socket: UdpSocket,
    /// 本地IP地址
    local_ip: IpAddr,
    /// 本地端口
    local_port: u16,
    /// 请求超时时间
    timeout: Duration,
}

impl StunClient {
    /// 创建STUN客户端
    /// 在指定端口绑定UDP套接字
    /// 
    /// # 参数
    /// * `port` - 本地监听端口，0表示随机端口
    pub fn new(port: u16) -> Result<Self, String> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let socket = UdpSocket::bind(addr).map_err(|e| format!("Failed to bind socket: {}", e))?;
        socket.set_nonblocking(false).map_err(|e| format!("Failed to set non-blocking: {}", e))?;

        let local_ip = local_ip().ok_or("Failed to detect local IP")?;
        let local_port = socket.local_addr().map_err(|e| format!("Failed to get local addr: {}", e))?.port();

        Ok(Self {
            socket,
            local_ip,
            local_port,
            timeout: Duration::from_secs(3),
        })
    }

    /// 执行NAT发现
    /// 向STUN服务器发送绑定请求，获取公网端点并检测NAT类型
    /// 
    /// # 参数
    /// * `stun_server` - STUN服务器信息
    pub fn discover(&self, stun_server: &StunServer) -> Result<NatDiscoveryResult, String> {
        let server_addr = parse_stun_url(&stun_server.url)
            .ok_or_else(|| format!("Invalid STUN server URL: {}", stun_server.url))?;

        // 获取公网IP和端口
        let (external_ip, external_port) = self.send_binding_request(server_addr)?;

        // 检测NAT类型
        let nat_type = self.detect_nat_type(server_addr);

        Ok(NatDiscoveryResult {
            external_ip: Some(external_ip),
            external_port: Some(external_port),
            nat_type,
            local_ip: self.local_ip.to_string(),
            local_port: self.local_port,
            stun_server_used: stun_server.url.clone(),
        })
    }

    /// 发送STUN绑定请求并解析响应
    fn send_binding_request(&self, server_addr: SocketAddr) -> Result<(String, u16), String> {
        let transaction_id = generate_transaction_id();
        let request = build_binding_request(&transaction_id);

        // 发送请求
        self.socket
            .send_to(&request, server_addr)
            .map_err(|e| format!("Failed to send STUN request: {}", e))?;

        // 接收响应
        let mut response_buf = [0u8; 512];
        let (bytes_read, _) = self
            .socket
            .recv_from(&mut response_buf)
            .map_err(|e| format!("Failed to receive STUN response: {}", e))?;

        if bytes_read < STUN_HEADER_SIZE {
            return Err("STUN response too short".to_string());
        }

        let response = &response_buf[..bytes_read];
        let (msg_type, msg_length, _) = parse_header(response);

        if msg_type != BINDING_RESPONSE {
            return Err(format!("Expected BINDING_RESPONSE (0x{:04x}), got 0x{:04x}", BINDING_RESPONSE, msg_type));
        }

        parse_mapped_address(response, msg_length)
    }

    /// 检测NAT类型
    /// 通过改变IP地址和端口测试响应来推断NAT类型
    fn detect_nat_type(&self, server_addr: SocketAddr) -> NatType {
        let change_ip = "1.0.0.1".parse().ok();
        let change_port: Option<u16> = Some(1);

        // 测试1：改变IP和端口
        let result1 = self.test_change_request(server_addr, change_ip, change_port);

        match result1 {
            // 可以收到响应说明是开放网络
            Ok(_) => NatType::OpenInternet,
            Err(_) => {
                // 测试2：只改变端口
                let result2 = self.test_change_request(server_addr, None, change_port);
                match result2 {
                    Ok(_) => NatType::FullCone,
                    Err(_) => {
                        // 测试3：不改变地址
                        let result3 = self.test_without_change(server_addr);
                        match result3 {
                            Ok(_) => NatType::Restricted,
                            Err(_) => NatType::PortRestricted,
                        }
                    }
                }
            }
        }
    }

    /// 测试改变请求（用于NAT类型检测）
    fn test_change_request(
        &self,
        server_addr: SocketAddr,
        change_ip: Option<IpAddr>,
        change_port: Option<u16>,
    ) -> Result<(), String> {
        let transaction_id = generate_transaction_id();
        let mut request = build_binding_request(&transaction_id);

        // 构建CHANGE_REQUEST属性
        let mut change_value: u32 = 0;
        if change_ip.is_some() {
            change_value |= 0x04;
        }
        if change_port.is_some() {
            change_value |= 0x02;
        }

        let change_attr = build_attribute(CHANGE_REQUEST, &change_value.to_be_bytes());

        request.extend_from_slice(&change_attr);

        // 更新消息长度
        let len = (request.len() - STUN_HEADER_SIZE) as u16;
        request[2] = (len >> 8) as u8;
        request[3] = (len & 0xFF) as u8;

        self.socket
            .set_read_timeout(Some(self.timeout))
            .ok();
        self.socket
            .send_to(&request, server_addr)
            .map_err(|e| format!("Send failed: {}", e))?;

        let mut response_buf = [0u8; 512];
        match self.socket.recv_from(&mut response_buf) {
            Ok((_, _)) => Ok(()),
            Err(_) => Err("No response".to_string()),
        }
    }

    /// 不改变地址的测试
    fn test_without_change(&self, server_addr: SocketAddr) -> Result<(), String> {
        let transaction_id = generate_transaction_id();
        let request = build_binding_request(&transaction_id);

        self.socket
            .set_read_timeout(Some(self.timeout))
            .ok();

        self.socket
            .send_to(&request, server_addr)
            .map_err(|e| format!("Send failed: {}", e))?;

        let mut response_buf = [0u8; 512];
        match self.socket.recv_from(&mut response_buf) {
            Ok((_, _)) => Ok(()),
            Err(_) => Err("No response".to_string()),
        }
    }
}

/// 获取本机IP地址
fn local_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip())
}

/// 解析STUN服务器URL
/// 支持 "stun:host:port" 或 "host:port" 格式
fn parse_stun_url(url: &str) -> Option<SocketAddr> {
    let url = url.strip_prefix("stun:")?;
    let parts: Vec<&str> = url.rsplitn(2, ':').collect();
    if parts.len() == 2 {
        let port: u16 = parts[0].parse().ok()?;
        let host = parts[1];
        Some(SocketAddr::new(host.parse().ok()?, port))
    } else {
        Some(SocketAddr::new(url.parse().ok()?, 19302))
    }
}

/// 生成12字节的事务ID
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

/// 构建STUN绑定请求消息
fn build_binding_request(transaction_id: &[u8; 12]) -> Vec<u8> {
    let mut msg = vec![0u8; STUN_HEADER_SIZE];
    msg[0..2].copy_from_slice(&BINDING_REQUEST.to_be_bytes());
    msg[4..8].copy_from_slice(&STUN_MAGIC_COOKIE.to_be_bytes());
    msg[8..20].copy_from_slice(transaction_id);
    msg
}

/// 构建STUN属性
fn build_attribute(attr_type: u16, value: &[u8]) -> Vec<u8> {
    let mut attr = vec![0u8; 4 + value.len()];
    attr[0..2].copy_from_slice(&attr_type.to_be_bytes());
    attr[2..4].copy_from_slice(&((value.len() as u16).to_be_bytes()));
    attr[4..].copy_from_slice(value);
    attr
}

/// 解析STUN消息头部
fn parse_header(data: &[u8]) -> (u16, u16, [u8; 12]) {
    let msg_type = u16::from_be_bytes([data[0], data[1]]);
    let msg_length = u16::from_be_bytes([data[2], data[3]]);
    let mut transaction_id = [0u8; 12];
    transaction_id.copy_from_slice(&data[4..16]);
    (msg_type, msg_length, transaction_id)
}

/// 从STUN响应中解析映射地址
fn parse_mapped_address(data: &[u8], msg_length: u16) -> Result<(String, u16), String> {
    let mut offset = STUN_HEADER_SIZE;
    let end = STUN_HEADER_SIZE + msg_length as usize;

    while offset + 4 < end {
        let attr_type = u16::from_be_bytes([data[offset], data[offset + 1]]);
        let attr_len = u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize;
        offset += 4;

        // 尝试解析XOR映射地址（优先）
        if attr_type == XOR_MAPPED_ADDRESS && attr_len >= 8 {
            let family = data[offset];
            let xport = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
            // 与魔术cookie高16位异或
            let xored_port = xport ^ ((STUN_MAGIC_COOKIE >> 16) as u16);

            let mut xaddr = [0u8; 4];
            xaddr.copy_from_slice(&data[offset + 4..offset + 8]);
            let xored_ip = u32::from_be_bytes(xaddr) ^ STUN_MAGIC_COOKIE;

            let ip = IpAddr::from([
                (xored_ip >> 24) as u8,
                (xored_ip >> 16) as u8,
                (xored_ip >> 8) as u8,
                xored_ip as u8,
            ]);

            if family == 0x01 {
                return Ok((ip.to_string(), xored_port));
            }
        } else if attr_type == MAPPED_ADDRESS && attr_len >= 8 {
            // 普通映射地址
            let family = data[offset];
            let port = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);

            let ip = IpAddr::from([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);

            if family == 0x01 {
                return Ok((ip.to_string(), port));
            }
        }

        offset += attr_len;
        // 属性需要4字节对齐
        if attr_len % 4 != 0 {
            offset += 4 - (attr_len % 4);
        }
    }

    Err("MAPPED_ADDRESS not found in response".to_string())
}

/// 同步STUN测试
/// 在当前线程执行NAT发现
pub fn stun_test(server_url: &str) -> Result<NatDiscoveryResult, String> {
    let client = StunClient::new(0)?;
    let server = StunServer {
        url: server_url.to_string(),
        name: "Test STUN".to_string(),
    };
    client.discover(&server)
}

/// 异步STUN测试
/// 在后台线程执行NAT发现
pub async fn stun_test_async(server_url: &str) -> Result<NatDiscoveryResult, String> {
    tokio::task::spawn_blocking(move || stun_test(server_url))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stun_url() {
        let addr = parse_stun_url("stun:stun.l.google.com:19302");
        assert!(addr.is_some());

        let addr2 = parse_stun_url("stun.l.google.com:19302");
        assert!(addr2.is_some());
    }

    #[test]
    fn test_generate_transaction_id() {
        let tid = generate_transaction_id();
        assert_eq!(tid.len(), 12);
    }
}