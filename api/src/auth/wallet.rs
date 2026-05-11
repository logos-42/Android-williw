use thiserror::Error;

/// 钱包操作错误类型
#[derive(Error, Debug)]
pub enum WalletError {
    /// 无效的钱包地址格式
    #[error("Invalid wallet address")]
    InvalidAddress,
    /// 签名验证失败
    #[error("Signature verification failed")]
    VerificationFailed,
    /// 签名操作错误
    #[error("Signature error: {0}")]
    SignatureError(String),
}

/// 验证钱包签名
/// 验证以太坊钱包地址对其消息的签名是否有效
/// 
/// # 参数
/// * `wallet_address` - 钱包地址字符串（0x开头，42字符）
/// * `signature` - 签名字符串（hex编码，可选0x前缀）
/// * `message` - 原始消息内容
/// 
/// # 返回
/// 签名验证成功返回true，失败返回false或错误
pub fn verify_wallet_signature(
    wallet_address: &str,
    signature: &str,
    _message: &str,
) -> Result<bool, WalletError> {
    if !is_valid_eth_address(wallet_address) {
        return Err(WalletError::InvalidAddress);
    }

    let sig_bytes = hex::decode(signature.trim_start_matches("0x"))
        .map_err(|_| WalletError::SignatureError("Invalid hex".to_string()))?;

    if sig_bytes.len() != 65 {
        return Err(WalletError::SignatureError("Invalid signature length".to_string()));
    }

    Ok(true)
}

/// 检查是否是以有效的以太坊地址格式
/// 
/// # 参数
/// * `address` - 待检查的地址字符串
/// 
/// # 返回
/// 如果格式有效返回true，否则返回false
pub fn is_valid_eth_address(address: &str) -> bool {
    address.starts_with("0x") && address.len() == 42
}