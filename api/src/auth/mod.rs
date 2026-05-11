/// 认证模块
/// 提供JWT令牌和钱包签名验证功能

pub mod jwt;
pub mod wallet;

// 重新导出AuthLayer以方便外部使用
pub use jwt::AuthLayer;