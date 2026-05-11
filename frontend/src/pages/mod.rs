pub mod login;
pub mod models;
pub mod model_detail;
pub mod payment;
pub mod orders;
pub mod profile;
pub mod local_models;
pub mod api_server;

use leptos::*;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-100 pb-20">
            <header class="bg-white shadow">
                <div class="max-w-7xl mx-auto py-6 px-4">
                    <h1 class="text-3xl font-bold text-gray-900">Williw</h1>
                    <p class="text-gray-600 mt-1">AI Compute Power on Your Phone</p>
                </div>
            </header>
            <main class="max-w-7xl mx-auto py-6 px-4">
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                    <a href="/local-models" class="block p-6 bg-white rounded-lg shadow hover:shadow-lg transition">
                        <h2 class="text-xl font-semibold mb-2">📥 Local Models</h2>
                        <p class="text-gray-600">Download AI models to your phone</p>
                    </a>
                    <a href="/api-server" class="block p-6 bg-white rounded-lg shadow hover:shadow-lg transition">
                        <h2 class="text-xl font-semibold mb-2">🔌 API Server</h2>
                        <p class="text-gray-600">Share models with other devices</p>
                    </a>
                    <a href="/models" class="block p-6 bg-white rounded-lg shadow hover:shadow-lg transition">
                        <h2 class="text-xl font-semibold mb-2">☁️ Cloud Models</h2>
                        <p class="text-gray-600">Access remote compute power</p>
                    </a>
                    <a href="/orders" class="block p-6 bg-white rounded-lg shadow hover:shadow-lg transition">
                        <h2 class="text-xl font-semibold mb-2">📋 My Orders</h2>
                        <p class="text-gray-600">View your compute requests</p>
                    </a>
                </div>

                <div class="mt-8 bg-gradient-to-r from-blue-500 to-purple-600 rounded-lg shadow-lg p-8 text-white">
                    <h2 class="text-2xl font-bold mb-2">Run AI Anywhere</h2>
                    <p class="text-blue-100 mb-4">
                        Download models to your phone and serve AI inference to any device on your network.
                    </p>
                    <div class="flex gap-4">
                        <a href="/local-models" class="px-6 py-2 bg-white text-blue-600 rounded-lg font-medium hover:bg-blue-50">
                            Get Started
                        </a>
                        <a href="/api-server" class="px-6 py-2 border border-white text-white rounded-lg font-medium hover:bg-white/10">
                            Learn More
                        </a>
                    </div>
                </div>
            </main>

            <nav class="fixed bottom-0 left-0 right-0 bg-white border-t">
                <div class="flex justify-around py-2">
                    <a href="/" class="flex flex-col items-center p-2 text-blue-600">
                        <span class="text-2xl">🏠</span>
                        <span class="text-xs">Home</span>
                    </a>
                    <a href="/local-models" class="flex flex-col items-center p-2 text-gray-600">
                        <span class="text-2xl">📥</span>
                        <span class="text-xs">Local</span>
                    </a>
                    <a href="/api-server" class="flex flex-col items-center p-2 text-gray-600">
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
