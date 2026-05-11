use leptos::*;
use crate::api::ApiClient;

#[component]
pub fn Profile() -> impl IntoView {
    let (profile, set_profile) = create_signal(Option::<crate::api::ProfileResponse>::None);
    let (loading, set_loading) = create_signal(true);

    on_mount(move || {
        spawn(async move {
            let client = ApiClient::new();
            match client.get_profile().await {
                Ok(p) => set_profile(Some(p)),
                Err(_) => {}
            }
            set_loading(false);
        });
    });

    let handle_disconnect = move || {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        storage.remove_item("auth_token").ok();
        storage.remove_item("wallet_address").ok();
        web_sys::window().unwrap().location().set_href("/login").ok();
    };

    view! {
        <div class="min-h-screen bg-gray-100">
            <nav class="bg-white shadow">
                <div class="max-w-7xl mx-auto px-4 py-4 flex justify-between items-center">
                    <a href="/" class="text-2xl font-bold text-gray-900">Williw</a>
                    <div class="flex gap-4">
                        <a href="/models" class="text-gray-600">Models</a>
                        <a href="/orders" class="text-gray-600">Orders</a>
                        <a href="/profile" class="text-blue-600 font-medium">Profile</a>
                    </div>
                </div>
            </nav>

            <main class="max-w-2xl mx-auto px-4 py-6">
                <h1 class="text-2xl font-bold mb-6">Profile</h1>

                {move || {
                    if loading() {
                        view! { <p class="text-center">Loading...</p> }
                    } else if let Some(p) = profile() {
                        view! {
                            <div class="space-y-6">
                                <div class="bg-white rounded-lg shadow p-6">
                                    <h2 class="text-lg font-semibold mb-4">Wallet</h2>
                                    <div class="bg-gray-100 p-4 rounded-lg">
                                        <p class="text-sm text-gray-600">Connected Address</p>
                                        <p class="font-mono text-sm break-all">{&p.user.wallet_address}</p>
                                    </div>
                                    <div class="mt-4 grid grid-cols-2 gap-4">
                                        <div class="bg-gray-50 p-4 rounded-lg">
                                            <p class="text-sm text-gray-600">Balance</p>
                                            <p class="text-2xl font-bold">${p.user.balance}</p>
                                        </div>
                                        <div class="bg-gray-50 p-4 rounded-lg">
                                            <p class="text-sm text-gray-600">Total Spent</p>
                                            <p class="text-2xl font-bold">${p.total_spent}</p>
                                        </div>
                                    </div>
                                </div>

                                <div class="bg-white rounded-lg shadow p-6">
                                    <h2 class="text-lg font-semibold mb-4">Statistics</h2>
                                    <div class="grid grid-cols-2 gap-4">
                                        <div class="bg-gray-50 p-4 rounded-lg">
                                            <p class="text-sm text-gray-600">Total Orders</p>
                                            <p class="text-2xl font-bold">{p.total_orders}</p>
                                        </div>
                                        <div class="bg-gray-50 p-4 rounded-lg">
                                            <p class="text-sm text-gray-600">Member Since</p>
                                            <p class="text-lg font-medium">{p.user.created_at.format("%Y-%m-%d")}</p>
                                        </div>
                                    </div>
                                </div>

                                <div class="bg-white rounded-lg shadow p-6">
                                    <h2 class="text-lg font-semibold mb-4">Settings</h2>
                                    <div class="space-y-3">
                                        <button class="w-full text-left px-4 py-3 bg-gray-50 rounded-lg hover:bg-gray-100">
                                            Edit Profile
                                        </button>
                                        <button class="w-full text-left px-4 py-3 bg-gray-50 rounded-lg hover:bg-gray-100">
                                            Notification Settings
                                        </button>
                                        <button class="w-full text-left px-4 py-3 bg-gray-50 rounded-lg hover:bg-gray-100">
                                            API Keys
                                        </button>
                                        <button
                                            class="w-full text-left px-4 py-3 bg-red-50 text-red-600 rounded-lg hover:bg-red-100"
                                            on:click={move |_| handle_disconnect()}
                                        >
                                            Disconnect Wallet
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    } else {
                        view! {
                            <div class="bg-white rounded-lg shadow p-8 text-center">
                                <p class="text-gray-600 mb-4">Please connect your wallet</p>
                                <a href="/login" class="text-blue-600 hover:underline">Connect Wallet</a>
                            </div>
                        }
                    }
                }}
            </main>
        </div>
    }
}
