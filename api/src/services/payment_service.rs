use crate::db::Database;
use williw_shared::*;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;

pub struct PaymentService {
    db: Arc<Database>,
}

impl PaymentService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn create_order(
        &self,
        user_id: Uuid,
        model_id: Uuid,
        amount: f64,
        method: PaymentMethod,
    ) -> Result<Order, String> {
        let order = Order {
            id: Uuid::new_v4(),
            user_id,
            model_id,
            amount,
            payment_method: method.clone(),
            status: OrderStatus::Pending,
            crypto_amount: None,
            crypto_currency: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.db.create_order(&order).await
            .map_err(|e| format!("Failed to create order: {}", e))?;

        Ok(order)
    }

    pub async fn initiate_payment(
        &self,
        order_id: Uuid,
        method: PaymentMethod,
    ) -> Result<PaymentResponse, String> {
        let order = self.db.get_order(&order_id)
            .await
            .map_err(|e| format!("{}", e))?
            .ok_or("Order not found")?;

        let model = self.db.get_model_by_id(&order.model_id)
            .await
            .map_err(|e| format!("{}", e))?
            .ok_or("Model not found")?;

        let total_price = order.amount * model.price_per_unit;

        match method {
            PaymentMethod::Wechat => Ok(PaymentResponse {
                qr_code: Some(format!("wechat://pay?amount={}", total_price)),
                payment_url: None,
                crypto_address: None,
                order_id,
            }),
            PaymentMethod::Alipay => Ok(PaymentResponse {
                qr_code: Some(format!("alipay://pay?amount={}", total_price)),
                payment_url: None,
                crypto_address: None,
                order_id,
            }),
            PaymentMethod::Usdt => Ok(PaymentResponse {
                qr_code: None,
                payment_url: None,
                crypto_address: Some(CryptoPaymentInfo {
                    address: std::env::var("USDT_ADDRESS").unwrap_or_else(|_| "TRC20_ADDRESS_PLACEHOLDER".to_string()),
                    amount: total_price,
                    currency: "USDT".to_string(),
                    network: "TRC20".to_string(),
                }),
                order_id,
            }),
            PaymentMethod::Eth => Ok(PaymentResponse {
                qr_code: None,
                payment_url: None,
                crypto_address: Some(CryptoPaymentInfo {
                    address: std::env::var("ETH_ADDRESS").unwrap_or_else(|_| "ETH_ADDRESS_PLACEHOLDER".to_string()),
                    amount: total_price,
                    currency: "ETH".to_string(),
                    network: "Ethereum".to_string(),
                }),
                order_id,
            }),
            PaymentMethod::Btc => Ok(PaymentResponse {
                qr_code: None,
                payment_url: None,
                crypto_address: Some(CryptoPaymentInfo {
                    address: std::env::var("BTC_ADDRESS").unwrap_or_else(|_| "BTC_ADDRESS_PLACEHOLDER".to_string()),
                    amount: total_price,
                    currency: "BTC".to_string(),
                    network: "Bitcoin".to_string(),
                }),
                order_id,
            }),
        }
    }

    pub async fn get_order_status(&self, order_id: Uuid) -> Result<Option<Order>, String> {
        self.db.get_order(&order_id)
            .await
            .map_err(|e| format!("{}", e))
    }

    pub async fn confirm_payment(&self, order_id: Uuid) -> Result<(), String> {
        self.db.update_order_status(&order_id, OrderStatus::Paid)
            .await
            .map_err(|e| format!("{}", e))
    }

    pub async fn get_user_orders(&self, user_id: Uuid) -> Result<Vec<Order>, String> {
        self.db.get_user_orders(&user_id)
            .await
            .map_err(|e| format!("{}", e))
    }
}
