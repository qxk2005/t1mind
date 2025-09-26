#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

int64_t init_sdk(int64_t port, char *data);

void async_event(int64_t port, const uint8_t *input, uintptr_t len);

const uint8_t *sync_event(const uint8_t *input, uintptr_t len);

int32_t set_stream_port(int64_t port);

int32_t set_log_stream_port(int64_t port);

void link_me_please(void);

void rust_log(int64_t level, const char *data);

void set_env(const char *data);

// Task Orchestration FFI Functions
const uint8_t *task_create_plan(const char *user_query, const char *session_id, const char *agent_id);
const uint8_t *task_confirm_plan(const char *plan_id);
const uint8_t *task_execute_plan(const char *plan_id, const char *context_json);
const uint8_t *task_cancel_execution(const char *plan_id);
const uint8_t *task_get_plan(const char *plan_id);
const uint8_t *task_get_active_plans(void);

// Agent Configuration FFI Functions
const uint8_t *agent_add_config(const char *config_json);
const uint8_t *agent_get_config(const char *agent_id);

// Execution Log FFI Functions
const uint8_t *execution_log_create(const char *session_id, const char *user_query, const char *task_plan_id, const char *agent_id, const char *user_id, const char *workspace_id);
const uint8_t *execution_log_search(const char *criteria_json);
const uint8_t *execution_log_get_details(const char *execution_id);
const uint8_t *execution_log_export(const char *criteria_json, const char *options_json);
const uint8_t *execution_log_get_statistics(int64_t start_time, int64_t end_time, const char *workspace_id);

// MCP Functions (existing)
const uint8_t *mcp_check_streamable_http(const char *url, const char *headers_json);
const uint8_t *mcp_check_sse(const char *url, const char *headers_json);
int32_t mcp_connect_sse(const char *id, const char *url, const char *headers_json);
int32_t mcp_disconnect_sse(const char *id);
const uint8_t *mcp_check_stdio(const char *command, const char *args_json, const char *env_json);

// Event Notification FFI Functions
int32_t task_set_progress_port(int64_t port);
int32_t task_send_notification(int64_t port, const char *event_type, const char *data_json);

// Memory Management
void free_bytes(uint8_t *ptr, uint32_t len);
