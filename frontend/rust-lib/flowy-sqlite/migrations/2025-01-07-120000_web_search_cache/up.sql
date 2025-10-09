-- Create table for web search cache entries
CREATE TABLE web_search_cache_table (
    cache_key TEXT PRIMARY KEY NOT NULL,
    query TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    search_response TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    hit_count BIGINT NOT NULL DEFAULT 0,
    metadata TEXT
);

-- Create indexes for efficient querying
CREATE INDEX idx_web_search_cache_provider_id ON web_search_cache_table (provider_id);
CREATE INDEX idx_web_search_cache_expires_at ON web_search_cache_table (expires_at);
CREATE INDEX idx_web_search_cache_created_at ON web_search_cache_table (created_at);
CREATE INDEX idx_web_search_cache_hit_count ON web_search_cache_table (hit_count);

-- Create table for web search cache statistics
CREATE TABLE web_search_cache_stats_table (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stat_name TEXT NOT NULL UNIQUE,
    stat_value TEXT NOT NULL,
    updated_at BIGINT NOT NULL
);

-- Insert initial cache statistics
INSERT INTO web_search_cache_stats_table (stat_name, stat_value, updated_at) VALUES
('total_hits', '0', strftime('%s', 'now')),
('total_misses', '0', strftime('%s', 'now')),
('total_entries', '0', strftime('%s', 'now')),
('last_cleanup', '0', strftime('%s', 'now'));
