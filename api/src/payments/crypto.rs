use thiserror::Error;
use williw_shared::CryptoPaymentInfo;

#[derive(Error, Debug)]
pub enum CryptoPayError {
    #[error("Invalid address")]
    InvalidAddress,
    #[error("Payment not found")]
    PaymentNotFound,
    #[error("Network error")]
    NetworkError,
}

pub struct CryptoPayment;

impl CryptoPayment {
    pub fn new() -> Self {
        Self
    }

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
