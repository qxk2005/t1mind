// Re-export web search entities from main entities module
// This ensures all web search related types use the ProtoBuf-enabled versions

pub use crate::entities::{
    // Provider related
    WebSearchProviderConfigPB,
    WebSearchProviderListPB,
    CreateWebSearchProviderRequestPB,
    UpdateWebSearchProviderRequestPB,
    DeleteWebSearchProviderRequestPB,
    GetWebSearchProviderRequestPB,
    TestWebSearchProviderRequestPB,
    TestWebSearchProviderResponsePB,
    WebSearchTestResultPB,
    WebSearchProviderTypePB,
    ProviderTestStatusPB,
    
    // Search request/response
    WebSearchRequestPB,
    WebSearchResponsePB,
    WebSearchResultPB,
    
    // Global config
    WebSearchGlobalConfigPB,
    UpdateWebSearchGlobalConfigRequestPB,
    
    // Cache related
    WebSearchCacheEntryPB,
    WebSearchCacheStatsPB,
    
    // Event related
    WebSearchEventPB,
    WebSearchEventTypePB,
};
