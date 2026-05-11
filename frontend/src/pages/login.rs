use leptos::*;
use crate::api::ApiClient;
use crate::components::WalletConnect;

#[component]
pub fn Login() -> impl IntoView {
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);

    let handle_wallet_connect = move |wallet_address: String| {
        set_loading(true);
        set_error(None);

        spawn(async move {
            let message = format!("Sign this message to login to Williw: {}", chrono::Utc::now().timestamp());
            let signature = format!("0x dummy_signature_for_{}", wallet_address);

            let client = ApiClient::new();
            match client.wallet_login(&wallet_address, &signature, &message).await {
                Ok(response) => {
                    let mut client = client;
                    client.set_token(response.token.clone());
                    let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
                    storage.set_item("auth_token", &response.token).ok();
                    storage.set_item("wallet_address", &response.user.wallet_address).ok();
                    web_sys::window().unwrap().location().set_href("/").ok();
                }
                Err(e) => {
                    set_error(Some(e));
                    set_loading(false);
                }
            }
        });
    };

    view! {
        <div class="min-h-screen bg-gray-100 flex items-center justify-center px-4">
            <div class="max-w-md w-full bg-white rounded-lg shadow-lg p-8">
                <div class="text-center mb-8">
                    <h1 class="text-3xl font-bold text-gray-900">Welcome to Williw</h1>
                    <p class="text-gray-600 mt-2">Connect your crypto wallet to get started</p>
                </div>

                <WalletConnect on_connect={handle_wallet_connect} />

                {move || {
                    if loading() {
                        view! { <p class="text-center text-gray-600 mt-4">Connecting...</p> }
                    } else {
                        view! { <></> }
                    }
                }}

                {move || {
                    if let Some(err) = error() {
                        view! {
                            <p class="text-red-500 text-center mt-4">{err}</p>
                        }
                    } else {
                        view! { <></> }
                    }
                }}

                <div class="mt-8 text-center text-sm text-gray-500">
                    <p>By connecting, you agree to our Terms of Service</p>
                </div>
            </div>
        </div>
    }
}
