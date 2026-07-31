// OpenAI 호환 프로바이더 re-exports
// openai.rs에 구현된 OpenAiCompatibleProvider를 각 프로바이더별로 래핑

pub use crate::openai::OpenAiCompatibleProvider;

pub struct OpenAiProvider;
impl OpenAiProvider {
    pub fn new(api_key: String) -> OpenAiCompatibleProvider {
        OpenAiCompatibleProvider::openai(api_key)
    }
}

pub struct OpenRouterProvider;
impl OpenRouterProvider {
    pub fn new(api_key: String) -> OpenAiCompatibleProvider {
        OpenAiCompatibleProvider::openrouter(api_key)
    }
}

pub struct XaiProvider;
impl XaiProvider {
    pub fn new(api_key: String) -> OpenAiCompatibleProvider {
        OpenAiCompatibleProvider::xai(api_key)
    }
}