pub mod login;
pub mod models;
pub mod model_detail;
pub mod payment;
pub mod orders;
pub mod profile;

use leptos::*;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-100">
            <header class="bg-white shadow">
                <div class="max-w-7xl mx-auto py-6 px-4">
                    <h1 class="text-3xl font-bold text-gray-900">Williw</h1>
                </div>
            </header>
            <main class="max-w-7xl mx-auto py-6 px-4">
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                    <a href="/models" class="block p-6 bg-white rounded-lg shadow hover:shadow-lg transition">
                        <h2 class="text-xl font-semibold mb-2">AI Models</h2>
                        <p class="text-gray-600">Browse and request compute power</p>
                    </a>
                    <a href="/orders" class="block p-6 bg-white rounded-lg shadow hover:shadow-lg transition">
                        <h2 class="text-xl font-semibold mb-2">My Orders</h2>
                        <p class="text-gray-600">View your compute requests</p>
                    </a>
                    <a href="/profile" class="block p-6 bg-white rounded-lg shadow hover:shadow-lg transition">
                        <h2 class="text-xl font-semibold mb-2">Profile</h2>
                        <p class="text-gray-600">Manage your wallet and settings</p>
                    </a>
                    <a href="/login" class="block p-6 bg-blue-500 text-white rounded-lg shadow hover:shadow-lg transition">
                        <h2 class="text-xl font-semibold mb-2">Connect Wallet</h2>
                        <p class="text-blue-100">Login with crypto wallet</p>
                    </a>
                </div>
            </main>
        </div>
    }
}
