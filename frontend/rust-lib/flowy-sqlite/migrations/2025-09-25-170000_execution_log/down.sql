-- Drop indexes
DROP INDEX IF EXISTS idx_mcp_tool_category;
DROP INDEX IF EXISTS idx_mcp_tool_status;
DROP INDEX IF EXISTS idx_mcp_tool_name;

DROP INDEX IF EXISTS idx_execution_reference_type;
DROP INDEX IF EXISTS idx_execution_reference_step_id;

DROP INDEX IF EXISTS idx_execution_step_mcp_tool;
DROP INDEX IF EXISTS idx_execution_step_order;
DROP INDEX IF EXISTS idx_execution_step_status;
DROP INDEX IF EXISTS idx_execution_step_log_id;

DROP INDEX IF EXISTS idx_execution_log_agent_id;
DROP INDEX IF EXISTS idx_execution_log_workspace_id;
DROP INDEX IF EXISTS idx_execution_log_user_id;
DROP INDEX IF EXISTS idx_execution_log_start_time;
DROP INDEX IF EXISTS idx_execution_log_status;
DROP INDEX IF EXISTS idx_execution_log_session_id;

-- Drop tables
DROP TABLE IF EXISTS mcp_tool_info_table;
DROP TABLE IF EXISTS execution_reference_table;
DROP TABLE IF EXISTS execution_step_table;
DROP TABLE IF EXISTS execution_log_table;
