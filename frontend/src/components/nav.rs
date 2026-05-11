/// 导航栏组件

use leptos::*;

/// 顶部导航栏组件
#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="bg-white shadow">
            <div class="max-w-7xl mx-auto px-4 py-4 flex justify-between items-center">
                <a href="/" class="text-2xl font-bold text-gray-900">Williw</a>
                <div class="flex gap-4">
                    <a href="/models" class="text-gray-600 hover:text-gray-900">Models</a>
                    <a href="/orders" class="text-gray-600 hover:text-gray-900">Orders</a>
                    <a href="/profile" class="text-gray-600 hover:text-gray-900">Profile</a>
                </div>
            </div>
        </nav>
    }
}

/// 移动端底部导航栏组件
#[component]
pub fn MobileNav() -> impl IntoView {
    view! {
        <nav class="fixed bottom-0 left-0 right-0 bg-white border-t">
            <div class="flex justify-around py-2">
                <a href="/models" class="flex flex-col items-center p-2">
                    <span class="text-2xl">🤖</span>
                    <span class="text-xs">Models</span>
                </a>
                <a href="/orders" class="flex flex-col items-center p-2">
                    <span class="text-2xl">📋</span>
                    <span class="text-xs">Orders</span>
                </a>
                <a href="/profile" class="flex flex-col items-center p-2">
                    <span class="text-2xl">👤</span>
                    <span class="text-xs">Profile</span>
                </a>
            </div>
        </nav>
    }
}