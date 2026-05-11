use thiserror::Error;
use williw_shared::CryptoPaymentInfo;

/// 加密货币支付错误类型
#[derive(Error, Debug)]
pub enum CryptoPayError {
    /// 无效地址
    #[error("Invalid address")]
    InvalidAddress,
    /// 未找到支付记录
    #[error("Payment not found")]
    PaymentNotFound,
    /// 网络错误
    #[error("Network error")]
    NetworkError,
}

/// 加密货币支付处理
pub struct CryptoPayment;

impl CryptoPayment {
    /// 创建新的加密货币支付处理器
    pub fn new() -> Self {
        Self
    }

    /// 生成支付地址
    /// 根据货币类型返回对应的收款地址
    /// 
    /// # 参数
    /// * `currency` - 货币类型（USDT、ETH、BTC）
    /// 
    /// # 返回
    /// 收款地址
    pub fn generate_address(&self, currency: &str) -> Result<String, CryptoPayError> {
        match currency {
            "USDT" => Ok(std::env::var("USDT_TRC20_ADDRESS")
                .unwrap_or_else(|_| "TRC20_USDT_PLACEHOLDER".to_string())),
            "ETH" => Ok(std::env::var("ETH_ADDRESS")
                .unwrap_or_else(|_| "ETH_PLACEHOLDER".to_string())),
            "BTC" => Ok(std::env::var("BTC_ADDRESS")
                .unwrap_or_else(|_| "BTC_PLACEHOLDER".to_string())),
            _ => Err(CryptoPayError::InvalidAddress),
        }
    }

    /// 创建加密货币支付信息
    /// 
    /// # 参数
    /// * `amount` - 支付金额
    /// * `currency` - 货币类型
    /// 
    /// # 返回
    /// 完整的支付信息，包含地址、金额、网络等
    pub fn create_payment_info(&self, amount: f64, currency: &str) -> Result<CryptoPaymentInfo, CryptoPayError> {
        let address = self.generate_address(currency)?;
        let network = match currency {
            "USDT" => "TRC20",
            "ETH" => "Ethereum",
            "BTC" => "Bitcoin",
            _ => return Err(CryptoPayError::InvalidAddress),
        };

        Ok(CryptoPaymentInfo {
            address,
            amount,
            currency: currency.to_string(),
            network: network.to_string(),
        })
    }
}

impl Default for CryptoPayment {
    fn default() -> Self {
        Self::new()
    }
}