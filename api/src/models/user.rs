use williw_shared::*;

/// 钱包登录请求结构
/// 用户使用钱包签名进行身份验证
#[derive(serde::Deserialize)]
pub struct LoginRequest {
    /// 钱包地址
    pub wallet_address: String,
    /// 签名数据
    pub signature: String,
    /// 原始消息内容
    pub message: String,
}

/// 登录成功的响应结构
#[derive(serde::Serialize)]
pub struct LoginResponse {
    /// JWT认证令牌
    pub token: String,
    /// 用户信息
    pub user: User,
}

/// 钱包签名验证请求结构
#[derive(serde::Deserialize)]
pub struct VerifyRequest {
    /// 钱包地址
    pub wallet_address: String,
    /// 签名数据
    pub signature: String,
}

/// 用户资料响应结构
/// 包含用户信息和统计摘要
#[derive(serde::Serialize)]
pub struct ProfileResponse {
    /// 用户详细信息
    pub user: User,
    /// 总订单数
    pub total_orders: i64,
    /// 总消费金额
    pub total_spent: f64,
}