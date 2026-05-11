/// Williw前端库
/// 包含API客户端、页面组件和UI组件

pub mod api;
pub mod pages;
pub mod components;

use leptos::*;

/// 应用根组件
/// 定义所有路由配置
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                /// 首页
                <Route path="/" component=pages::Home />
                /// 登录页
                <Route path="/login" component=pages::Login />
                /// 模型列表页
                <Route path="/models" component=pages::Models />
                /// 模型详情页
                <Route path="/models/:id" component=pages::ModelDetail />
                /// 本地模型页
                <Route path="/local-models" component=pages::LocalModels />
                /// API服务器页
                <Route path="/api-server" component=pages::ApiServer />
                /// 支付页
                <Route path="/payment/:order_id" component=pages::Payment />
                /// 订单页
                <Route path="/orders" component=pages::Orders />
                /// 个人资料页
                <Route path="/profile" component=pages::Profile />
            </Routes>
        </Router>
    }
}