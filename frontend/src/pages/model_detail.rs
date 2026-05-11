/// 模型详情页面组件
use leptos::*;
use uuid::Uuid;
use crate::api::ApiClient;
use williw_shared::{AiModel, PaymentMethod};

/// 模型详情页面属性
#[derive(Clone)]
pub struct ModelDetailProps {
    /// 模型ID
    pub id: Uuid,
}

/// 模型详情页面组件
/// 展示模型详细信息并提供购买选项
#[component]
pub fn ModelDetail(props: ModelDetailProps) -> impl IntoView {
    // 模型数据
    let (model, set_model) = create_signal(Option::<AiModel>::None);
    // 加载状态
    let (loading, set_loading) = create_signal(true);
    // 购买数量
    let (amount, set_amount) = create_signal(1.0);
    // 创建订单中状态
    let (creating, set_creating) = create_signal(false);
    // 订单ID
    let (order_id, set_order_id) = create_signal(Option::<Uuid>::None);

    let model_id = props.id;

    // 组件挂载时加载模型数据
    on_mount(move || {
        spawn(async move {
            let client = ApiClient::new();
            match client.get_model(model_id).await {
                Ok(m) => set_model(Some(m)),
                Err(_) => {}
            }
            set_loading(false);
        });
    });

    /// 处理购买请求
    let handle_request = move || {
        if let Some(m) = model() {
            set_creating(true);
            let model_id = m.id;
            let amt = amount();
            spawn(async move {
                let client = ApiClient::new();
                match client.create_order(model_id, amt, PaymentMethod::Usdt).await {
                    Ok(order) => {
                        let id = order.id;
                        set_order_id(Some(id));
                        // 跳转到支付页面
                        web_sys::window().unwrap().location().set_href(&format!("/payment/{}", id)).ok();
                    }
                    Err(_) => {
                        set_creating(false);
                    }
                }
            });
        }
    };

    view! {
        <div class="min-h-screen bg-gray-100">
            <nav class="bg-white shadow">
                <div class="max-w-7xl mx-auto px-4 py-4 flex justify-between items-center">
                    <a href="/" class="text-2xl font-bold text-gray-900">Williw</a>
                    <div class="flex gap-4">
                        <a href="/models" class="text-blue-600 font-medium">Models</a>
                        <a href="/orders" class="text-gray-600">Orders</a>
                        <a href="/profile" class="text-gray-600">Profile</a>
                    </div>
                </div>
            </nav>

            <main class="max-w-4xl mx-auto px-4 py-6">
                {move || {
                    if loading() {
                        view! { <p class="text-center py-12">Loading...</p> }
                    } else if let Some(m) = model() {
                        let category = format!("{:?}", m.category).to_lowercase();
                        view! {
                            <div class="bg-white rounded-lg shadow-lg overflow-hidden">
                                <div class="p-6">
                                    <div class="flex justify-between items-start mb-4">
                                        <div>
                                            <h1 class="text-3xl font-bold text-gray-900">{&m.name}</h1>
                                            <p class="text-gray-600">{&m.provider}</p>
                                        </div>
                                        <span class="px-3 py-1 bg-gray-200 rounded-full text-sm">{category}</span>
                                    </div>

                                    <p class="text-gray-700 mb-6">{&m.description}</p>

                                    <div class="grid grid-cols-2 gap-4 mb-6">
                                        <div class="bg-gray-50 p-4 rounded-lg">
                                            <p class="text-sm text-gray-600">Compute Power</p>
                                            <p class="text-2xl font-bold">{m.compute_power} TFLOPS</p>
                                        </div>
                                        <div class="bg-gray-50 p-4 rounded-lg">
                                            <p class="text-sm text-gray-600">Price per Unit</p>
                                            <p class="text-2xl font-bold">${m.price_per_unit}</p>
                                        </div>
                                    </div>

                                    <div class="border-t pt-6">
                                        <h3 class="text-lg font-semibold mb-4">Request Compute</h3>
                                        <div class="flex gap-4 items-end">
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 mb-1">Amount</label>
                                                <input
                                                    type="number"
                                                    min="1"
                                                    value={amount()}
                                                    on:input={move |ev| {
                                                        if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                                            set_amount(v);
                                                        }
                                                    }}
                                                    class="w-32 rounded border-gray-300 shadow-sm"
                                                />
                                            </div>
                                            <button
                                                class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
                                                disabled={creating()}
                                                on:click={move |_| handle_request()}
                                            >
                                                {if creating() { "Creating..." } else { "Create Order" }}
                                            </button>
                                        </div>
                                        <p class="text-sm text-gray-500 mt-2">
                                            Total: ${amount() * m.price_per_unit}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        }
                    } else {
                        view! { <p class="text-center py-12">Model not found</p> }
                    }
                }}
            </main>
        </div>
    }
}