use crate::db::Database;
use crate::auth::jwt;
use crate::models::{LoginRequest, LoginResponse, ProfileResponse};
use williw_shared::*;
use std::sync::Arc;

pub struct AuthService {
    db: Arc<Database>,
}

impl AuthService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn wallet_login(&self, req: LoginRequest) -> Result<LoginResponse, String> {
        let address = req.wallet_address.to_lowercase();

        let user = match self.db.get_user_by_wallet(&address).await {
            Ok(Some(u)) => u,
            Ok(None) => {
                let new_user = User::new(address.clone());
                self.db.create_user(&new_user).await
                    .map_err(|e| format!("Failed to create user: {}", e))?;
                new_user
            }
            Err(e) => return Err(format!("Database error: {}", e)),
        };

        let token = jwt::create_token(user.id, &user.wallet_address)
            .map_err(|e| format!("Token creation failed: {}", e))?;

        Ok(LoginResponse { token, user })
    }

    pub async fn get_profile(&self, user_id: Uuid) -> Result<ProfileResponse, String> {
        let user = self.db.get_user_by_wallet(&user_id.to_string())
            .await
            .map_err(|e| format!("{}", e))?
            .ok_or("User not found")?;

        let orders = self.db.get_user_orders(&user_id).await
            .map_err(|e| format!("{}", e))?;

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
