use leptos::view;
use williw_frontend::App;

fn main() {
    _ = console_error_panic_hook::set_hook;
    mount_to_body(|| view! { <App /> });
}
