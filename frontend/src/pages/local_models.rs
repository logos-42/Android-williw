use leptos::*;
use uuid::Uuid;
use crate::api::ApiClient;
use williw_shared::{LocalModel, LocalModelStatus};

#[component]
pub fn LocalModels() -> impl IntoView {
    let (models, set_models) = create_signal(Vec::<LocalModel>::new());
    let (loading, set_loading) = create_signal(true);
    let (device_info, set_device_info) = create_signal(Option::<williw_shared::DeviceInfo>::None);
    let (downloading, set_downloading) = create_signal(Option::<Uuid>::None);

    let load_data = move || {
        spawn(async move {
            set_loading(true);
            let client = ApiClient::new();

            match client.get_local_models().await {
                Ok(m) => set_models(m),
                Err(_) => {}
            }

            match client.get_device_info().await {
                Ok(info) => set_device_info(Some(info)),
                Err(_) => {}
            }

            set_loading(false);
        });
    };

    on_mount(move || {
        load_data();
    });

    let handle_download = move |model_id: Uuid| {
        set_downloading(Some(model_id));
        spawn(async move {
            let client = ApiClient::new();
            match client.download_model(model_id).await {
                Ok(_) => {
                    load_data();
                }
                Err(_) => {}
            }
            set_downloading(None);
        });
    };

    let handle_delete = move |model_id: Uuid| {
        spawn(async move {
            let client = ApiClient::new();
            match client.delete_local_model(model_id).await {
                Ok(_) => load_data(),
                Err(_) => {}
            }
        });
    };

    let handle_set_default = move |model_id: Uuid| {
        spawn(async move {
            let client = ApiClient::new();
            match client.set_default_model(model_id).await {
                Ok(_) => load_data(),
                Err(_) => {}
            }
        });
    };

    let status_badge = |status: &LocalModelStatus| -> String {
        match status {
            LocalModelStatus::NotDownloaded => "bg-gray-200 text-gray-700".to_string(),
            LocalModelStatus::Downloading => "bg-yellow-100 text-yellow-700".to_string(),
            LocalModelStatus::Downloaded => "bg-blue-100 text-blue-700".to_string(),
            LocalModelStatus::Installing => "bg-purple-100 text-purple-700".to_string(),
            LocalModelStatus::Ready => "bg-green-100 text-green-700".to_string(),
            LocalModelStatus::Error => "bg-red-100 text-red-700".to_string(),
        }
    };

    let status_text = |status: &LocalModelStatus| -> String {
        match status {
            LocalModelStatus::NotDownloaded => "Not Downloaded".to_string(),
            LocalModelStatus::Downloading => "Downloading...".to_string(),
            LocalModelStatus::Downloaded => "Downloaded".to_string(),
            LocalModelStatus::Installing => "Installing...".to_string(),
            LocalModelStatus::Ready => "Ready".to_string(),
            LocalModelStatus::Error => "Error".to_string(),
        }
    };

    view! {
        <div class="min-h-screen bg-gray-100 pb-20">
            <nav class="bg-white shadow">
                <div class="max-w-7xl mx-auto px-4 py-4 flex justify-between items-center">
                    <a href="/" class="text-2xl font-bold text-gray-900">Williw</a>
                    <div class="flex gap-4">
                        <a href="/models" class="text-gray-600">Cloud</a>
                        <a href="/local-models" class="text-blue-600 font-medium">Local</a>
                        <a href="/api-server" class="text-gray-600">API Server</a>
                    </div>
                </div>
            </nav>

            <main class="max-w-4xl mx-auto px-4 py-6">
                {move || {
                    if let Some(info) = device_info() {
                        view! {
                            <div class="bg-white rounded-lg shadow p-4 mb-6">
                                <div class="flex justify-between items-center">
                                    <div>
                                        <h2 class="text-lg font-semibold">{info.device_name}</h2>
                                        <p class="text-sm text-gray-600">
                                            Memory: {info.available_memory_gb}GB available |
                                            Storage: {info.available_storage_gb}GB available
                                        </p>
                                    </div>
                                    <div class="flex items-center gap-2">
                                        <span class={"px-3 py-1 rounded-full text-sm "}>
                                            {if info.is_server_running { "Server ON" } else { "Server OFF" }}
                                        </span>
                                        <span class="text-sm text-gray-500">:{(info.server_port)}</span>
                                    </div>
                                </div>
                            </div>
                        }
                    } else {
                        view! { <></> }
                    }
                }}

                <h1 class="text-2xl font-bold mb-6">Local Models</h1>

                {move || {
                    if loading() {
                        view! { <p class="text-center py-12">Loading models...</p> }
                    } else if models().is_empty() {
                        view! {
                            <div class="text-center py-12">
                                <p class="text-gray-600">No models available</p>
                            </div>
                        }
                    } else {
                        view! {
                            <div class="space-y-4">
                                {models().into_iter().map(|model| {
                                    let is_downloading = downloading() == Some(model.id);
                                    let m = model.clone();
                                    view! {
                                        <div class="bg-white rounded-lg shadow p-6">
                                            <div class="flex justify-between items-start">
                                                <div class="flex-1">
                                                    <div class="flex items-center gap-2 mb-2">
                                                        <h3 class="text-xl font-semibold">{&model.name}</h3>
                                                        {if model.is_default {
                                                            view! { <span class="px-2 py-0.5 bg-blue-100 text-blue-700 text-xs rounded">Default</span> }
                                                        } else {
                                                            view! { <></> }
                                                        }}
                                                    </div>
                                                    <p class="text-sm text-gray-600 mb-2">{format!("{:?}", model.model_type)}</p>
                                                    <p class="text-gray-500 text-sm mb-3">{model.size_gb} GB</p>

                                                    <div class="flex items-center gap-2">
                                                        <span class={"px-3 py-1 rounded-full text-sm "}>{status_text(&model.status)}</span>
                                                        {if let Some(progress) = model.download_progress {
                                                            view! {
                                                                <div class="flex-1 bg-gray-200 rounded-full h-2">
                                                                    <div class="bg-blue-600 h-2 rounded-full" style={format!("width: {}%", progress)}></div>
                                                                </div>
                                                            }
                                                        } else {
                                                            view! { <></> }
                                                        }}
                                                    </div>
                                                </div>

                                                <div class="flex flex-col gap-2 ml-4">
                                                    {match model.status {
                                                        LocalModelStatus::NotDownloaded => {
                                                            view! {
                                                                <button
                                                                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
                                                                    disabled={is_downloading}
                                                                    on:click={move |_| handle_download(m.id)}
                                                                >
                                                                    {if is_downloading { "Downloading..." } else { "Download" }}
                                                                </button>
                                                            }
                                                        },
                                                        LocalModelStatus::Downloading => {
                                                            view! {
                                                                <button class="px-4 py-2 bg-gray-400 text-white rounded-lg cursor-not-allowed" disabled>
                                                                    Downloading...
                                                                </button>
                                                            }
                                                        },
                                                        LocalModelStatus::Ready => {
                                                            view! {
                                                                <div class="flex flex-col gap-2">
                                                                    <button
                                                                        class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700"
                                                                        on:click={move |_| handle_set_default(m.id)}
                                                                    >
                                                                        Set Default
                                                                    </button>
                                                                    <button
                                                                        class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700"
                                                                        on:click={move |_| handle_delete(m.id)}
                                                                    >
                                                                        Delete
                                                                    </button>
                                                                </div>
                                                            }
                                                        },
                                                        _ => {
                                                            view! {
                                                                <button
                                                                    class="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700"
                                                                    on:click={move |_| handle_download(m.id)}
                                                                >
                                                                    Retry
                                                                </button>
                                                            }
                                                        }
                                                    }}
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }
                    }
                }}
            </main>

            <nav class="fixed bottom-0 left-0 right-0 bg-white border-t">
                <div class="flex justify-around py-2">
                    <a href="/local-models" class="flex flex-col items-center p-2 text-blue-600">
                        <span class="text-2xl">📥</span>
                        <span class="text-xs">Downloads</span>
                    </a>
                    <a href="/api-server" class="flex flex-col items-center p-2 text-gray-600">
                        <span class="text-2xl">🔌</span>
                        <span class="text-xs">API Server</span>
                    </a>
                    <a href="/profile" class="flex flex-col items-center p-2 text-gray-600">
                        <span class="text-2xl">👤</span>
                        <span class="text-xs">Profile</span>
                    </a>
                </div>
            </nav>
        </div>
    }
}
