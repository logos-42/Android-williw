/// 支付页面组件
use leptos::*;
use uuid::Uuid;
use crate::api::ApiClient;
use williw_shared::{PaymentMethod, PaymentResponse, Order};

/// 支付页面属性
#[derive(Clone)]
pub struct PaymentProps {
    /// 订单ID
    pub order_id: Uuid,
}

/// 支付页面组件
/// 展示订单信息并提供多种支付方式
#[component]
pub fn Payment(props: PaymentProps) -> impl IntoView {
    // 订单数据
    let (order, set_order) = create_signal(Option::<Order>::None);
    // 支付信息
    let (payment_info, set_payment_info) = create_signal(Option::<PaymentResponse>::None);
    // 加载状态
    let (loading, set_loading) = create_signal(true);
    // 选中的支付方式
    let (selected_method, set_selected_method) = create_signal(PaymentMethod::Usdt);

    let order_id = props.order_id;

    // 组件挂载时加载订单状态
    on_mount(move || {
        spawn(async move {
            let client = ApiClient::new();
            match client.get_order_status(order_id).await {
                Ok(o) => set_order(Some(o)),
                Err(_) => {}
            }
            set_loading(false);
        });
    });

    /// 发起支付
    let initiate_payment = move |method: PaymentMethod| {
        set_loading(true);
        set_selected_method(method);
        let order_id = order_id;
        spawn(async move {
            let client = ApiClient::new();
            match client.initiate_payment(order_id, method).await {
                Ok(info) => {
                    set_payment_info(Some(info));
                    set_loading(false);
                }
                Err(_) => set_loading(false),
            }
        });
    };

    /// 获取支付方式标签
    let method_labels = |method: &PaymentMethod| -> String {
        match method {
            PaymentMethod::Wechat => "WeChat Pay".to_string(),
            PaymentMethod::Alipay => "Alipay".to_string(),
            PaymentMethod::Usdt => "USDT".to_string(),
            PaymentMethod::Eth => "ETH".to_string(),
            PaymentMethod::Btc => "BTC".to_string(),
        }
    };

    view! {
        <div class="min-h-screen bg-gray-100">
            <nav class="bg-white shadow">
                <div class="max-w-7xl mx-auto px-4 py-4 flex justify-between items-center">
                    <a href="/" class="text-2xl font-bold text-gray-900">Williw</a>
                    <div class="flex gap-4">
                        <a href="/models" class="text-gray-600">Models</a>
                        <a href="/orders" class="text-gray-600">Orders</a>
                        <a href="/profile" class="text-gray-600">Profile</a>
                    </div>
                </div>
            </nav>

            <main class="max-w-2xl mx-auto px-4 py-6">
                <h1 class="text-2xl font-bold mb-6">Complete Payment</h1>

                {move || {
                    if loading() {
                        view! { <p class="text-center">Loading...</p> }
                    } else if let Some(o) = order() {
                        let is_paid = matches!(o.status, williw_shared::OrderStatus::Paid | williw_shared::OrderStatus::Completed);
                        view! {
                            <div class="bg-white rounded-lg shadow p-6 mb-6">
                                <h2 class="text-lg font-semibold mb-4">Order #{o.id.to_string()[..8].to_string()}</h2>
                                <p class="text-2xl font-bold mb-4">${o.amount}</p>
                                <p class={"text-xl font-semibold "}>{method_labels(&o.payment_method)}</p>
                            </div>

                            {if is_paid {
                                view! {
                                    <div class="bg-green-100 border border-green-400 text-green-700 px-4 py-3 rounded">
                                        <p class="font-bold">Payment Complete!</p>
                                        <p>Your order is being processed.</p>
                                    </div>
                                }
                            } else if payment_info().is_some() {
                                let info = payment_info().unwrap();
                                view! {
                                    <div class="bg-white rounded-lg shadow p-6">
                                        {if let Some(crypto) = info.crypto_address {
                                            view! {
                                                <div class="text-center">
                                                    <p class="text-lg font-semibold mb-4">Send {crypto.amount} {crypto.currency}</p>
                                                    <div class="bg-gray-100 p-4 rounded-lg mb-4 break-all">
                                                        <p class="text-sm text-gray-600">Address</p>
                                                        <p class="font-mono text-lg">{crypto.address}</p>
                                                    </div>
                                                    <p class="text-sm text-gray-600">Network: {crypto.network}</p>
                                                </div>
                                            }
                                        } else if let Some(qr) = info.qr_code {
                                            view! {
                                                <div class="text-center">
                                                    <p class="text-lg font-semibold mb-4">Scan QR Code</p>
                                                    <div class="bg-gray-100 p-4 rounded-lg inline-block">
                                                        <p class="font-mono">{qr}</p>
                                                    </div>
                                                </div>
                                            }
                                        } else {
                                            view! { <p>Payment info not available</p> }
                                        }}
                                    </div>
                                }
                            } else {
                                view! {
                                    <div class="bg-white rounded-lg shadow p-6">
                                        <h3 class="text-lg font-semibold mb-4">Select Payment Method</h3>
                                        <div class="grid grid-cols-1 gap-3">
                                            <button
                                                class="p-4 border rounded-lg hover:border-blue-500 transition flex items-center justify-between"
                                                on:click={move |_| initiate_payment(PaymentMethod::Wechat)}
                                            >
                                                <span>WeChat Pay</span>
                                                <span class="text-2xl">💬</span>
                                            </button>
                                            <button
                                                class="p-4 border rounded-lg hover:border-blue-500 transition flex items-center justify-between"
                                                on:click={move |_| initiate_payment(PaymentMethod::Alipay)}
                                            >
                                                <span>Alipay</span>
                                                <span class="text-2xl">💴</span>
                                            </button>
                                            <button
                                                class="p-4 border rounded-lg hover:border-blue-500 transition flex items-center justify-between"
                                                on:click={move |_| initiate_payment(PaymentMethod::Usdt)}
                                            >
                                                <span>USDT (TRC20)</span>
                                                <span class="text-2xl">🔺</span>
                                            </button>
                                            <button
                                                class="p-4 border rounded-lg hover:border-blue-500 transition flex items-center justify-between"
                                                on:click={move |_| initiate_payment(PaymentMethod::Eth)}
                                            >
                                                <span>Ethereum (ETH)</span>
                                                <span class="text-2xl">⟠</span>
                                            </button>
                                            <button
                                                class="p-4 border rounded-lg hover:border-blue-500 transition flex items-center justify-between"
                                                on:click={move |_| initiate_payment(PaymentMethod::Btc)}
                                            >
                                                <span>Bitcoin (BTC)</span>
                                                <span class="text-2xl">₿</span>
                                            </button>
                                        </div>
                                    </div>
                                }
                            }}
                        }
                    } else {
                        view! { <p>Order not found</p> }
                    }
                }}
            </main>
        </div>
    }
}