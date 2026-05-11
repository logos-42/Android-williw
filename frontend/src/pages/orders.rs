use leptos::*;
use crate::api::ApiClient;
use williw_shared::Order;

#[component]
pub fn Orders() -> impl IntoView {
    let (orders, set_orders) = create_signal(Vec::<Order>::new());
    let (loading, set_loading) = create_signal(true);

    on_mount(move || {
        spawn(async move {
            let client = ApiClient::new();
            match client.get_user_orders().await {
                Ok(o) => set_orders(o),
                Err(_) => {}
            }
            set_loading(false);
        });
    });

    let status_color = |status: &williw_shared::OrderStatus| -> String {
        match status {
            williw_shared::OrderStatus::Pending => "bg-yellow-100 text-yellow-800".to_string(),
            williw_shared::OrderStatus::Paid => "bg-blue-100 text-blue-800".to_string(),
            williw_shared::OrderStatus::Processing => "bg-purple-100 text-purple-800".to_string(),
            williw_shared::OrderStatus::Completed => "bg-green-100 text-green-800".to_string(),
            williw_shared::OrderStatus::Failed => "bg-red-100 text-red-800".to_string(),
            williw_shared::OrderStatus::Refunded => "bg-gray-100 text-gray-800".to_string(),
        }
    };

    let format_status = |status: &williw_shared::OrderStatus| -> String {
        format!("{:?}", status).to_lowercase()
    };

    view! {
        <div class="min-h-screen bg-gray-100">
            <nav class="bg-white shadow">
                <div class="max-w-7xl mx-auto px-4 py-4 flex justify-between items-center">
                    <a href="/" class="text-2xl font-bold text-gray-900">Williw</a>
                    <div class="flex gap-4">
                        <a href="/models" class="text-gray-600">Models</a>
                        <a href="/orders" class="text-blue-600 font-medium">Orders</a>
                        <a href="/profile" class="text-gray-600">Profile</a>
                    </div>
                </div>
            </nav>

            <main class="max-w-4xl mx-auto px-4 py-6">
                <h1 class="text-2xl font-bold mb-6">My Orders</h1>

                {move || {
                    if loading() {
                        view! { <p class="text-center">Loading...</p> }
                    } else if orders().is_empty() {
                        view! {
                            <div class="bg-white rounded-lg shadow p-8 text-center">
                                <p class="text-gray-600 mb-4">No orders yet</p>
                                <a href="/models" class="text-blue-600 hover:underline">Browse Models</a>
                            </div>
                        }
                    } else {
                        view! {
                            <div class="space-y-4">
                                {orders().into_iter().map(|order| {
                                    let status = order.status.clone();
                                    let method = order.payment_method.clone();
                                    view! {
                                        <div class="bg-white rounded-lg shadow p-6">
                                            <div class="flex justify-between items-start">
                                                <div>
                                                    <p class="font-mono text-sm text-gray-500">#{order.id.to_string()[..8].to_string()}</p>
                                                    <p class="text-lg font-semibold mt-1">${order.amount}</p>
                                                </div>
                                                <span class={"px-3 py-1 rounded-full text-sm "}>{format_status(&order.status)}</span>
                                            </div>
                                            <div class="mt-4 flex justify-between items-center text-sm">
                                                <span class="text-gray-600">
                                                    {match method {
                                                        williw_shared::PaymentMethod::Wechat => "WeChat",
                                                        williw_shared::PaymentMethod::Alipay => "Alipay",
                                                        williw_shared::PaymentMethod::Usdt => "USDT",
                                                        williw_shared::PaymentMethod::Eth => "ETH",
                                                        williw_shared::PaymentMethod::Btc => "BTC",
                                                    }}
                                                </span>
                                                <span class="text-gray-500">
                                                    {order.created_at.format("%Y-%m-%d %H:%M")}
                                                </span>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }
                    }
                }}
            </main>
        </div>
    }
}
