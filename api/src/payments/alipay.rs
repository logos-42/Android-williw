use thiserror::Error;

#[derive(Error, Debug)]
pub enum AlipayError {
    #[error("Invalid request")]
    InvalidRequest,
    #[error("Payment failed")]
    PaymentFailed,
    #[error("Callback verification failed")]
    CallbackVerificationFailed,
}

pub struct Alipay;

impl Alipay {
    pub fn new() -> Self {
        Self
    }

    pub fn create_qr_code(&self, amount: f64, order_id: &str) -> Result<String, AlipayError> {
        Ok(format!("alipay://pay?amount={}&order_id={}", amount, order_id))
    }

    pub fn verify_callback(&self, _data: &[u8]) -> Result<bool, AlipayError> {
        Ok(true)
    }
}

impl Default for Alipay {
    fn default() -> Self {
        Self::new()
    }
}
