use leptos::*;
use crate::api::ApiClient;
use williw_shared::LocalApiConfig;

#[component]
pub fn ApiServer() -> impl IntoView {
    let (config, set_config) = create_signal(Option::<LocalApiConfig>::None);
    let (device_info, set_device_info) = create_signal(Option::<williw_shared::DeviceInfo>::None);
    let (loading, set_loading) = create_signal(true);
    let (server_status, set_server_status) = create_signal(false);
    let (message, set_message) = create_signal(Option::<String>::None);

    let load_data = move || {
        spawn(async move {
            set_loading(true);
            let client = ApiClient::new();

            match client.get_server_config().await {
                Ok(c) => set_config(Some(c)),
                Err(_) => {}
            }

            match client.get_device_info().await {
                Ok(info) => {
                    set_device_info(Some(info.clone()));
                    set_server_status(info.is_server_running);
                }
                Err(_) => {}
            }

            set_loading(false);
        });
    };

    on_mount(move || {
        load_data();
    });

    let handle_toggle_server = move || {
        spawn(async move {
            let client = ApiClient::new();
            let result = if server_status() {
                client.stop_local_server().await
            } else {
                client.start_local_server().await
            };

            match result {
                Ok(msg) => {
                    set_message(Some(msg));
                    set_server_status(!server_status());
                }
                Err(e) => {
                    set_message(Some(format!("Error: {}", e)));
                }
            }
        });
    };

    let handle_update_config = move |new_config: LocalApiConfig| {
        spawn(async move {
            let client = ApiClient::new();
            match client.update_server_config(new_config.clone()).await {
                Ok(c) => {
                    set_config(Some(c));
                    set_message(Some("Configuration saved".to_string()));
                }
                Err(e) => {
                    set_message(Some(format!("Error: {}", e)));
                }
            }
        });
    };

    let local_ip = "192.168.1.xxx";

    view! {
        <div class="min-h-screen bg-gray-100 pb-20">
            <nav class="bg-white shadow">
                <div class="max-w-7xl mx-auto px-4 py-4 flex justify-between items-center">
                    <a href="/" class="text-2xl font-bold text-gray-900">Williw</a>
                    <div class="flex gap-4">
                        <a href="/local-models" class="text-gray-600">Local Models</a>
                        <a href="/api-server" class="text-blue-600 font-medium">API Server</a>
                        <a href="/profile" class="text-gray-600">Profile</a>
                    </div>
                </div>
            </nav>

            <main class="max-w-2xl mx-auto px-4 py-6">
                <h1 class="text-2xl font-bold mb-6">Local API Server</h1>

                {move || {
                    if let Some(msg) = message() {
                        view! {
                            <div class="bg-blue-100 border border-blue-400 text-blue-700 px-4 py-3 rounded mb-6">
                                {msg}
                            </div>
                        }
                    } else {
                        view! { <></> }
                    }
                }}

                <div class="bg-white rounded-lg shadow p-6 mb-6">
                    <h2 class="text-lg font-semibold mb-4">Server Status</h2>

                    <div class="flex items-center justify-between mb-4">
                        <div class="flex items-center gap-3">
                            <span class={"w-4 h-4 rounded-full ".to_string() + if server_status() { "bg-green-500" } else { "bg-gray-400" }}></span>
                            <span class="font-medium">
                                {if server_status() { "Server Running" } else { "Server Stopped" }}
                            </span>
                        </div>
                        <button
                            class={"px-6 py-2 rounded-lg font-medium text-white transition ".to_string() + if server_status() { "bg-red-600 hover:bg-red-700" } else { "bg-green-600 hover:bg-green-700" }}
                            on:click={move |_| handle_toggle_server()}
                        >
                            {if server_status() { "Stop Server" } else { "Start Server" }}
                        </button>
                    </div>

                    {if server_status() {
                        let info = device_info();
                        view! {
                            <div class="bg-gray-50 p-4 rounded-lg">
                                <h3 class="font-medium mb-3">Connection Info</h3>
                                <div class="space-y-2 text-sm">
                                    <div class="flex justify-between">
                                        <span class="text-gray-600">Local:</span>
                                        <code class="bg-gray-200 px-2 py-0.5 rounded">http://localhost:{info.map(|i| i.server_port).unwrap_or(8081)}</code>
                                    </div>
                                    <div class="flex justify-between">
                                        <span class="text-gray-600">LAN:</span>
                                        <code class="bg-gray-200 px-2 py-0.5 rounded">http://{local_ip}:{info.map(|i| i.server_port).unwrap_or(8081)}</code>
                                    </div>
                                    <p class="text-xs text-gray-500 mt-3">
                                        Other devices on the same network can access your local models via the LAN address.
                                    </p>
                                </div>
                            </div>
                        }
                    } else {
                        view! {
                            <div class="bg-gray-50 p-4 rounded-lg">
                                <p class="text-sm text-gray-600">
                                    Start the server to allow other devices to use your downloaded models via API.
                                </p>
                            </div>
                        }
                    }}
                </div>

                <div class="bg-white rounded-lg shadow p-6">
                    <h2 class="text-lg font-semibold mb-4">Server Configuration</h2>

                    {move || {
                        if let Some(c) = config() {
                            view! {
                                <div class="space-y-4">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                            Port
                                        </label>
                                        <input
                                            type="number"
                                            value={c.port}
                                            class="w-full px-4 py-2 border border-gray-300 rounded-lg"
                                            on:input={move |ev| {
                                                if let Ok(port) = event_target_value(&ev).parse::<u16>() {
                                                    let mut new_config = c.clone();
                                                    new_config.port = port;
                                                    handle_update_config(new_config);
                                                }
                                            }}
                                        />
                                    </div>

                                    <div class="flex items-center justify-between">
                                        <div>
                                            <p class="font-medium">External Access</p>
                                            <p class="text-sm text-gray-500">Allow devices outside your network</p>
                                        </div>
                                        <button
                                            class={"w-12 h-6 rounded-full transition ".to_string() + if c.allow_external_access { "bg-blue-600" } else { "bg-gray-300" }}
                                            on:click={move |_| {
                                                let mut new_config = c.clone();
                                                new_config.allow_external_access = !c.allow_external_access;
                                                handle_update_config(new_config);
                                            }}
                                        >
                                            <span class={"block w-5 h-5 bg-white rounded-full transition transform ".to_string() + if c.allow_external_access { "translate-x-6" } else { "translate-x-0.5" }}></span>
                                        </button>
                                    </div>

                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                            API Key (optional)
                                        </label>
                                        <input
                                            type="password"
                                            placeholder="Set an API key for authentication"
                                            value={c.api_key.unwrap_or_default()}
                                            class="w-full px-4 py-2 border border-gray-300 rounded-lg"
                                            on:input={move |ev| {
                                                let key = event_target_value(&ev);
                                                let mut new_config = c.clone();
                                                new_config.api_key = if key.is_empty() { None } else { Some(key) };
                                                handle_update_config(new_config);
                                            }}
                                        />
                                    </div>
                                </div>
                            }
                        } else {
                            view! { <p class="text-gray-600">Loading configuration...</p> }
                        }
                    }}
                </div>

                <div class="bg-white rounded-lg shadow p-6 mt-6">
                    <h2 class="text-lg font-semibold mb-4">API Usage</h2>
                    <div class="bg-gray-50 p-4 rounded-lg">
                        <p class="text-sm font-medium mb-2">Example API Call:</p>
                        <pre class="bg-gray-800 text-green-400 p-4 rounded text-xs overflow-x-auto">
curl -X POST http://{local_ip}:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "model": "lf25-7b",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'</pre>
                    </div>

                    <div class="mt-4 text-sm text-gray-600">
                        <p class="font-medium mb-2">Supported Endpoints:</p>
                        <ul class="list-disc list-inside space-y-1">
                            <li><code class="bg-gray-100 px-1 rounded">POST /v1/chat/completions</code> - Chat completion</li>
                            <li><code class="bg-gray-100 px-1 rounded">GET /v1/models</code> - List available models</li>
                            <li><code class="bg-gray-100 px-1 rounded">GET /health</code> - Server health check</li>
                        </ul>
                    </div>
                </div>
            </main>

            <nav class="fixed bottom-0 left-0 right-0 bg-white border-t">
                <div class="flex justify-around py-2">
                    <a href="/local-models" class="flex flex-col items-center p-2 text-gray-600">
                        <span class="text-2xl">📥</span>
                        <span class="text-xs">Downloads</span>
                    </a>
                    <a href="/api-server" class="flex flex-col items-center p-2 text-blue-600">
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
