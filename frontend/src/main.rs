use leptos::view;
use williw_frontend::App;

/// 主入口函数
/// 挂载应用到body
fn main() {
    // 设置panic钩子用于调试
    _ = console_error_panic_hook::set_hook;
    // 将App组件挂载到body
    mount_to_body(|| view! { <App /> });
}