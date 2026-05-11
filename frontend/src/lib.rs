pub mod api;
pub mod pages;
pub mod components;

use leptos::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/" component=pages::Home />
                <Route path="/login" component=pages::Login />
                <Route path="/models" component=pages::Models />
                <Route path="/models/:id" component=pages::ModelDetail />
                <Route path="/payment/:order_id" component=pages::Payment />
                <Route path="/orders" component=pages::Orders />
                <Route path="/profile" component=pages::Profile />
            </Routes>
        </Router>
    }
}
