-- Create table for execution logs
CREATE TABLE execution_log_table
(
    id                    TEXT PRIMARY KEY NOT NULL,
    session_id            TEXT             NOT NULL,
    task_plan_id          TEXT,
    user_query            TEXT             NOT NULL,
    start_time            BIGINT           NOT NULL,
    end_time              BIGINT,
    status                INTEGER          NOT NULL DEFAULT 0,
    error_message         TEXT,
    error_type            INTEGER,
    agent_id              TEXT,
    user_id               TEXT,
    workspace_id          TEXT,
    total_steps           INTEGER          NOT NULL DEFAULT 0,
    completed_steps       INTEGER          NOT NULL DEFAULT 0,
    failed_steps          INTEGER          NOT NULL DEFAULT 0,
    skipped_steps         INTEGER          NOT NULL DEFAULT 0,
    context               TEXT             NOT NULL DEFAULT '{}',
    result_summary        TEXT,
    used_mcp_tools        TEXT             NOT NULL DEFAULT '[]',
    tags                  TEXT             NOT NULL DEFAULT '[]',
    retry_count           INTEGER          NOT NULL DEFAULT 0,
    max_retries           INTEGER          NOT NULL DEFAULT 3,
    parent_execution_id   TEXT,
    child_execution_ids   TEXT             NOT NULL DEFAULT '[]',
    created_at            BIGINT           NOT NULL,
    updated_at            BIGINT           NOT NULL
);

-- Create table for execution steps
CREATE TABLE execution_step_table
(
    id                    TEXT PRIMARY KEY NOT NULL,
    execution_log_id      TEXT             NOT NULL,
    name                  TEXT             NOT NULL,
    description           TEXT             NOT NULL,
    mcp_tool_id           TEXT             NOT NULL,
    mcp_tool_name         TEXT             NOT NULL,
    mcp_tool_config       TEXT             NOT NULL DEFAULT '{}',
    input_parameters      TEXT             NOT NULL DEFAULT '{}',
    output_result         TEXT,
    execution_time_ms     INTEGER          NOT NULL DEFAULT 0,
    status                INTEGER          NOT NULL DEFAULT 0,
    start_time            BIGINT,
    end_time              BIGINT,
    error_message         TEXT,
    error_type            INTEGER,
    error_stack           TEXT,
    step_order            INTEGER          NOT NULL DEFAULT 0,
    retry_count           INTEGER          NOT NULL DEFAULT 0,
    max_retries           INTEGER          NOT NULL DEFAULT 3,
    dependencies          TEXT             NOT NULL DEFAULT '[]',
    tags                  TEXT             NOT NULL DEFAULT '[]',
    metadata              TEXT             NOT NULL DEFAULT '{}',
    can_skip              BOOLEAN          NOT NULL DEFAULT FALSE,
    is_critical           BOOLEAN          NOT NULL DEFAULT FALSE,
    created_at            BIGINT           NOT NULL,
    updated_at            BIGINT           NOT NULL,
    FOREIGN KEY (execution_log_id) REFERENCES execution_log_table (id) ON DELETE CASCADE
);

-- Create table for execution references
CREATE TABLE execution_reference_table
(
    id                    TEXT PRIMARY KEY NOT NULL,
    execution_step_id     TEXT             NOT NULL,
    reference_type        INTEGER          NOT NULL,
    title                 TEXT             NOT NULL,
    content               TEXT,
    url                   TEXT,
    source                TEXT,
    timestamp             BIGINT           NOT NULL,
    metadata              TEXT             NOT NULL DEFAULT '{}',
    relevance_score       REAL             NOT NULL DEFAULT 0.0,
    created_at            BIGINT           NOT NULL,
    FOREIGN KEY (execution_step_id) REFERENCES execution_step_table (id) ON DELETE CASCADE
);

-- Create table for MCP tool info
CREATE TABLE mcp_tool_info_table
(
    id                        TEXT PRIMARY KEY NOT NULL,
    name                      TEXT             NOT NULL,
    display_name              TEXT,
    description               TEXT             NOT NULL DEFAULT '',
    version                   TEXT             NOT NULL DEFAULT '',
    provider                  TEXT             NOT NULL DEFAULT '',
    category                  TEXT             NOT NULL DEFAULT '',
    status                    INTEGER          NOT NULL DEFAULT 0,
    config                    TEXT             NOT NULL DEFAULT '{}',
    schema                    TEXT             NOT NULL DEFAULT '{}',
    requires_auth             BOOLEAN          NOT NULL DEFAULT FALSE,
    auth_config               TEXT,
    icon_url                  TEXT,
    documentation_url         TEXT,
    last_checked              BIGINT,
    last_used                 BIGINT,
    usage_count               INTEGER          NOT NULL DEFAULT 0,
    success_count             INTEGER          NOT NULL DEFAULT 0,
    failure_count             INTEGER          NOT NULL DEFAULT 0,
    average_execution_time_ms INTEGER          NOT NULL DEFAULT 0,
    created_at                BIGINT           NOT NULL,
    updated_at                BIGINT           NOT NULL
);

-- Create indexes for better query performance
CREATE INDEX idx_execution_log_session_id ON execution_log_table (session_id);
CREATE INDEX idx_execution_log_status ON execution_log_table (status);
CREATE INDEX idx_execution_log_start_time ON execution_log_table (start_time);
CREATE INDEX idx_execution_log_user_id ON execution_log_table (user_id);
CREATE INDEX idx_execution_log_workspace_id ON execution_log_table (workspace_id);
CREATE INDEX idx_execution_log_agent_id ON execution_log_table (agent_id);

CREATE INDEX idx_execution_step_log_id ON execution_step_table (execution_log_id);
CREATE INDEX idx_execution_step_status ON execution_step_table (status);
CREATE INDEX idx_execution_step_order ON execution_step_table (step_order);
CREATE INDEX idx_execution_step_mcp_tool ON execution_step_table (mcp_tool_name);

CREATE INDEX idx_execution_reference_step_id ON execution_reference_table (execution_step_id);
CREATE INDEX idx_execution_reference_type ON execution_reference_table (reference_type);

CREATE INDEX idx_mcp_tool_name ON mcp_tool_info_table (name);
CREATE INDEX idx_mcp_tool_status ON mcp_tool_info_table (status);
CREATE INDEX idx_mcp_tool_category ON mcp_tool_info_table (category);
