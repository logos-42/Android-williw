use crate::db::Database;
use williw_shared::*;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;

/// 支付服务
/// 处理订单创建、支付和回调
pub struct PaymentService {
    /// 数据库连接
    db: Arc<Database>,
}

impl PaymentService {
    /// 创建支付服务实例
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 创建订单
    /// 
    /// # 参数
    /// * `user_id` - 用户ID
    /// * `model_id` - 模型ID
    /// * `amount` - 购买数量
    /// * `method` - 支付方式
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

    /// 发起支付
    /// 根据支付方式生成支付二维码或加密货币地址
    /// 
    /// # 参数
    /// * `order_id` - 订单ID
    /// * `method` - 支付方式
    pub async fn initiate_payment(
        &self,
        order_id: Uuid,
        method: PaymentMethod,
    ) -> Result<PaymentResponse, String> {
        // 获取订单信息
        let order = self.db.get_order(&order_id)
            .await
            .map_err(|e| format!("{}", e))?
            .ok_or("Order not found")?;

        // 获取模型信息计算总价
        let model = self.db.get_model_by_id(&order.model_id)
            .await
            .map_err(|e| format!("{}", e))?
            .ok_or("Model not found")?;

        let total_price = order.amount * model.price_per_unit;

        // 根据支付方式生成响应
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

    /// 获取订单状态
    /// 
    /// # 参数
    /// * `order_id` - 订单ID
    pub async fn get_order_status(&self, order_id: Uuid) -> Result<Option<Order>, String> {
        self.db.get_order(&order_id)
            .await
            .map_err(|e| format!("{}", e))
    }

    /// 确认支付
    /// 在支付回调成功后更新订单状态
    /// 
    /// # 参数
    /// * `order_id` - 订单ID
    pub async fn confirm_payment(&self, order_id: Uuid) -> Result<(), String> {
        self.db.update_order_status(&order_id, OrderStatus::Paid)
            .await
            .map_err(|e| format!("{}", e))
    }

    /// 获取用户订单列表
    /// 
    /// # 参数
    /// * `user_id` - 用户ID
    pub async fn get_user_orders(&self, user_id: Uuid) -> Result<Vec<Order>, String> {
        self.db.get_user_orders(&user_id)
            .await
            .map_err(|e| format!("{}", e))
    }
}