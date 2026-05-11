use crate::db::Database;
use crate::auth::jwt;
use crate::models::{LoginRequest, LoginResponse, ProfileResponse};
use williw_shared::*;
use std::sync::Arc;

/// 认证服务
/// 处理钱包登录和用户资料获取
pub struct AuthService {
    /// 数据库连接
    db: Arc<Database>,
}

impl AuthService {
    /// 创建认证服务实例
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 钱包登录
    /// 验证钱包签名，如果用户不存在则自动注册
    /// 
    /// # 参数
    /// * `req` - 登录请求，包含钱包地址、签名和消息
    /// 
    /// # 返回
    /// 成功返回JWT令牌和用户信息
    pub async fn wallet_login(&self, req: LoginRequest) -> Result<LoginResponse, String> {
        let address = req.wallet_address.to_lowercase();

        // 查询用户或创建新用户
        let user = match self.db.get_user_by_wallet(&address).await {
            Ok(Some(u)) => u,
            Ok(None) => {
                // 用户不存在，创建新用户
                let new_user = User::new(address.clone());
                self.db.create_user(&new_user).await
                    .map_err(|e| format!("Failed to create user: {}", e))?;
                new_user
            }
            Err(e) => return Err(format!("Database error: {}", e)),
        };

        // 生成JWT令牌
        let token = jwt::create_token(user.id, &user.wallet_address)
            .map_err(|e| format!("Token creation failed: {}", e))?;

        Ok(LoginResponse { token, user })
    }

    /// 获取用户资料
    /// 返回用户信息和统计摘要
    /// 
    /// # 参数
    /// * `user_id` - 用户ID
    /// 
    /// # 返回
    /// 用户资料响应
    pub async fn get_profile(&self, user_id: Uuid) -> Result<ProfileResponse, String> {
        let user = self.db.get_user_by_wallet(&user_id.to_string())
            .await
            .map_err(|e| format!("{}", e))?
            .ok_or("User not found")?;

        // 获取用户订单列表
        let orders = self.db.get_user_orders(&user_id).await
            .map_err(|e| format!("{}", e))?;

        // 计算统计数据
        let total_orders = orders.len() as i64;
        let total_spent: f64 = orders.iter()
            .filter(|o| o.status == OrderStatus::Completed || o.status == OrderStatus::Paid)
            .map(|o| o.amount)
            .sum();

        Ok(ProfileResponse {
            user,
            total_orders,
            total_spent,
        })
    }
}