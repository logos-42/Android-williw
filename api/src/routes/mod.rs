pub mod auth;
pub mod compute;
pub mod payment;
pub mod local;

pub use auth::routes as auth_routes;
pub use compute::routes as compute_routes;
pub use payment::routes as payment_routes;
pub use local::routes as local_routes;
