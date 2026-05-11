use leptos::*;
use williw_shared::AiModel;

#[component]
pub fn ModelCard(props: ModelCardProps) -> impl IntoView {
    let model = props.model;
    let category = format!("{:?}", model.category).to_lowercase();

    let category_icon = move || -> String {
        match model.category {
            williw_shared::ModelCategory::Llm => "💬".to_string(),
            williw_shared::ModelCategory::Image => "🎨".to_string(),
            williw_shared::ModelCategory::Audio => "🎧".to_string(),
            williw_shared::ModelCategory::Video => "🎬".to_string(),
            williw_shared::ModelCategory::Multimodal => "🔮".to_string(),
        }
    };

    view! {
        <a
            href={format!("/models/{}", model.id)}
            class="block bg-white rounded-lg shadow hover:shadow-lg transition overflow-hidden"
        >
            <div class="p-6">
                <div class="flex justify-between items-start mb-4">
                    <span class="text-4xl">{category_icon()}</span>
                    <span class="px-2 py-1 bg-gray-100 rounded text-xs">{category}</span>
                </div>
                <h3 class="text-xl font-semibold text-gray-900 mb-1">{&model.name}</h3>
                <p class="text-sm text-gray-600 mb-4">{&model.provider}</p>
                <p class="text-gray-500 text-sm line-clamp-2 mb-4">{&model.description}</p>
                <div class="flex justify-between items-center">
                    <div>
                        <p class="text-xs text-gray-500">Power</p>
                        <p class="font-semibold">{model.compute_power} TFLOPS</p>
                    </div>
                    <div class="text-right">
                        <p class="text-xs text-gray-500">Price</p>
                        <p class="font-semibold">${model.price_per_unit}</p>
                    </div>
                </div>
            </div>
        </a>
    }
}

#[derive(Clone)]
pub struct ModelCardProps {
    pub model: AiModel,
}
