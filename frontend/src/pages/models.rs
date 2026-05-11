/// 模型列表页面组件
use leptos::*;
use crate::api::ApiClient;
use crate::components::ModelCard;
use williw_shared::ModelFilter;

/// 模型列表页面组件
/// 展示AI模型列表并支持过滤搜索
#[component]
pub fn Models() -> impl IntoView {
    // 模型列表
    let (models, set_models) = create_signal(Vec::<williw_shared::AiModel>::new());
    // 加载状态
    let (loading, set_loading) = create_signal(true);
    // 过滤条件
    let (filter, set_filter) = create_signal(ModelFilter::default());

    /// 加载模型数据
    let load_models = move || {
        spawn(async move {
            set_loading(true);
            let client = ApiClient::new();
            match client.get_models(Some(filter())).await {
                Ok(m) => set_models(m),
                Err(_) => {}
            }
            set_loading(false);
        });
    };

    on_mount(move || {
        load_models();
    });

    /// 更新过滤条件
    let update_filter = move |key: &str, value: String| {
        let mut f = filter();
        match key {
            "category" => f.category = serde_json::from_str(&format!("\"{}\"", value)).ok(),
            "provider" => f.provider = Some(value),
            "search" => f.search = Some(value),
            _ => {}
        }
        set_filter(f);
        load_models();
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

            <main class="max-w-7xl mx-auto px-4 py-6">
                // 过滤栏
                <div class="mb-6 bg-white rounded-lg shadow p-4">
                    <h2 class="text-lg font-semibold mb-4">Filter Models</h2>
                    <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">Category</label>
                            <select
                                class="w-full rounded border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                                on:change={move |ev| {
                                    let val = event_target_value(&ev);
                                    update_filter("category", val);
                                }}
                            >
                                <option value="">All</option>
                                <option value="llm">LLM</option>
                                <option value="image">Image</option>
                                <option value="audio">Audio</option>
                                <option value="video">Video</option>
                                <option value="multimodal">Multimodal</option>
                            </select>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">Provider</label>
                            <select
                                class="w-full rounded border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                                on:change={move |ev| {
                                    let val = event_target_value(&ev);
                                    update_filter("provider", val);
                                }}
                            >
                                <option value="">All</option>
                                <option value="OpenAI">OpenAI</option>
                                <option value="Anthropic">Anthropic</option>
                                <option value="StabilityAI">StabilityAI</option>
                                <option value="Google">Google</option>
                                <option value="Meta">Meta</option>
                            </select>
                        </div>
                        <div class="md:col-span-2">
                            <label class="block text-sm font-medium text-gray-700 mb-1">Search</label>
                            <input
                                type="text"
                                placeholder="Search models..."
                                class="w-full rounded border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                                on:input={move |ev| {
                                    let val = event_target_value(&ev);
                                    update_filter("search", val);
                                }}
                            />
                        </div>
                    </div>
                </div>

                // 模型网格
                {move || {
                    if loading() {
                        view! {
                            <div class="text-center py-12">
                                <p class="text-gray-600">Loading models...</p>
                            </div>
                        }
                    } else if models().is_empty() {
                        view! {
                            <div class="text-center py-12">
                                <p class="text-gray-600">No models found</p>
                            </div>
                        }
                    } else {
                        view! {
                            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
                                {models().into_iter().map(|model| {
                                    view! { <ModelCard model /> }
                                }).collect_view()}
                            </div>
                        }
                    }
                }}
            </main>
        </div>
    }
}