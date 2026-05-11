pub mod auth;
pub mod compute;
pub mod payment;

pub use auth::routes as auth_routes;
pub use compute::routes as compute_routes;
pub use payment::routes as payment_routes;
