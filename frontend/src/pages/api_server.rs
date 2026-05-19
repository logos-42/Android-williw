/// API服务器页面组件
use leptos::*;
use crate::api::ApiClient;
use williw_shared::{LocalApiConfig, P2pStatus, P2pConfig, P2pConnectionInfo};

/// API服务器页面组件
/// 管理本地API服务器和P2P连接
#[component]
pub fn ApiServer() -> impl IntoView {
    // 配置和状态
    let (config, set_config) = create_signal(Option::<LocalApiConfig>::None);
    let (p2p_config, set_p2p_config) = create_signal(Option::<P2pConfig>::None);
    let (p2p_status, set_p2p_status) = create_signal(Option::<P2pStatus>::None);
    let (p2p_info, set_p2p_info) = create_signal(Option::<P2pConnectionInfo>::None);
    let (device_info, set_device_info) = create_signal(Option::<williw_shared::DeviceInfo>::None);
    let (loading, set_loading) = create_signal(true);
    let (server_status, set_server_status) = create_signal(false);
    let (p2p_online, set_p2p_online) = create_signal(false);
    let (message, set_message) = create_signal(Option::<String>::None);
    let (connect_peer_id, set_connect_peer_id) = create_signal(String::new());
    let (share_code, set_share_code) = create_signal(String::new());

    /// 加载所有数据
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

            match client.get_p2p_status().await {
                Ok(status) => {
                    set_p2p_status(Some(status.clone()));
                    set_p2p_online(status.is_online);
                }
                Err(_) => {}
            }

            match client.get_p2p_config().await {
                Ok(cfg) => set_p2p_config(Some(cfg)),
                Err(_) => {}
            }

            set_loading(false);
        });
    };

    on_mount(move || {
        load_data();
    });

    /// 切换本地服务器开关
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

    /// 切换P2P开关
    let handle_toggle_p2p = move || {
        spawn(async move {
            let client = ApiClient::new();
            let result = if p2p_online() {
                client.p2p_go_offline().await
            } else {
                client.p2p_go_online().await
            };

            match result {
                Ok(msg) => {
                    set_message(Some(msg.clone()));
                    set_p2p_online(!p2p_online());
                    if !p2p_online() {
                        match client.get_p2p_connection_info().await {
                            Ok(info) => {
                                set_p2p_info(Some(info.clone()));
                                set_share_code(info.connection_code);
                            }
                            Err(_) => {}
                        }
                    }
                    load_data();
                }
                Err(e) => {
                    set_message(Some(format!("Error: {}", e)));
                }
            }
        });
    };

    /// 连接到对等节点
    let handle_connect_peer = move || {
        let peer_id = connect_peer_id();
        if peer_id.is_empty() { return; }

        spawn(async move {
            let client = ApiClient::new();
            match client.connect_to_peer(peer_id).await {
                Ok(response) => {
                    set_message(Some(format!("Connected! Tunnel endpoint: {}", response.tunnel_endpoint)));
                    set_connect_peer_id(String::new());
                }
                Err(e) => {
                    set_message(Some(format!("Connection failed: {}", e)));
                }
            }
        });
    };

    /// 更新服务器配置
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
                <h1 class="text-2xl font-bold mb-6">API Server & P2P</h1>

                // 消息提示
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

                // 本地服务器卡片
                <div class="bg-white rounded-lg shadow p-6 mb-6">
                    <h2 class="text-lg font-semibold mb-4">Local Server</h2>

                    <div class="flex items-center justify-between mb-4">
                        <div class="flex items-center gap-3">
                            <span class={"w-4 h-4 rounded-full ".to_string() + if server_status() { "bg-green-500" } else { "bg-gray-400" }}></span>
                            <span class="font-medium">
                                {if server_status() { "LAN Server Running" } else { "LAN Server Stopped" }}
                            </span>
                        </div>
                        <button
                            class={"px-6 py-2 rounded-lg font-medium text-white transition ".to_string() + if server_status() { "bg-red-600 hover:bg-red-700" } else { "bg-green-600 hover:bg-green-700" }}
                            on:click={move |_| handle_toggle_server()}
                        >
                            {if server_status() { "Stop" } else { "Start" }}
                        </button>
                    </div>

                    {if server_status() {
                        view! {
                            <div class="bg-gray-50 p-4 rounded-lg text-sm">
                                <p class="font-medium mb-2">Local Network Access:</p>
                                <code class="block bg-gray-200 p-2 rounded mb-1">http://localhost:{device_info().map(|i| i.server_port).unwrap_or(8081)}</code>
                                <code class="block bg-gray-200 p-2 rounded">http://{local_ip}:{device_info().map(|i| i.server_port).unwrap_or(8081)}</code>
                            </div>
                        }
                    } else {
                        view! {
                            <div class="bg-gray-50 p-4 rounded-lg">
                                <p class="text-sm text-gray-600">Start server for local network access</p>
                            </div>
                        }
                    }}
                </div>

                // P2P隧道卡片
                <div class="bg-white rounded-lg shadow p-6 mb-6">
                    <h2 class="text-lg font-semibold mb-4">🌐 P2P Tunnel (Internet Access)</h2>

                    <div class="flex items-center justify-between mb-4">
                        <div class="flex items-center gap-3">
                            <span class={"w-4 h-4 rounded-full ".to_string() + if p2p_online() { "bg-green-500 animate-pulse" } else { "bg-gray-400" }}></span>
                            <span class="font-medium">
                                {if p2p_online() { "P2P Online" } else { "P2P Offline" }}
                            </span>
                        </div>
                        <button
                            class={"px-6 py-2 rounded-lg font-medium text-white transition ".to_string() + if p2p_online() { "bg-red-600 hover:bg-red-700" } else { "bg-blue-600 hover:bg-blue-700" }}
                            on:click={move |_| handle_toggle_p2p()}
                        >
                            {if p2p_online() { "Go Offline" } else { "Go Online" }}
                        </button>
                    </div>

                    {if p2p_online() {
                        let info = p2p_info();
                        view! {
                            <div class="bg-green-50 border border-green-200 rounded-lg p-4 mb-4">
                                <p class="font-medium text-green-800 mb-2">Your Connection Code:</p>
                                <div class="bg-white p-3 rounded text-center">
                                    <span class="text-3xl font-mono font-bold tracking-wider">{share_code()}</span>
                                </div>
                                <p class="text-xs text-green-600 mt-2">
                                    Share this code with others so they can connect to your device
                                </p>
                            </div>
                        }
                    } else {
                        view! {
                            <div class="bg-gray-50 p-4 rounded-lg mb-4">
                                <p class="text-sm text-gray-600">
                                    Go online to allow anyone to connect to your models over the internet via P2P tunnel.
                                </p>
                            </div>
                        }
                    }}

                    <div class="border-t pt-4 mt-4">
                        <h3 class="font-medium mb-3">Connect to Another Device</h3>
                        <div class="flex gap-2">
                            <input
                                type="text"
                                placeholder="Enter peer connection code..."
                                value={connect_peer_id()}
                                on:input={move |ev| set_connect_peer_id(event_target_value(&ev))}
                                class="flex-1 px-4 py-2 border border-gray-300 rounded-lg"
                            />
                            <button
                                class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
                                disabled={connect_peer_id().is_empty() || !p2p_online()}
                                on:click={move |_| handle_connect_peer()}
                            >
                                Connect
                            </button>
                        </div>
                    </div>
                </div>

                // P2P配置卡片
                <div class="bg-white rounded-lg shadow p-6 mb-6">
                    <h2 class="text-lg font-semibold mb-4">P2P Configuration</h2>

                    {move || {
                        if let Some(cfg) = p2p_config() {
                            view! {
                                <div class="space-y-4">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                            Tunnel Mode
                                        </label>
                                        <select class="w-full px-4 py-2 border border-gray-300 rounded-lg">
                                            <option value="relay_fallback" selected={matches!(cfg.tunnel_mode, williw_shared::P2pTunnelMode::RelayFallback)}>
                                                STUN + Relay Fallback (Recommended)
                                            </option>
                                            <option value="stun_only" selected={matches!(cfg.tunnel_mode, williw_shared::P2pTunnelMode::StunOnly)}>
                                                STUN Only (May fail behind strict NAT)
                                            </option>
                                            <option value="relay_preferred" selected={matches!(cfg.tunnel_mode, williw_shared::P2pTunnelMode::RelayPreferred)}>
                                                Relay Preferred (Always works, uses relay server)
                                            </option>
                                        </select>
                                    </div>

                                    <div class="flex items-center justify-between">
                                        <div>
                                            <p class="font-medium">Enable P2P</p>
                                            <p class="text-sm text-gray-500">Allow peer-to-peer connections</p>
                                        </div>
                                        <span class={"px-3 py-1 rounded-full text-sm ".to_string() + if cfg.enabled { "bg-green-100 text-green-700" } else { "bg-gray-100 text-gray-700" }}>
                                            {if cfg.enabled { "Enabled" } else { "Disabled" }}
                                        </span>
                                    </div>

                                    {if let Some(peer_id) = &cfg.peer_id {
                                        view! {
                                            <div class="bg-gray-50 p-3 rounded-lg">
                                                <p class="text-sm text-gray-600">Your Peer ID:</p>
                                                <code class="font-mono text-sm">{peer_id}</code>
                                            </div>
                                        }
                                    } else {
                                        view! { <></> }
                                    }}
                                </div>
                            }
                        } else {
                            view! { <p class="text-gray-600">Loading P2P configuration...</p> }
                        }
                    }}
                </div>

                // 快速连接说明
                <div class="bg-white rounded-lg shadow p-6">
                    <h2 class="text-lg font-semibold mb-4">Quick Connect</h2>
                    <div class="bg-gray-50 p-4 rounded-lg">
                        <p class="text-sm font-medium mb-2">How P2P works:</p>
                        <ol class="text-sm text-gray-600 space-y-2 list-decimal list-inside">
                            <li>Your device goes "online" and gets a unique connection code</li>
                            <li>Share your code with friends who want to use your models</li>
                            <li>They enter your code to establish a direct P2P tunnel</li>
                            <li>They can now call your AI models over the internet!</li>
                        </ol>
                    </div>

                        <div class="mt-4 text-sm text-gray-600">
                            <p class="font-medium mb-2">Example API Call via P2P:</p>
                            <pre class="bg-gray-800 text-green-400 p-3 rounded text-xs overflow-x-auto">
curl -X POST https://p2p.williw.ai/{share_code()}/v1/chat/completions
  -H "Content-Type: application/json"
  -H "Authorization: Bearer YOUR_API_KEY"
  -d "{
    \"model\": \"lf25-7b\",
    \"messages\": [{\"role\": \"user\", \"content\": \"Hello!\"}]
  }"</pre>
                        </div>
                </div>
            </main>

            // 底部导航栏
            <nav class="fixed bottom-0 left-0 right-0 bg-white border-t">
                <div class="flex justify-around py-2">
                    <a href="/local-models" class="flex flex-col items-center p-2 text-gray-600">
                        <span class="text-2xl">📥</span>
                        <span class="text-xs">Downloads</span>
                    </a>
                    <a href="/api-server" class="flex flex-col items-center p-2 text-blue-600">
                        <span class="text-2xl">🔌</span>
                        <span class="text-xs">API</span>
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