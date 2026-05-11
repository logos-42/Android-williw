use thiserror::Error;

#[derive(Error, Debug)]
pub enum WechatPayError {
    #[error("Invalid request")]
    InvalidRequest,
    #[error("Payment failed")]
    PaymentFailed,
    #[error("Callback verification failed")]
    CallbackVerificationFailed,
}

pub struct WechatPay;

impl WechatPay {
    pub fn new() -> Self {
        Self
    }

    pub fn create_qr_code(&self, amount: f64, order_id: &str) -> Result<String, WechatPayError> {
        Ok(format!("wechat://pay?amount={}&order_id={}", amount, order_id))
    }

    pub fn verify_callback(&self, _data: &[u8]) -> Result<bool, WechatPayError> {
        Ok(true)
    }
}

impl Default for WechatPay {
    fn default() -> Self {
        Self::new()
    }
}
