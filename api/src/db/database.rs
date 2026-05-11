use sqlx::{SqlitePool, sqlite::SqlitePoolOptions, Row};
use uuid::Uuid;
use chrono::Utc;
use williw_shared::*;

/// 数据库连接池封装结构体
/// 提供与SQLite数据库的连接管理和所有数据库操作
pub struct Database {
    /// SQLx连接池
    pool: SqlitePool,
}

impl Database {
    /// 创建新的数据库连接实例
    /// 从环境变量DATABASE_URL获取连接字符串，默认使用sqlite:williw.db
    pub async fn new() -> Result<Self, sqlx::Error> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:williw.db?mode=rwc".to_string());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        Ok(Self { pool })
    }

    /// 运行数据库迁移，创建所有必要的表
    /// 包括users、models、orders和compute_requests四张表
    pub async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        // 创建用户表，存储钱包地址、邮箱和余额
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                wallet_address TEXT NOT NULL UNIQUE,
                email TEXT,
                balance REAL NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 创建AI模型表，存储模型信息、算力和价格
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS models (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider TEXT NOT NULL,
                category TEXT NOT NULL,
                description TEXT NOT NULL,
                compute_power REAL NOT NULL,
                price_per_unit REAL NOT NULL,
                status TEXT NOT NULL,
                image_url TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 创建订单表，记录用户购买计算资源的所有订单
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS orders (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                model_id TEXT NOT NULL,
                amount REAL NOT NULL,
                payment_method TEXT NOT NULL,
                status TEXT NOT NULL,
                crypto_amount REAL,
                crypto_currency TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id),
                FOREIGN KEY (model_id) REFERENCES models(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 创建计算请求表，记录用户对AI模型的调用
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS compute_requests (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                model_id TEXT NOT NULL,
                amount REAL NOT NULL,
                status TEXT NOT NULL,
                result TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id),
                FOREIGN KEY (model_id) REFERENCES models(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 初始化种子数据，向数据库插入默认AI模型列表
    /// 如果模型表已有数据则跳过
    pub async fn seed_models(&self) -> Result<(), sqlx::Error> {
        // 检查是否已有模型数据
        let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM models")
            .fetch_one(&self.pool)
            .await?
            .get("count");

        if count > 0 {
            return Ok(());
        }

        // 默认AI模型列表
        let models = vec![
            ("GPT-4 Turbo", "OpenAI", "llm", "Most powerful GPT model for complex tasks", 100.0, 0.03),
            ("GPT-3.5 Turbo", "OpenAI", "llm", "Fast and cost-effective language model", 50.0, 0.002),
            ("Claude 3 Opus", "Anthropic", "llm", "Anthropic's most capable model", 95.0, 0.015),
            ("Claude 3 Sonnet", "Anthropic", "llm", "Balanced performance and speed", 70.0, 0.003),
            ("Stable Diffusion XL", "StabilityAI", "image", "High-quality image generation", 80.0, 0.02),
            ("DALL-E 3", "OpenAI", "image", "State-of-the-art image creation", 90.0, 0.04),
            ("Whisper Large", "OpenAI", "audio", "Professional speech recognition", 60.0, 0.006),
            ("Gemini Pro", "Google", "multimodal", "Google's multimodal AI", 75.0, 0.005),
            ("Llama 3 70B", "Meta", "llm", "Open source large language model", 85.0, 0.01),
            ("Midjourney v6", "Midjourney", "image", "Artistic image generation", 65.0, 0.035),
        ];

        // 批量插入模型数据
        for (name, provider, category, desc, power, price) in models {
            sqlx::query(
                r#"INSERT INTO models (id, name, provider, category, description, compute_power, price_per_unit, status, image_url)
                   VALUES (?, ?, ?, ?, ?, ?, ?, 'active', NULL)"#,
            )
            .bind(Uuid::new_v4().to_string())
            .bind(name)
            .bind(provider)
            .bind(category)
            .bind(desc)
            .bind(power)
            .bind(price)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// 根据钱包地址查询用户
    /// 
    /// # 参数
    /// * `wallet` - 钱包地址字符串
    /// 
    /// # 返回
    /// 找到返回User对象，否则返回None
    pub async fn get_user_by_wallet(&self, wallet: &str) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM users WHERE wallet_address = ?")
            .bind(wallet)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| User {
            id: Uuid::parse_str(&r.get::<String, _>("id")).unwrap(),
            wallet_address: r.get("wallet_address"),
            email: r.get("email"),
            balance: r.get("balance"),
            created_at: chrono::DateTime::parse_from_rfc3339(&r.get::<String, _>("created_at"))
                .unwrap()
                .with_timezone(&Utc),
        }))
    }

    /// 创建新用户记录
    /// 
    /// # 参数
    /// * `user` - 用户对象引用
    pub async fn create_user(&self, user: &User) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO users (id, wallet_address, email, balance, created_at)
               VALUES (?, ?, ?, ?, ?)"#,
        )
        .bind(user.id.to_string())
        .bind(&user.wallet_address)
        .bind(&user.email)
        .bind(user.balance)
        .bind(user.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// 获取所有AI模型，支持多条件过滤
    /// 
    /// # 参数
    /// * `filter` - 模型过滤条件
    /// 
    /// # 过滤条件包括
    /// * category - 模型类别
    /// * provider - 提供商
    /// * min_power - 最小算力
    /// * max_price - 最大单价
    /// * search - 名称/描述关键词搜索
    pub async fn get_all_models(&self, filter: &ModelFilter) -> Result<Vec<AiModel>, sqlx::Error> {
        let mut query = "SELECT * FROM models WHERE status = 'active'".to_string();
        let mut params: Vec<String> = vec![];

        // 根据过滤条件动态构建查询
        if let Some(ref cat) = filter.category {
            query.push_str(" AND category = ?");
            params.push(cat.to_string());
        }
        if let Some(ref provider) = filter.provider {
            query.push_str(" AND provider = ?");
            params.push(provider.clone());
        }
        if let Some(min) = filter.min_power {
            query.push_str(" AND compute_power >= ?");
            params.push(min.to_string());
        }
        if let Some(max) = filter.max_price {
            query.push_str(" AND price_per_unit <= ?");
            params.push(max.to_string());
        }
        if let Some(ref search) = filter.search {
            query.push_str(" AND (name LIKE ? OR description LIKE ?)");
            params.push(format!("%{}%", search));
            params.push(format!("%{}%", search));
        }

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        // 将查询结果映射为AiModel对象
        let models: Vec<AiModel> = rows
            .into_iter()
            .map(|r| AiModel {
                id: Uuid::parse_str(&r.get::<String, _>("id")).unwrap(),
                name: r.get("name"),
                provider: r.get("provider"),
                category: serde_json::from_str(&format!("\"{}\"", r.get::<String, _>("category"))).unwrap(),
                description: r.get("description"),
                compute_power: r.get("compute_power"),
                price_per_unit: r.get("price_per_unit"),
                status: serde_json::from_str(&format!("\"{}\"", r.get::<String, _>("status"))).unwrap(),
                image_url: r.get("image_url"),
            })
            .collect();

        Ok(models)
    }

    /// 根据ID获取单个AI模型
    pub async fn get_model_by_id(&self, id: &Uuid) -> Result<Option<AiModel>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM models WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| AiModel {
            id: Uuid::parse_str(&r.get::<String, _>("id")).unwrap(),
            name: r.get("name"),
            provider: r.get("provider"),
            category: serde_json::from_str(&format!("\"{}\"", r.get::<String, _>("category"))).unwrap(),
            description: r.get("description"),
            compute_power: r.get("compute_power"),
            price_per_unit: r.get("price_per_unit"),
            status: serde_json::from_str(&format!("\"{}\"", r.get::<String, _>("status"))).unwrap(),
            image_url: r.get("image_url"),
        }))
    }

    /// 创建新订单记录
    /// 
    /// # 参数
    /// * `order` - 订单对象引用
    pub async fn create_order(&self, order: &Order) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO orders (id, user_id, model_id, amount, payment_method, status, crypto_amount, crypto_currency, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(order.id.to_string())
        .bind(order.user_id.to_string())
        .bind(order.model_id.to_string())
        .bind(order.amount)
        .bind(serde_json::to_string(&order.payment_method).unwrap())
        .bind(serde_json::to_string(&order.status).unwrap())
        .bind(order.crypto_amount)
        .bind(&order.crypto_currency)
        .bind(order.created_at.to_rfc3339())
        .bind(order.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// 根据ID获取订单详情
    pub async fn get_order(&self, id: &Uuid) -> Result<Option<Order>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM orders WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Order {
            id: Uuid::parse_str(&r.get::<String, _>("id")).unwrap(),
            user_id: Uuid::parse_str(&r.get::<String, _>("user_id")).unwrap(),
            model_id: Uuid::parse_str(&r.get::<String, _>("model_id")).unwrap(),
            amount: r.get("amount"),
            payment_method: serde_json::from_str(&r.get::<String, _>("payment_method")).unwrap(),
            status: serde_json::from_str(&r.get::<String, _>("status")).unwrap(),
            crypto_amount: r.get("crypto_amount"),
            crypto_currency: r.get("crypto_currency"),
            created_at: chrono::DateTime::parse_from_rfc3339(&r.get::<String, _>("created_at"))
                .unwrap()
                .with_timezone(&Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&r.get::<String, _>("updated_at"))
                .unwrap()
                .with_timezone(&Utc),
        }))
    }

    /// 更新订单状态
    /// 
    /// # 参数
    /// * `id` - 订单UUID
    /// * `status` - 新状态
    pub async fn update_order_status(&self, id: &Uuid, status: OrderStatus) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE orders SET status = ?, updated_at = ? WHERE id = ?")
            .bind(serde_json::to_string(&status).unwrap())
            .bind(Utc::now().to_rfc3339())
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 获取指定用户的所有订单（按时间倒序）
    pub async fn get_user_orders(&self, user_id: &Uuid) -> Result<Vec<Order>, sqlx::Error> {
        let rows = sqlx::query("SELECT * FROM orders WHERE user_id = ? ORDER BY created_at DESC")
            .bind(user_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        let orders: Vec<Order> = rows
            .into_iter()
            .map(|r| Order {
                id: Uuid::parse_str(&r.get::<String, _>("id")).unwrap(),
                user_id: Uuid::parse_str(&r.get::<String, _>("user_id")).unwrap(),
                model_id: Uuid::parse_str(&r.get::<String, _>("model_id")).unwrap(),
                amount: r.get("amount"),
                payment_method: serde_json::from_str(&r.get::<String, _>("payment_method")).unwrap(),
                status: serde_json::from_str(&r.get::<String, _>("status")).unwrap(),
                crypto_amount: r.get("crypto_amount"),
                crypto_currency: r.get("crypto_currency"),
                created_at: chrono::DateTime::parse_from_rfc3339(&r.get::<String, _>("created_at"))
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&r.get::<String, _>("updated_at"))
                    .unwrap()
                    .with_timezone(&Utc),
            })
            .collect();

        Ok(orders)
    }

    /// 创建新的计算请求记录
    /// 
    /// # 参数
    /// * `req` - 计算请求对象引用
    pub async fn create_compute_request(&self, req: &ComputeRequest) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO compute_requests (id, user_id, model_id, amount, status, result, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(req.id.to_string())
        .bind(req.user_id.to_string())
        .bind(req.model_id.to_string())
        .bind(req.amount)
        .bind(serde_json::to_string(&req.status).unwrap())
        .bind(&req.result)
        .bind(req.created_at.to_rfc3339())
        .bind(req.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// 根据ID获取计算请求详情
    pub async fn get_compute_request(&self, id: &Uuid) -> Result<Option<ComputeRequest>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM compute_requests WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| ComputeRequest {
            id: Uuid::parse_str(&r.get::<String, _>("id")).unwrap(),
            user_id: Uuid::parse_str(&r.get::<String, _>("user_id")).unwrap(),
            model_id: Uuid::parse_str(&r.get::<String, _>("model_id")).unwrap(),
            amount: r.get("amount"),
            status: serde_json::from_str(&r.get::<String, _>("status")).unwrap(),
            result: r.get("result"),
            created_at: chrono::DateTime::parse_from_rfc3339(&r.get::<String, _>("created_at"))
                .unwrap()
                .with_timezone(&Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&r.get::<String, _>("updated_at"))
                .unwrap()
                .with_timezone(&Utc),
        }))
    }
}