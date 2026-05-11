/// 路由模块
/// 定义所有API端点的路由规则

pub mod auth;
pub mod compute;
pub mod payment;
pub mod local;
pub mod p2p;

// 重新导出路由工厂函数
pub use auth::routes as auth_routes;
pub use compute::routes as compute_routes;
pub use payment::routes as payment_routes;
pub use local::routes as local_routes;
pub use p2p::routes as p2p_routes;