// @generated automatically by Diesel CLI.

diesel::table! {
    af_collab_metadata (object_id) {
        object_id -> Text,
        updated_at -> BigInt,
        prev_sync_state_vector -> Binary,
        collab_type -> Integer,
    }
}

diesel::table! {
    chat_local_setting_table (chat_id) {
        chat_id -> Text,
        local_model_path -> Text,
        local_model_name -> Text,
    }
}

diesel::table! {
    chat_message_table (message_id) {
        message_id -> BigInt,
        chat_id -> Text,
        content -> Text,
        created_at -> BigInt,
        author_type -> BigInt,
        author_id -> Text,
        reply_message_id -> Nullable<BigInt>,
        metadata -> Nullable<Text>,
        is_sync -> Bool,
    }
}

diesel::table! {
    chat_table (chat_id) {
        chat_id -> Text,
        created_at -> BigInt,
        metadata -> Text,
        rag_ids -> Nullable<Text>,
        is_sync -> Bool,
        summary -> Text,
    }
}

diesel::table! {
    collab_snapshot (id) {
        id -> Text,
        object_id -> Text,
        title -> Text,
        desc -> Text,
        collab_type -> Text,
        timestamp -> BigInt,
        data -> Binary,
    }
}

diesel::table! {
    index_collab_record_table (oid) {
        oid -> Text,
        workspace_id -> Text,
        content_hash -> Text,
    }
}

diesel::table! {
    local_ai_model_table (name) {
        name -> Text,
        model_type -> SmallInt,
    }
}

diesel::table! {
    upload_file_part (upload_id, e_tag) {
        upload_id -> Text,
        e_tag -> Text,
        part_num -> Integer,
    }
}

diesel::table! {
    upload_file_table (workspace_id, file_id, parent_dir) {
        workspace_id -> Text,
        file_id -> Text,
        parent_dir -> Text,
        local_file_path -> Text,
        content_type -> Text,
        chunk_size -> Integer,
        num_chunk -> Integer,
        upload_id -> Text,
        created_at -> BigInt,
        is_finish -> Bool,
    }
}

diesel::table! {
    user_data_migration_records (id) {
        id -> Integer,
        migration_name -> Text,
        executed_at -> Timestamp,
    }
}

diesel::table! {
    user_table (id) {
        id -> Text,
        name -> Text,
        icon_url -> Text,
        token -> Text,
        email -> Text,
        auth_type -> Integer,
        updated_at -> BigInt,
    }
}

diesel::table! {
    user_workspace_table (id) {
        id -> Text,
        name -> Text,
        uid -> BigInt,
        created_at -> BigInt,
        database_storage_id -> Text,
        icon -> Text,
        member_count -> BigInt,
        role -> Nullable<Integer>,
        workspace_type -> Integer,
    }
}

diesel::table! {
    workspace_members_table (email, workspace_id) {
        email -> Text,
        role -> Integer,
        name -> Text,
        avatar_url -> Nullable<Text>,
        uid -> BigInt,
        workspace_id -> Text,
        updated_at -> Timestamp,
        joined_at -> Nullable<BigInt>,
    }
}

diesel::table! {
    workspace_setting_table (id) {
        id -> Text,
        disable_search_indexing -> Bool,
        ai_model -> Text,
    }
}

diesel::table! {
    workspace_shared_user (workspace_id, view_id, email) {
        workspace_id -> Text,
        view_id -> Text,
        email -> Text,
        name -> Text,
        avatar_url -> Text,
        role -> Integer,
        access_level -> Integer,
        order -> Integer,
    }
}

diesel::table! {
    workspace_shared_view (uid, workspace_id, view_id) {
        uid -> BigInt,
        workspace_id -> Text,
        view_id -> Text,
        permission_id -> Integer,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    execution_log_table (id) {
        id -> Text,
        session_id -> Text,
        task_plan_id -> Nullable<Text>,
        user_query -> Text,
        start_time -> BigInt,
        end_time -> Nullable<BigInt>,
        status -> Integer,
        error_message -> Nullable<Text>,
        error_type -> Nullable<Integer>,
        agent_id -> Nullable<Text>,
        user_id -> Nullable<Text>,
        workspace_id -> Nullable<Text>,
        total_steps -> Integer,
        completed_steps -> Integer,
        failed_steps -> Integer,
        skipped_steps -> Integer,
        context -> Text,
        result_summary -> Nullable<Text>,
        used_mcp_tools -> Text,
        tags -> Text,
        retry_count -> Integer,
        max_retries -> Integer,
        parent_execution_id -> Nullable<Text>,
        child_execution_ids -> Text,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::table! {
    execution_step_table (id) {
        id -> Text,
        execution_log_id -> Text,
        name -> Text,
        description -> Text,
        mcp_tool_id -> Text,
        mcp_tool_name -> Text,
        mcp_tool_config -> Text,
        input_parameters -> Text,
        output_result -> Nullable<Text>,
        execution_time_ms -> Integer,
        status -> Integer,
        start_time -> Nullable<BigInt>,
        end_time -> Nullable<BigInt>,
        error_message -> Nullable<Text>,
        error_type -> Nullable<Integer>,
        error_stack -> Nullable<Text>,
        step_order -> Integer,
        retry_count -> Integer,
        max_retries -> Integer,
        dependencies -> Text,
        tags -> Text,
        metadata -> Text,
        can_skip -> Bool,
        is_critical -> Bool,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::table! {
    execution_reference_table (id) {
        id -> Text,
        execution_step_id -> Text,
        reference_type -> Integer,
        title -> Text,
        content -> Nullable<Text>,
        url -> Nullable<Text>,
        source -> Nullable<Text>,
        timestamp -> BigInt,
        metadata -> Text,
        relevance_score -> Double,
        created_at -> BigInt,
    }
}

diesel::table! {
    mcp_tool_info_table (id) {
        id -> Text,
        name -> Text,
        display_name -> Nullable<Text>,
        description -> Text,
        version -> Text,
        provider -> Text,
        category -> Text,
        status -> Integer,
        config -> Text,
        schema -> Text,
        requires_auth -> Bool,
        auth_config -> Nullable<Text>,
        icon_url -> Nullable<Text>,
        documentation_url -> Nullable<Text>,
        last_checked -> Nullable<BigInt>,
        last_used -> Nullable<BigInt>,
        usage_count -> Integer,
        success_count -> Integer,
        failure_count -> Integer,
        average_execution_time_ms -> Integer,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
  af_collab_metadata,
  chat_local_setting_table,
  chat_message_table,
  chat_table,
  collab_snapshot,
  execution_log_table,
  execution_step_table,
  execution_reference_table,
  mcp_tool_info_table,
  index_collab_record_table,
  local_ai_model_table,
  upload_file_part,
  upload_file_table,
  user_data_migration_records,
  user_table,
  user_workspace_table,
  workspace_members_table,
  workspace_setting_table,
  workspace_shared_user,
  workspace_shared_view,
);
