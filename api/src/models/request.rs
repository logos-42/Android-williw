use williw_shared::*;
use uuid::Uuid;

/// 创建计算请求的请求结构
#[derive(serde::Deserialize)]
pub struct ComputeRequestCreate {
    /// 要调用的模型ID
    pub model_id: Uuid,
    /// 计算资源数量
    pub amount: f64,
}

/// 计算请求的响应结构，包含请求详情和模型信息
#[derive(serde::Serialize)]
pub struct ComputeRequestResponse {
    /// 计算请求详情
    pub request: ComputeRequest,
    /// 关联的AI模型信息
    pub model: AiModel,
}

/// 模型过滤查询参数
#[derive(serde::Deserialize)]
pub struct ModelFilterParams {
    /// 模型类别（如llm、image、audio、multimodal）
    pub category: Option<String>,
    /// 提供商名称
    pub provider: Option<String>,
    /// 最小算力阈值
    pub min_power: Option<f64>,
    /// 最大单价阈值
    pub max_price: Option<f64>,
    /// 名称或描述的关键词搜索
    pub search: Option<String>,
}

/// 将URL查询参数转换为ModelFilter结构
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