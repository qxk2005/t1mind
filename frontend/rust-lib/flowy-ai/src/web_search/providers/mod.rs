pub mod tavily;
pub mod brave;

pub use tavily::{
    TavilySearchProvider, create_tavily_provider,
};

pub use brave::{
    BraveSearchProvider, create_brave_search_provider,
};

/// WebSearchProvider trait 定义
#[async_trait::async_trait]
pub trait WebSearchProvider: Send + Sync {
    /// 执行搜索
    async fn search(&self, request: crate::web_search::entities::WebSearchRequestPB) -> flowy_error::FlowyResult<crate::web_search::entities::WebSearchResponsePB>;
    
    /// 运行测试
    async fn run_tests(&self) -> flowy_error::FlowyResult<crate::web_search::entities::TestWebSearchProviderResponsePB>;
    
    /// 获取供应商配置
    fn get_config(&self) -> &crate::web_search::entities::WebSearchProviderConfigPB;
    
    /// 检查供应商是否可用
    fn is_available(&self) -> bool;
}
