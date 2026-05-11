use leptos::*;

#[component]
pub fn WalletConnect(props: WalletConnectProps) -> impl IntoView {
    let (address, set_address) = create_signal(String::new());
    let (connecting, set_connecting) = create_signal(false);

    let handle_connect = move || {
        let addr = address();
        if addr.is_empty() {
            let sample_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f1E2dB";
            set_address(sample_address.to_string());
        }
        if web_sys::window().unwrap().local_storage().unwrap().is_some() {
            props.on_connect(address());
        }
    };

    view! {
        <div class="space-y-4">
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">
                    Wallet Address
                </label>
                <input
                    type="text"
                    placeholder="0x..."
                    value={address()}
                    on:input={move |ev| set_address(event_target_value(&ev))}
                    class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
            </div>

            <div class="text-center text-sm text-gray-500">
                <p class="mb-2">Don't have a wallet?</p>
                <div class="flex justify-center gap-2">
                    <a href="#" class="text-blue-600 hover:underline">MetaMask</a>
                    <span>|</span>
                    <a href="#" class="text-blue-600 hover:underline">WalletConnect</a>
                    <span>|</span>
                    <a href="#" class="text-blue-600 hover:underline">Coinbase</a>
                </div>
            </div>

            <button
                class="w-full py-3 px-4 bg-blue-600 text-white font-semibold rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
                disabled={connecting()}
                on:click={move |_| handle_connect()}
            >
                {if connecting() { "Connecting..." } else { "Connect Wallet" }}
            </button>
        </div>
    }
}

#[derive(Clone)]
pub struct WalletConnectProps {
    pub on_connect: Box<dyn Fn(String)>,
}
