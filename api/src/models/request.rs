use williw_shared::*;

#[derive(serde::Deserialize)]
pub struct ComputeRequestCreate {
    pub model_id: Uuid,
    pub amount: f64,
}

#[derive(serde::Serialize)]
pub struct ComputeRequestResponse {
    pub request: ComputeRequest,
    pub model: AiModel,
}

#[derive(serde::Deserialize)]
pub struct ModelFilterParams {
    pub category: Option<String>,
    pub provider: Option<String>,
    pub min_power: Option<f64>,
    pub max_price: Option<f64>,
    pub search: Option<String>,
}

impl From<ModelFilterParams> for ModelFilter {
    fn from(params: ModelFilterParams) -> Self {
        let category = params.category.and_then(|c| {
            serde_json::from_str(&format!("\"{}\"", c)).ok()
        });
        ModelFilter {
            category,
            provider: params.provider,
            min_power: params.min_power,
            max_price: params.max_price,
            search: params.search,
        }
    }
}
