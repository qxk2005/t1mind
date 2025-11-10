use crate::entities::*;
use crate::notification::{send_notification, UserNotification};
use crate::services::cloud_config::{
  get_cloud_config, get_or_create_cloud_config, save_cloud_config,
};
use crate::services::data_import::prepare_import;
use crate::user_manager::UserManager;
use flowy_error::{ErrorCode, FlowyError, FlowyResult};
use flowy_sqlite::kv::KVStorePreferences;
use flowy_user_pub::entities::*;
use flowy_user_pub::sql::UserWorkspaceChangeset;
use lib_dispatch::prelude::*;
use lib_infra::box_any::BoxAny;
use serde_json::Value;
use std::path::Path;
use std::str::FromStr;
use std::sync::Weak;
use std::{convert::TryInto, sync::Arc};
use tracing::event;
use uuid::Uuid;

fn upgrade_manager(manager: AFPluginState<Weak<UserManager>>) -> FlowyResult<Arc<UserManager>> {
  let manager = manager
    .upgrade()
    .ok_or(FlowyError::internal().with_context("The user session is already drop"))?;
  Ok(manager)
}

fn upgrade_store_preferences(
  store: AFPluginState<Weak<KVStorePreferences>>,
) -> FlowyResult<Arc<KVStorePreferences>> {
  let store = store
    .upgrade()
    .ok_or(FlowyError::internal().with_context("The store preferences is already drop"))?;
  Ok(store)
}

#[tracing::instrument(level = "debug", name = "sign_in", skip(data, manager), fields(
    email = % data.email
), err)]
pub async fn sign_in_with_email_password_handler(
  data: AFPluginData<SignInPayloadPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<GotrueTokenResponsePB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  let params: SignInParams = data.into_inner().try_into()?;

  match manager
    .sign_in_with_password(&params.email, &params.password)
    .await
  {
    Ok(token) => data_result_ok(token.into()),
    Err(err) => Err(err),
  }
}

#[tracing::instrument(
    level = "debug",
    name = "sign_up",
    skip(data, manager),
    fields(
        email = % data.email,
        name = % data.name,
    ),
    err
)]
pub async fn sign_up(
  data: AFPluginData<SignUpPayloadPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<UserProfilePB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  let params: SignUpParams = data.into_inner().try_into()?;
  let auth_type = params.auth_type;

  match manager.sign_up(auth_type, BoxAny::new(params)).await {
    Ok(profile) => data_result_ok(UserProfilePB::from(profile)),
    Err(err) => Err(err),
  }
}

#[tracing::instrument(level = "debug", skip(manager))]
pub async fn init_user_handler(
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let manager = upgrade_manager(manager)?;
  manager.init_user().await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip(manager))]
pub async fn get_user_profile_handler(
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<UserProfilePB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  let session = manager.get_session()?;

  let mut user_profile = manager
    .get_user_profile_from_disk(session.user_id, &session.workspace_id)
    .await?;

  let weak_manager = Arc::downgrade(&manager);
  let cloned_user_profile = user_profile.clone();
  let workspace_id = session.workspace_id.clone();

  // Refresh the user profile in the background
  tokio::spawn(async move {
    if let Some(manager) = weak_manager.upgrade() {
      let _ = manager
        .refresh_user_profile(&cloned_user_profile, &workspace_id)
        .await;
    }
  });

  // When the user is logged in with a local account, the email field is a placeholder and should
  // not be exposed to the client. So we set the email field to an empty string.
  if user_profile.auth_type == AuthType::Local {
    user_profile.email = "".to_string();
  }

  data_result_ok(user_profile.into())
}

#[tracing::instrument(level = "debug", skip(manager))]
pub async fn sign_out_handler(manager: AFPluginState<Weak<UserManager>>) -> Result<(), FlowyError> {
  let (tx, rx) = tokio::sync::oneshot::channel();
  tokio::spawn(async move {
    let result = async {
      let manager = upgrade_manager(manager)?;
      manager.sign_out().await?;
      Ok::<(), FlowyError>(())
    }
    .await;
    let _ = tx.send(result);
  });
  rx.await??;
  Ok(())
}

#[tracing::instrument(level = "debug", skip(manager))]
pub async fn delete_account_handler(
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let manager = upgrade_manager(manager)?;
  manager.delete_account().await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip(data, manager))]
pub async fn update_user_profile_handler(
  data: AFPluginData<UpdateUserProfilePayloadPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let manager = upgrade_manager(manager)?;
  let params: UpdateUserProfileParams = data.into_inner().try_into()?;
  manager.update_user_profile(params).await?;
  Ok(())
}

const APPEARANCE_SETTING_CACHE_KEY: &str = "appearance_settings";

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn set_appearance_setting(
  store_preferences: AFPluginState<Weak<KVStorePreferences>>,
  data: AFPluginData<AppearanceSettingsPB>,
) -> Result<(), FlowyError> {
  let store_preferences = upgrade_store_preferences(store_preferences)?;
  let mut setting = data.into_inner();
  if setting.theme.is_empty() {
    setting.theme = APPEARANCE_DEFAULT_THEME.to_string();
  }
  store_preferences.set_object(APPEARANCE_SETTING_CACHE_KEY, &setting)?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn get_appearance_setting(
  store_preferences: AFPluginState<Weak<KVStorePreferences>>,
) -> DataResult<AppearanceSettingsPB, FlowyError> {
  let store_preferences = upgrade_store_preferences(store_preferences)?;
  match store_preferences.get_str(APPEARANCE_SETTING_CACHE_KEY) {
    None => data_result_ok(AppearanceSettingsPB::default()),
    Some(s) => {
      let setting = serde_json::from_str(&s).unwrap_or_else(|err| {
        tracing::error!(
          "Deserialize AppearanceSettings failed: {:?}, fallback to default",
          err
        );
        AppearanceSettingsPB::default()
      });
      data_result_ok(setting)
    },
  }
}

const DATE_TIME_SETTINGS_CACHE_KEY: &str = "date_time_settings";

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn set_date_time_settings(
  store_preferences: AFPluginState<Weak<KVStorePreferences>>,
  data: AFPluginData<DateTimeSettingsPB>,
) -> Result<(), FlowyError> {
  let store_preferences = upgrade_store_preferences(store_preferences)?;
  let mut setting = data.into_inner();
  if setting.timezone_id.is_empty() {
    setting.timezone_id = "".to_string();
  }

  store_preferences.set_object(DATE_TIME_SETTINGS_CACHE_KEY, &setting)?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn get_date_time_settings(
  store_preferences: AFPluginState<Weak<KVStorePreferences>>,
) -> DataResult<DateTimeSettingsPB, FlowyError> {
  let store_preferences = upgrade_store_preferences(store_preferences)?;
  match store_preferences.get_str(DATE_TIME_SETTINGS_CACHE_KEY) {
    None => data_result_ok(DateTimeSettingsPB::default()),
    Some(s) => {
      let setting = match serde_json::from_str(&s) {
        Ok(setting) => setting,
        Err(e) => {
          tracing::error!(
            "Deserialize DateTimeSettings failed: {:?}, fallback to default",
            e
          );
          DateTimeSettingsPB::default()
        },
      };
      data_result_ok(setting)
    },
  }
}

const NOTIFICATION_SETTINGS_CACHE_KEY: &str = "notification_settings";
const IMPORT_SETTINGS_CACHE_KEY: &str = "import_settings";

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn set_notification_settings(
  store_preferences: AFPluginState<Weak<KVStorePreferences>>,
  data: AFPluginData<NotificationSettingsPB>,
) -> Result<(), FlowyError> {
  let store_preferences = upgrade_store_preferences(store_preferences)?;
  let setting = data.into_inner();
  store_preferences.set_object(NOTIFICATION_SETTINGS_CACHE_KEY, &setting)?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn get_notification_settings(
  store_preferences: AFPluginState<Weak<KVStorePreferences>>,
) -> DataResult<NotificationSettingsPB, FlowyError> {
  let store_preferences = upgrade_store_preferences(store_preferences)?;
  match store_preferences.get_str(NOTIFICATION_SETTINGS_CACHE_KEY) {
    None => data_result_ok(NotificationSettingsPB::default()),
    Some(s) => {
      let setting = serde_json::from_str(&s).unwrap_or_else(|e| {
        tracing::error!(
          "Deserialize NotificationSettings failed: {:?}, fallback to default",
          e
        );
        NotificationSettingsPB::default()
      });
      data_result_ok(setting)
    },
  }
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn import_appflowy_data_folder_handler(
  data: AFPluginData<ImportAppFlowyDataPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let data = data.try_into_inner()?;
  let (tx, rx) = tokio::sync::oneshot::channel();
  tokio::spawn(async move {
    let result = async {
      let manager = upgrade_manager(manager)?;
      let imported_folder = prepare_import(
        &data.path,
        data.parent_view_id,
        &manager.authenticate_user.user_config.app_version,
      )
      .map_err(|err| FlowyError::new(ErrorCode::AppFlowyDataFolderImportError, err.to_string()))?
      .with_container_name(data.import_container_name);

      manager.perform_import(imported_folder).await?;
      Ok::<(), FlowyError>(())
    }
    .await;
    let _ = tx.send(result);
  });
  rx.await??;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn get_user_setting(
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<UserSettingPB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  let user_setting = manager.user_setting()?;
  data_result_ok(user_setting)
}

#[tracing::instrument(level = "debug", skip(data, manager), err)]
pub async fn sign_in_with_magic_link_handler(
  data: AFPluginData<MagicLinkSignInPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let manager = upgrade_manager(manager)?;
  let params = data.into_inner();
  manager
    .sign_in_with_magic_link(&params.email, &params.redirect_to)
    .await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip(data, manager), err)]
pub async fn sign_in_with_passcode_handler(
  data: AFPluginData<PasscodeSignInPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<GotrueTokenResponsePB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  let params = data.into_inner();
  let response = manager
    .sign_in_with_passcode(&params.email, &params.passcode)
    .await?;
  data_result_ok(response.into())
}

#[tracing::instrument(level = "debug", skip(data, manager), err)]
pub async fn oauth_sign_in_handler(
  data: AFPluginData<OauthSignInPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<UserProfilePB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  let params = data.into_inner();
  let authenticator: AuthType = params.auth_type.into();
  let user_profile = manager
    .sign_up(authenticator, BoxAny::new(params.map))
    .await?;
  data_result_ok(user_profile.into())
}

#[tracing::instrument(level = "debug", skip(data, manager), err)]
pub async fn gen_sign_in_url_handler(
  data: AFPluginData<SignInUrlPayloadPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<SignInUrlPB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  let params = data.into_inner();
  let authenticator: AuthType = params.authenticator.into();
  let sign_in_url = manager
    .generate_sign_in_url_with_email(&authenticator, &params.email)
    .await?;
  data_result_ok(SignInUrlPB { sign_in_url })
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn sign_in_with_provider_handler(
  data: AFPluginData<OauthProviderPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<OauthProviderDataPB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  tracing::debug!("Sign in with provider: {:?}", data.provider.as_str());
  let sign_in_url = manager.generate_oauth_url(data.provider.as_str()).await?;
  event!(tracing::Level::DEBUG, "Sign in url: {}", sign_in_url);
  data_result_ok(OauthProviderDataPB {
    oauth_url: sign_in_url,
  })
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn set_cloud_config_handler(
  manager: AFPluginState<Weak<UserManager>>,
  data: AFPluginData<UpdateCloudConfigPB>,
  store_preferences: AFPluginState<Weak<KVStorePreferences>>,
) -> Result<(), FlowyError> {
  let manager = upgrade_manager(manager)?;
  let session = manager.get_session()?;
  let update = data.into_inner();
  let store_preferences = upgrade_store_preferences(store_preferences)?;
  let mut config = get_cloud_config(session.user_id, &store_preferences)
    .ok_or(FlowyError::internal().with_context("Can't find any cloud config"))?;

  let cloud_service = manager.cloud_service()?;
  if let Some(enable_sync) = update.enable_sync {
    cloud_service.set_enable_sync(session.user_id, enable_sync);
    config.enable_sync = enable_sync;
  }

  save_cloud_config(session.user_id, &store_preferences, &config)?;

  let payload = CloudSettingPB {
    enable_sync: config.enable_sync,
    enable_encrypt: config.enable_encrypt,
    encrypt_secret: config.encrypt_secret,
    server_url: cloud_service.service_url(),
  };

  send_notification(
    // Don't change this key. it's also used in the frontend
    "user_cloud_config",
    UserNotification::DidUpdateCloudConfig,
  )
  .payload(payload)
  .send();
  Ok(())
}

pub async fn get_cloud_config_handler(
  manager: AFPluginState<Weak<UserManager>>,
  store_preferences: AFPluginState<Weak<KVStorePreferences>>,
) -> DataResult<CloudSettingPB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  let session = manager.get_session()?;
  let store_preferences = upgrade_store_preferences(store_preferences)?;
  let cloud_service = manager.cloud_service()?;
  // Generate the default config if the config is not exist
  let config = get_or_create_cloud_config(session.user_id, &store_preferences);
  data_result_ok(CloudSettingPB {
    enable_sync: config.enable_sync,
    enable_encrypt: config.enable_encrypt,
    encrypt_secret: config.encrypt_secret,
    server_url: cloud_service.service_url(),
  })
}

#[tracing::instrument(level = "debug", skip(manager), err)]
pub async fn get_all_workspace_handler(
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<RepeatedUserWorkspacePB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  let session = manager.get_session()?;
  let profile = manager
    .get_user_profile_from_disk(session.user_id, &session.workspace_id)
    .await?;
  let user_workspaces = manager
    .get_all_user_workspaces(profile.uid, profile.auth_type)
    .await?;

  data_result_ok(RepeatedUserWorkspacePB::from(user_workspaces))
}

#[tracing::instrument(level = "info", skip(data, manager), err)]
pub async fn open_workspace_handler(
  data: AFPluginData<OpenUserWorkspacePB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let manager = upgrade_manager(manager)?;
  let params = data.try_into_inner()?;
  let workspace_id = Uuid::from_str(&params.workspace_id)?;
  manager
    .open_workspace(&workspace_id, WorkspaceType::from(params.workspace_type))
    .await?;
  Ok(())
}

#[tracing::instrument(level = "info", skip(data, manager), err)]
pub async fn get_user_workspace_handler(
  data: AFPluginData<UserWorkspaceIdPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<UserWorkspacePB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  let params = data.try_into_inner()?;
  let workspace_id = Uuid::from_str(&params.workspace_id)?;
  let uid = manager.user_id()?;
  let user_workspace = manager.get_user_workspace_from_db(uid, &workspace_id)?;
  data_result_ok(UserWorkspacePB::from(user_workspace))
}

#[tracing::instrument(level = "debug", skip(data, manager), err)]
pub async fn update_network_state_handler(
  data: AFPluginData<NetworkStatePB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let manager = upgrade_manager(manager)?;
  let reachable = data.into_inner().ty.is_reachable();
  manager.cloud_service()?.set_network_reachable(reachable);
  manager
    .app_life_cycle
    .read()
    .await
    .on_network_status_changed(reachable);
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn get_anon_user_handler(
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<UserProfilePB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  let user_profile = manager.get_anon_user().await?;
  data_result_ok(user_profile)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn open_anon_user_handler(
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let manager = upgrade_manager(manager)?;
  manager.open_anon_user().await?;
  Ok(())
}

pub async fn push_realtime_event_handler(
  payload: AFPluginData<RealtimePayloadPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  match serde_json::from_str::<Value>(&payload.into_inner().json_str) {
    Ok(json) => {
      let manager = upgrade_manager(manager)?;
      manager.receive_realtime_event(json).await;
    },
    Err(e) => {
      tracing::error!("Deserialize RealtimePayload failed: {:?}", e);
    },
  }
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn create_reminder_event_handler(
  data: AFPluginData<ReminderPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let manager = upgrade_manager(manager)?;
  let params = data.into_inner();
  manager.add_reminder(params).await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn get_all_reminder_event_handler(
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<RepeatedReminderPB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  let reminders = manager
    .get_all_reminders()
    .await
    .unwrap_or_default()
    .into_iter()
    .map(ReminderPB::from)
    .collect::<Vec<_>>();

  data_result_ok(reminders.into())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn remove_reminder_event_handler(
  data: AFPluginData<ReminderIdentifierPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let manager = upgrade_manager(manager)?;

  let params = data.into_inner();
  let _ = manager.remove_reminder(params.id.as_str()).await;

  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn update_reminder_event_handler(
  data: AFPluginData<ReminderPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let manager = upgrade_manager(manager)?;
  let params = data.into_inner();
  manager.update_reminder(params).await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn delete_workspace_member_handler(
  data: AFPluginData<RemoveWorkspaceMemberPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let data = data.try_into_inner()?;
  let manager = upgrade_manager(manager)?;
  let workspace_id = Uuid::from_str(&data.workspace_id)?;
  manager
    .remove_workspace_member(data.email, workspace_id)
    .await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn get_workspace_members_handler(
  data: AFPluginData<QueryWorkspacePB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<RepeatedWorkspaceMemberPB, FlowyError> {
  let data = data.try_into_inner()?;
  let manager = upgrade_manager(manager)?;
  let workspace_id = Uuid::from_str(&data.workspace_id)?;
  let members = manager
    .get_workspace_members(workspace_id)
    .await?
    .into_iter()
    .map(WorkspaceMemberPB::from)
    .collect();
  data_result_ok(RepeatedWorkspaceMemberPB { items: members })
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn update_workspace_member_handler(
  data: AFPluginData<UpdateWorkspaceMemberPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let data = data.try_into_inner()?;
  let manager = upgrade_manager(manager)?;
  let workspace_id = Uuid::from_str(&data.workspace_id)?;
  manager
    .update_workspace_member(data.email, workspace_id, data.role.into())
    .await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn create_workspace_handler(
  data: AFPluginData<CreateWorkspacePB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<UserWorkspacePB, FlowyError> {
  let data = data.try_into_inner()?;
  let workspace_type = WorkspaceType::from(data.workspace_type);
  let manager = upgrade_manager(manager)?;
  let new_workspace = manager.create_workspace(&data.name, workspace_type).await?;
  data_result_ok(UserWorkspacePB::from(new_workspace))
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn delete_workspace_handler(
  delete_workspace_param: AFPluginData<UserWorkspaceIdPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let workspace_id = delete_workspace_param.try_into_inner()?.workspace_id;
  let manager = upgrade_manager(manager)?;
  let workspace_id = Uuid::from_str(&workspace_id)?;
  manager.delete_workspace(&workspace_id).await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn rename_workspace_handler(
  rename_workspace_param: AFPluginData<RenameWorkspacePB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let params = rename_workspace_param.try_into_inner()?;
  let manager = upgrade_manager(manager)?;
  let workspace_id = Uuid::from_str(&params.workspace_id)?;
  let changeset = UserWorkspaceChangeset {
    id: params.workspace_id,
    name: Some(params.new_name),
    icon: None,
    role: None,
    member_count: None,
  };
  manager.patch_workspace(&workspace_id, changeset).await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn change_workspace_icon_handler(
  change_workspace_icon_param: AFPluginData<ChangeWorkspaceIconPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let params = change_workspace_icon_param.try_into_inner()?;
  let manager = upgrade_manager(manager)?;
  let workspace_id = Uuid::from_str(&params.workspace_id)?;
  let changeset = UserWorkspaceChangeset {
    id: workspace_id.to_string(),
    name: None,
    icon: Some(params.new_icon),
    role: None,
    member_count: None,
  };
  manager.patch_workspace(&workspace_id, changeset).await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn invite_workspace_member_handler(
  param: AFPluginData<WorkspaceMemberInvitationPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let param = param.try_into_inner()?;
  let manager = upgrade_manager(manager)?;
  let workspace_id = Uuid::from_str(&param.workspace_id)?;
  manager
    .invite_member_to_workspace(workspace_id, param.invitee_email, param.role.into())
    .await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn list_workspace_invitations_handler(
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<RepeatedWorkspaceInvitationPB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  let invitations = manager.list_pending_workspace_invitations().await?;
  let invitations_pb: Vec<WorkspaceInvitationPB> = invitations
    .into_iter()
    .map(WorkspaceInvitationPB::from)
    .collect();
  data_result_ok(RepeatedWorkspaceInvitationPB {
    items: invitations_pb,
  })
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn accept_workspace_invitations_handler(
  param: AFPluginData<AcceptWorkspaceInvitationPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let invite_id = param.try_into_inner()?.invite_id;
  let manager = upgrade_manager(manager)?;
  manager.accept_workspace_invitation(invite_id).await?;
  Ok(())
}

pub async fn leave_workspace_handler(
  param: AFPluginData<UserWorkspaceIdPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let workspace_id = param.into_inner().workspace_id;
  let workspace_id = Uuid::from_str(&workspace_id)?;
  let manager = upgrade_manager(manager)?;
  manager.leave_workspace(&workspace_id).await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn subscribe_workspace_handler(
  params: AFPluginData<SubscribeWorkspacePB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<PaymentLinkPB, FlowyError> {
  let params = params.try_into_inner()?;
  let manager = upgrade_manager(manager)?;
  let payment_link = manager.subscribe_workspace(params).await?;
  data_result_ok(PaymentLinkPB { payment_link })
}

pub async fn get_workspace_subscription_info_handler(
  params: AFPluginData<UserWorkspaceIdPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<WorkspaceSubscriptionInfoPB, FlowyError> {
  let params = params.try_into_inner()?;
  let manager = upgrade_manager(manager)?;
  let subs = manager
    .get_workspace_subscription_info(params.workspace_id)
    .await?;
  data_result_ok(subs)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn cancel_workspace_subscription_handler(
  param: AFPluginData<CancelWorkspaceSubscriptionPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let params = param.into_inner();
  let manager = upgrade_manager(manager)?;
  manager
    .cancel_workspace_subscription(params.workspace_id, params.plan.into(), Some(params.reason))
    .await?;
  Ok(())
}

pub async fn get_workspace_usage_handler(
  param: AFPluginData<UserWorkspaceIdPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<WorkspaceUsagePB, FlowyError> {
  let workspace_id = Uuid::from_str(&param.into_inner().workspace_id)?;
  let manager = upgrade_manager(manager)?;
  let workspace_usage = manager.get_workspace_usage(&workspace_id).await?;
  data_result_ok(WorkspaceUsagePB::from(workspace_usage))
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn get_billing_portal_handler(
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<BillingPortalPB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  let url = manager.get_billing_portal_url().await?;
  data_result_ok(BillingPortalPB { url })
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn update_workspace_subscription_payment_period_handler(
  params: AFPluginData<UpdateWorkspaceSubscriptionPaymentPeriodPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> FlowyResult<()> {
  let workspace_id = Uuid::from_str(&params.workspace_id)?;
  let params = params.try_into_inner()?;
  let manager = upgrade_manager(manager)?;
  manager
    .update_workspace_subscription_payment_period(
      &workspace_id,
      params.plan.into(),
      params.recurring_interval.into(),
    )
    .await
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn get_subscription_plan_details_handler(
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<RepeatedSubscriptionPlanDetailPB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  let plans = manager
    .get_subscription_plan_details()
    .await?
    .into_iter()
    .map(SubscriptionPlanDetailPB::from)
    .collect::<Vec<_>>();
  data_result_ok(RepeatedSubscriptionPlanDetailPB { items: plans })
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn get_workspace_member_info(
  param: AFPluginData<WorkspaceMemberIdPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<WorkspaceMemberPB, FlowyError> {
  let manager = upgrade_manager(manager)?;
  let workspace_id = Uuid::parse_str(&manager.get_session()?.workspace_id)?;
  let member = manager
    .get_workspace_member_info(param.uid, &workspace_id)
    .await?;
  data_result_ok(member.into())
}

#[tracing::instrument(level = "info", skip_all, err)]
pub async fn update_workspace_setting_handler(
  params: AFPluginData<UpdateUserWorkspaceSettingPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let params = params.try_into_inner()?;
  let manager = upgrade_manager(manager)?;
  manager.update_workspace_setting(params).await?;
  Ok(())
}

#[tracing::instrument(level = "info", skip_all, err)]
pub async fn get_workspace_setting_handler(
  params: AFPluginData<UserWorkspaceIdPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> DataResult<WorkspaceSettingsPB, FlowyError> {
  let params = params.try_into_inner()?;
  let workspace_id = Uuid::from_str(&params.workspace_id)?;
  let manager = upgrade_manager(manager)?;
  let pb = manager.get_workspace_settings(&workspace_id).await?;
  data_result_ok(pb)
}

#[tracing::instrument(level = "info", skip_all, err)]
pub async fn notify_did_switch_plan_handler(
  params: AFPluginData<SuccessWorkspaceSubscriptionPB>,
  manager: AFPluginState<Weak<UserManager>>,
) -> Result<(), FlowyError> {
  let success = params.into_inner();
  let manager = upgrade_manager(manager)?;
  manager.notify_did_switch_plan(success).await?;
  Ok(())
}

// Import Settings Handlers

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn set_import_settings(
  store_preferences: AFPluginState<Weak<KVStorePreferences>>,
  data: AFPluginData<ImportSettingsPB>,
) -> Result<(), FlowyError> {
  let store_preferences = upgrade_store_preferences(store_preferences)?;
  let setting = data.into_inner();
  store_preferences.set_object(IMPORT_SETTINGS_CACHE_KEY, &setting)?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn get_import_settings(
  store_preferences: AFPluginState<Weak<KVStorePreferences>>,
) -> DataResult<ImportSettingsPB, FlowyError> {
  let store_preferences = upgrade_store_preferences(store_preferences)?;
  match store_preferences.get_str(IMPORT_SETTINGS_CACHE_KEY) {
    None => data_result_ok(ImportSettingsPB::default()),
    Some(s) => {
      let setting = serde_json::from_str(&s).unwrap_or_else(|e| {
        tracing::error!(
          "Deserialize ImportSettings failed: {:?}, fallback to default",
          e
        );
        ImportSettingsPB::default()
      });
      data_result_ok(setting)
    },
  }
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn update_import_settings(
  store_preferences: AFPluginState<Weak<KVStorePreferences>>,
  data: AFPluginData<UpdateImportSettingsPB>,
) -> Result<(), FlowyError> {
  let store_preferences = upgrade_store_preferences(store_preferences)?;
  let update = data.into_inner();
  
  // Get current settings or use default
  let mut current_settings = match store_preferences.get_str(IMPORT_SETTINGS_CACHE_KEY) {
    None => ImportSettingsPB::default(),
    Some(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
      tracing::error!(
        "Deserialize ImportSettings failed: {:?}, fallback to default",
        e
      );
      ImportSettingsPB::default()
    }),
  };
  
  // Apply updates
  if let Some(max_concurrent_conversions) = update.max_concurrent_conversions {
    current_settings.max_concurrent_conversions = max_concurrent_conversions;
  }
  if let Some(default_import_path) = update.default_import_path {
    current_settings.default_import_path = default_import_path;
  }
  if let Some(preserve_formatting) = update.preserve_formatting {
    current_settings.preserve_formatting = preserve_formatting;
  }
  if let Some(extract_images) = update.extract_images {
    current_settings.extract_images = extract_images;
  }
  if let Some(extract_tables) = update.extract_tables {
    current_settings.extract_tables = extract_tables;
  }
  if let Some(log_level) = update.log_level {
    current_settings.log_level = log_level;
  }
  if let Some(auto_create_folder) = update.auto_create_folder {
    current_settings.auto_create_folder = auto_create_folder;
  }
  if let Some(enable_progress_notifications) = update.enable_progress_notifications {
    current_settings.enable_progress_notifications = enable_progress_notifications;
  }
  if let Some(conversion_timeout_seconds) = update.conversion_timeout_seconds {
    current_settings.conversion_timeout_seconds = conversion_timeout_seconds;
  }
  if let Some(auto_retry_on_failure) = update.auto_retry_on_failure {
    current_settings.auto_retry_on_failure = auto_retry_on_failure;
  }
  if let Some(max_retry_attempts) = update.max_retry_attempts {
    current_settings.max_retry_attempts = max_retry_attempts;
  }
  if let Some(save_conversion_logs) = update.save_conversion_logs {
    current_settings.save_conversion_logs = save_conversion_logs;
  }
  if let Some(log_retention_days) = update.log_retention_days {
    current_settings.log_retention_days = log_retention_days;
  }
  
  // Save updated settings
  store_preferences.set_object(IMPORT_SETTINGS_CACHE_KEY, &current_settings)?;
  Ok(())
}

/// 检查 PDF 导入工具状态
#[tracing::instrument(level = "info", skip_all, err)]
pub async fn check_import_tools_status() -> DataResult<crate::entities::ImportToolsStatusPB, FlowyError> {
  use crate::entities::ImportToolsStatusPB;
  use std::time::{SystemTime, UNIX_EPOCH};
  
  tracing::info!("[工具检查] ========== 开始检查导入工具 ==========");
  
  let mut tools = Vec::new();
  
  // 只检查 Marker 工具（精准 PDF 导入工具）
  // 其他工具（poppler、tesseract、python、pdfminer 等）已不再需要
  let marker_info = check_marker_tool();
  tools.push(marker_info);
  
  let checked_at = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;
  
  tracing::info!("[工具检查] ========== 工具检查完成，共检查 {} 个工具 ==========", tools.len());
  for tool in &tools {
    tracing::info!(
      "[工具检查] {} - 状态: {:?}, 版本: {:?}, 路径: {:?}",
      tool.name,
      tool.status,
      tool.version,
      tool.path
    );
  }
  
  let result = ImportToolsStatusPB {
    tools,
    checked_at,
  };
  
  data_result_ok(result)
}

/// 检查单个工具
fn check_tool(
  command: &str,
  display_name: &str,
  description: &str,
  install_instruction: &str,
  version_cmd: Option<Vec<&str>>,
) -> crate::entities::ImportToolInfoPB {
  use std::process::Command;
  
  tracing::info!("[工具检查] 开始检查工具: {} ({})", display_name, command);
  
  // 记录当前环境变量
  let current_path = std::env::var("PATH").unwrap_or_default();
  tracing::info!("[工具检查] 当前 PATH: {}", current_path);
  let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
  tracing::info!("[工具检查] 当前 SHELL: {}", shell);
  
  // 首先尝试使用 which/where 命令查找工具路径（跨平台）
  let path_from_which = {
    #[cfg(target_os = "windows")]
    {
      // Windows 平台使用 where 命令
      let command_exe = if command.ends_with(".exe") {
        command.to_string()
      } else {
        format!("{}.exe", command)
      };
      
      // 尝试使用 where 命令
      match Command::new("where").arg(&command_exe).output() {
        Ok(output) => {
          let stdout = String::from_utf8_lossy(&output.stdout);
          let stderr = String::from_utf8_lossy(&output.stderr);
          tracing::info!(
            "[工具检查] where 命令执行完成 - 状态: {}, stdout: '{}', stderr: '{}'",
            output.status,
            stdout.trim(),
            stderr.trim()
          );
          
          if output.status.success() {
            // where 命令可能返回多行，取第一行
            let path_str = stdout.lines().next()
              .map(|s| s.trim().to_string())
              .filter(|s| !s.is_empty());
            if let Some(path) = path_str {
              tracing::info!("[工具检查] 通过 where 找到 {}: {}", command, path);
              Some(path)
            } else {
              None
            }
          } else {
            None
          }
        }
        Err(e) => {
          tracing::error!(
            "[工具检查] where 执行出错 - 错误类型: {:?}, 错误信息: {}",
            e.kind(),
            e
          );
          None
        }
      }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
      // Unix-like 平台使用 which 命令
      // 尝试使用 shell 环境来执行 which，以确保能获取正确的 PATH
      std::env::var("SHELL")
        .ok()
        .and_then(|shell| {
          // 构建命令，确保加载 shell 配置文件以获取正确的 PATH
          let which_cmd = if shell.contains("zsh") {
            // 对于 zsh，加载 .zshrc 或 .zprofile，然后执行 which
            format!("source ~/.zshrc 2>/dev/null || source ~/.zprofile 2>/dev/null || true; which {}", command)
          } else if shell.contains("bash") {
            // 对于 bash，加载 .bash_profile 或 .bashrc
            format!("source ~/.bash_profile 2>/dev/null || source ~/.bashrc 2>/dev/null || true; which {}", command)
          } else {
            format!("which {}", command)
          };
          
          tracing::info!("[工具检查] 尝试通过 shell 执行 which: {} -c \"{}\"", shell, which_cmd);
          
          // 使用 shell 来执行 which 命令，这样可以获取正确的 PATH
          match Command::new(&shell)
            .arg("-c")
            .arg(&which_cmd)
            .output()
          {
            Ok(output) => {
              let stdout = String::from_utf8_lossy(&output.stdout);
              let stderr = String::from_utf8_lossy(&output.stderr);
              tracing::info!(
                "[工具检查] Shell which 命令执行完成 - 状态: {}, stdout: '{}', stderr: '{}'",
                output.status,
                stdout.trim(),
                stderr.trim()
              );
              
              if output.status.success() {
                let path_str = stdout.trim().to_string();
                if !path_str.is_empty() {
                  tracing::info!("[工具检查] 通过 shell which 找到 {}: {}", command, path_str);
                  Some(path_str)
                } else {
                  tracing::warn!("[工具检查] Shell which 返回空结果");
                  None
                }
              } else {
                tracing::debug!("[工具检查] Shell which 命令失败: {}", stderr.trim());
                None
              }
            }
            Err(e) => {
              // 记录权限或执行错误
              tracing::error!(
                "[工具检查] Shell which 执行出错 - 错误类型: {:?}, 错误信息: {}",
                e.kind(),
                e
              );
              if e.kind() == std::io::ErrorKind::PermissionDenied {
                tracing::warn!("[工具检查] 权限被拒绝: {}", e);
              }
              None
            }
          }
        })
        .or_else(|| {
          tracing::info!("[工具检查] Shell which 失败，回退到直接执行 which {}", command);
          // 如果 shell 方式失败，回退到直接使用 which 命令
          match Command::new("which")
            .arg(command)
            .output()
          {
            Ok(output) => {
              let stdout = String::from_utf8_lossy(&output.stdout);
              let stderr = String::from_utf8_lossy(&output.stderr);
              tracing::info!(
                "[工具检查] 直接 which 命令执行完成 - 状态: {}, stdout: '{}', stderr: '{}'",
                output.status,
                stdout.trim(),
                stderr.trim()
              );
              
              if output.status.success() {
                let path_str = stdout.trim().to_string();
                if !path_str.is_empty() {
                  tracing::info!("[工具检查] 通过直接 which 找到 {}: {}", command, path_str);
                  Some(path_str)
                } else {
                  None
                }
              } else {
                None
              }
            }
            Err(e) => {
              tracing::error!(
                "[工具检查] 直接 which 执行出错 - 错误类型: {:?}, 错误信息: {}",
                e.kind(),
                e
              );
              if e.kind() == std::io::ErrorKind::PermissionDenied {
                tracing::warn!("[工具检查] 权限被拒绝: {}", e);
              }
              None
            }
          }
        })
    }
  };
  
  // 在尝试执行版本命令之前，先检查常见路径中是否存在可执行文件
  let path_from_common_dirs = {
    #[cfg(target_os = "windows")]
    {
      // Windows 平台
      let command_exe = if command.ends_with(".exe") {
        command.to_string()
      } else {
        format!("{}.exe", command)
      };
      
      let common_paths = vec![
        r"C:\Program Files\poppler\bin",
        r"C:\Program Files (x86)\poppler\bin",
        r"C:\poppler\bin",
        format!(r"{}\poppler\bin", std::env::var("USERPROFILE").unwrap_or_default()),
        r"C:\Windows\System32",
        r"C:\Windows",
      ];
      
      let mut found_path = None;
      for base_path in common_paths {
        let test_path = std::path::Path::new(&base_path).join(&command_exe);
        if test_path.exists() {
          tracing::info!("[工具检查] 在常见路径中找到 {} 可执行文件: {}", command, test_path.display());
          found_path = Some(test_path.to_string_lossy().to_string());
          break;
        }
      }
      found_path
    }
    
    #[cfg(not(target_os = "windows"))]
    {
      // Unix-like 平台（macOS, Linux）
      let common_paths = vec![
        "/opt/homebrew/bin",  // Apple Silicon Mac
        "/usr/local/bin",      // Intel Mac / Linux
        "/usr/bin",
        "/bin",
      ];
      
      let mut found_path = None;
      for base_path in common_paths {
        let test_path = format!("{}/{}", base_path, command);
        if std::path::Path::new(&test_path).exists() {
          tracing::info!("[工具检查] 在常见路径中找到 {} 可执行文件: {}", command, test_path);
          found_path = Some(test_path);
          break;
        }
      }
      found_path
    }
  };
  
  // 如果 which 找不到，尝试直接执行命令来检查是否存在
  // 这在 macOS 应用中特别重要，因为应用的 PATH 可能不包含 Homebrew 路径
  let (status, version, path) = if let Some(cmd_parts) = version_cmd.as_ref() {
    // 尝试执行版本命令来检查工具是否存在
    // 优先使用 shell 环境执行，以确保能获取正确的 PATH
    let version_output_result = std::env::var("SHELL")
      .ok()
      .and_then(|shell| {
        // 构建命令字符串（参数通常比较简单，如 -v 或 --version）
        let cmd_str = if cmd_parts.len() > 1 {
          let args: Vec<String> = cmd_parts[1..].iter().map(|s| s.to_string()).collect();
          format!("{} {}", cmd_parts[0], args.join(" "))
        } else {
          cmd_parts[0].to_string()
        };
        
        // 使用 shell 执行命令，确保加载完整的 PATH 环境
        // 对于 zsh，需要加载 .zshrc 或 .zprofile 来获取正确的 PATH
        tracing::info!("[工具检查] 尝试通过 shell 执行版本命令: {} -c \"{}\"", shell, cmd_str);
        
        // 构建一个更完整的命令，确保加载环境变量
        // 对于 macOS，Homebrew 路径通常在 ~/.zprofile 或 ~/.zshrc 中设置
        let full_cmd = if shell.contains("zsh") {
          // 对于 zsh，先 source .zprofile 或 .zshrc，然后执行命令
          // .zprofile 在登录时加载，通常包含 Homebrew 的 PATH 设置
          format!("source ~/.zprofile 2>/dev/null || source ~/.zshrc 2>/dev/null || true; {}", cmd_str)
        } else if shell.contains("bash") {
          // 对于 bash，先 source .bash_profile 或 .bashrc
          format!("source ~/.bash_profile 2>/dev/null || source ~/.bashrc 2>/dev/null || true; {}", cmd_str)
        } else {
          cmd_str
        };
        
        tracing::info!("[工具检查] 完整命令: {} -c \"{}\"", shell, full_cmd);
        
        match Command::new(&shell)
          .arg("-c")
          .arg(&full_cmd)
          .output()
        {
          Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let status_code = output.status.code().unwrap_or(-1);
            tracing::info!(
              "[工具检查] Shell 版本命令执行完成 - 退出码: {}, 状态: {}, stdout: '{}', stderr: '{}'",
              status_code,
              output.status,
              stdout.trim(),
              stderr.trim()
            );
            
            if output.status.success() {
              tracing::info!("[工具检查] 通过 shell 成功执行 {} 版本命令", command);
              Some(output)
            } else {
              tracing::warn!(
                "[工具检查] Shell 版本命令执行失败 - 退出码: {}, stderr: '{}'",
                status_code,
                stderr.trim()
              );
              None
            }
          }
          Err(e) => {
            tracing::error!(
              "[工具检查] Shell 版本命令执行出错 - 错误类型: {:?}, 错误信息: {}",
              e.kind(),
              e
            );
            if e.kind() == std::io::ErrorKind::PermissionDenied {
              tracing::warn!("[工具检查] 权限被拒绝: {}", e);
            }
            None
          }
        }
      })
      .or_else(|| {
        // 如果 shell 方式失败，尝试使用找到的路径来执行命令
        if let Some(ref found_path) = path_from_common_dirs {
          tracing::info!("[工具检查] 尝试使用找到的路径执行命令: {}", found_path);
          match Command::new(found_path)
            .args(&cmd_parts[1..])
            .output()
          {
            Ok(output) => {
              let stdout = String::from_utf8_lossy(&output.stdout);
              let stderr = String::from_utf8_lossy(&output.stderr);
              tracing::info!(
                "[工具检查] 使用路径执行命令完成 - 状态: {}, stdout: '{}', stderr: '{}'",
                output.status,
                stdout.trim(),
                stderr.trim()
              );
              
              if output.status.success() {
                tracing::info!("[工具检查] 使用路径执行 {} 命令成功", command);
                return Some(output);
              }
            }
            Err(e) => {
              tracing::warn!("[工具检查] 使用路径执行命令失败: {}", e);
            }
          }
        }
        
        // 回退到直接执行命令（依赖 PATH）
        let cmd_args: Vec<String> = cmd_parts.iter().map(|s| s.to_string()).collect();
        tracing::info!("[工具检查] Shell 方式失败，回退到直接执行命令: {:?}", cmd_args);
        
        match Command::new(cmd_parts[0])
          .args(&cmd_parts[1..])
          .output()
        {
          Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::info!(
              "[工具检查] 直接执行命令完成 - 状态: {}, stdout: '{}', stderr: '{}'",
              output.status,
              stdout.trim(),
              stderr.trim()
            );
            
            if output.status.success() {
              tracing::info!("[工具检查] 直接执行 {} 命令成功", command);
              Some(output)
            } else {
              tracing::warn!("[工具检查] 直接执行 {} 命令失败", command);
              None
            }
          }
          Err(e) => {
            tracing::error!(
              "[工具检查] 直接执行命令出错 - 错误类型: {:?}, 错误信息: {}",
              e.kind(),
              e
            );
            if e.kind() == std::io::ErrorKind::PermissionDenied {
              tracing::warn!("[工具检查] 权限被拒绝: {}", e);
            } else if e.kind() == std::io::ErrorKind::NotFound {
              tracing::warn!("[工具检查] 命令未找到: {}", cmd_parts[0]);
            }
            None
          }
        }
      });
    
    match version_output_result {
      Some(version_output) if version_output.status.success() => {
        // 命令执行成功，说明工具存在
        let version_text = String::from_utf8_lossy(&version_output.stdout);
        // 提取版本号（简化处理）
        let version_str = version_text
          .lines()
          .next()
          .and_then(|line| {
            // 尝试提取版本号
            line.split_whitespace()
              .find(|s| s.chars().any(|c| c.is_ascii_digit()))
              .map(|s| s.to_string())
          })
          .or_else(|| {
            let trimmed = version_text.trim().to_string();
            if !trimmed.is_empty() {
              Some(trimmed)
            } else {
              None
            }
          });
        
        // 如果没有从 which 获取到路径，使用之前找到的常见路径，或尝试查找
        let final_path = path_from_which
          .or(path_from_common_dirs.clone())
          .or_else(|| {
            tracing::info!("[工具检查] which 未找到路径，尝试在常见路径中查找 {}", command);
            
            // 检查常见的 Homebrew 路径（按优先级）
            let common_paths = vec![
              "/opt/homebrew/bin",  // Apple Silicon Mac 的 Homebrew 路径
              "/usr/local/bin",      // Intel Mac 的 Homebrew 路径
              "/usr/bin",
              "/bin",
            ];
            
            // 尝试在常见路径中查找
            for base_path in common_paths {
              let test_path = format!("{}/{}", base_path, command);
              tracing::debug!("[工具检查] 检查路径: {}", test_path);
              if std::path::Path::new(&test_path).exists() {
                tracing::info!("[工具检查] 在常见路径中找到 {}: {}", command, test_path);
                return Some(test_path);
              }
            }
            
            // 如果都找不到，但命令能执行，说明在 PATH 中，使用命令名作为路径
            tracing::warn!("[工具检查] 在常见路径中未找到 {}，但命令能执行，可能在其他 PATH 中", command);
            Some(command.to_string())
          });
        
        tracing::info!(
          "[工具检查] {} 检查成功 - 状态: 已安装, 版本: {:?}, 路径: {:?}",
          display_name,
          version_str,
          final_path
        );
        (
          ImportToolStatusPB::ToolAvailable,
          version_str,
          final_path,
        )
      }
      Some(_) => {
        // 命令执行失败，但可能是因为参数问题，尝试使用 which
        if let Some(path_str) = path_from_which {
          tracing::info!(
            "[工具检查] {} 检查部分成功 - 状态: 已安装 (版本命令失败但找到路径), 路径: {}",
            display_name,
            path_str
          );
          (
            ImportToolStatusPB::ToolAvailable,
            None,
            Some(path_str),
          )
        } else {
          tracing::warn!(
            "[工具检查] {} 检查失败 - 状态: 未安装 (版本命令失败且未找到路径)",
            display_name
          );
          (
            ImportToolStatusPB::ToolNotInstalled,
            None,
            None,
          )
        }
      }
      None => {
        // 命令执行失败或出错，检查 which 的结果
        if let Some(path_str) = path_from_which {
          // 如果 which 能找到路径，即使版本命令失败，也认为工具存在
          tracing::info!(
            "[工具检查] {} 检查部分成功 - 状态: 已安装 (通过路径找到), 路径: {}",
            display_name,
            path_str
          );
          (
            ImportToolStatusPB::ToolAvailable,
            None,
            Some(path_str),
          )
        } else {
          tracing::warn!(
            "[工具检查] {} 检查失败 - 状态: 未安装 (所有检查方法均失败)",
            display_name
          );
          (
            ImportToolStatusPB::ToolNotInstalled,
            None,
            None,
          )
        }
      }
    }
  } else {
    // 没有版本命令，仅依赖 which
    if let Some(path_str) = path_from_which {
      tracing::info!(
        "[工具检查] {} 检查成功 - 状态: 已安装 (通过路径), 路径: {}",
        display_name,
        path_str
      );
      (
        ImportToolStatusPB::ToolAvailable,
        None,
        Some(path_str),
      )
    } else {
      tracing::warn!(
        "[工具检查] {} 检查失败 - 状态: 未安装 (未找到路径)",
        display_name
      );
      (
        ImportToolStatusPB::ToolNotInstalled,
        None,
        None,
      )
    }
  };
  
  let install_inst = if status == ImportToolStatusPB::ToolNotInstalled {
    Some(install_instruction.to_string())
  } else {
    None
  };
  
  ImportToolInfoPB {
    name: display_name.to_string(),
    status,
    version,
    path,
    install_instruction: install_inst,
    description: description.to_string(),
  }
}

/// 检查 Python 模块（带回退机制，用于检查版本）
fn check_python_module_with_fallback(
  module_to_check: &str,
  version_module: &str,
  package_name: &str,
  description: &str,
  install_instruction: &str,
) -> crate::entities::ImportToolInfoPB {
  use std::process::Command;
  
  // Windows 使用 python，Unix-like 使用 python3
  let python_cmd = if cfg!(target_os = "windows") {
    "python"
  } else {
    "python3"
  };
  
  // 首先检查模块是否可以导入
  let check_import = Command::new(python_cmd)
    .arg("-c")
    .arg(&format!("import {}", module_to_check))
    .output();
  
  let (status, version) = match check_import {
    Ok(output) if output.status.success() => {
      // 模块可以导入，尝试获取版本
      let version_cmd = format!("import {}; print(getattr({}, '__version__', 'unknown'))", version_module, version_module);
      let version_result = Command::new(python_cmd)
        .arg("-c")
        .arg(&version_cmd)
        .output();
      
      let version_str = if let Ok(v_output) = version_result {
        if v_output.status.success() {
          let version_text = String::from_utf8_lossy(&v_output.stdout)
            .trim()
            .to_string();
          if version_text != "unknown" && !version_text.is_empty() {
            Some(version_text)
          } else {
            None
          }
        } else {
          None
        }
      } else {
        None
      };
      
      (ImportToolStatusPB::ToolAvailable, version_str)
    }
    Ok(output) => {
      // 检查错误信息
      let error_msg = String::from_utf8_lossy(&output.stderr);
      if error_msg.contains("ModuleNotFoundError") || error_msg.contains("No module named") {
        (ImportToolStatusPB::ToolNotInstalled, None)
      } else {
        (ImportToolStatusPB::ToolUnavailable, None)
      }
    }
    _ => (ImportToolStatusPB::ToolUnknown, None),
  };
  
  let install_inst = if status == ImportToolStatusPB::ToolNotInstalled {
    Some(install_instruction.to_string())
  } else {
    None
  };
  
  ImportToolInfoPB {
    name: format!("{} ({})", module_to_check, package_name),
    status,
    version,
    path: None,
    install_instruction: install_inst,
    description: description.to_string(),
  }
}

/// 检查 Marker 工具（精准 PDF 导入工具）
/// 
/// Marker 工具用于将 PDF 转换为 Markdown，提供比 OCR 更精准的文本提取。
/// 工具应该位于应用包的 Resources/marker/ 目录中。
fn check_marker_tool() -> crate::entities::ImportToolInfoPB {
  use crate::entities::{ImportToolInfoPB, ImportToolStatusPB};
  
  tracing::info!("[工具检查] 开始检查 Marker 工具");
  
  // 获取当前可执行文件路径
  let exe_path = match std::env::current_exe() {
    Ok(path) => path,
    Err(e) => {
      tracing::warn!("[工具检查] 无法获取当前可执行文件路径: {}", e);
      return ImportToolInfoPB {
        name: "Marker (精准 PDF 导入)".to_string(),
        status: ImportToolStatusPB::ToolUnknown,
        version: None,
        path: None,
        install_instruction: Some("Marker 工具应随应用包一起提供，请重新安装应用。".to_string()),
        description: "用于精准 PDF 导入的工具，将 PDF 转换为 Markdown 格式，保留格式和结构".to_string(),
      };
    }
  };
  
  tracing::debug!("[工具检查] 当前可执行文件路径: {}", exe_path.display());
  
  // 根据平台确定 Marker 工具路径
  let marker_path = resolve_marker_path(&exe_path);
  
  match marker_path {
    Ok(path) => {
      // 验证 Marker 工具是否存在且可执行
      match verify_marker_path(&path) {
        Ok(()) => {
          tracing::info!("[工具检查] Marker 工具验证成功: {}", path.display());
          
          // 尝试获取版本信息（如果 marker 支持 --version 或 -v）
          let version = get_marker_version(&path);
          
          ImportToolInfoPB {
            name: "Marker (精准 PDF 导入)".to_string(),
            status: ImportToolStatusPB::ToolAvailable,
            version,
            path: Some(path.display().to_string()),
            install_instruction: None,
            description: "用于精准 PDF 导入的工具，将 PDF 转换为 Markdown 格式，保留格式和结构".to_string(),
          }
        }
        Err(e) => {
          tracing::warn!("[工具检查] Marker 工具验证失败: {}", e);
          ImportToolInfoPB {
            name: "Marker (精准 PDF 导入)".to_string(),
            status: ImportToolStatusPB::ToolUnavailable,
            version: None,
            path: Some(path.display().to_string()),
            install_instruction: Some(format!("Marker 工具存在但无法使用: {}\n请检查文件权限或重新安装应用。", e)),
            description: "用于精准 PDF 导入的工具，将 PDF 转换为 Markdown 格式，保留格式和结构".to_string(),
          }
        }
      }
    }
    Err(e) => {
      tracing::warn!("[工具检查] Marker 工具未找到: {}", e);
      ImportToolInfoPB {
        name: "Marker (精准 PDF 导入)".to_string(),
        status: ImportToolStatusPB::ToolNotInstalled,
        version: None,
        path: None,
        install_instruction: Some(format!(
          "Marker 工具未找到: {}\n\
          Marker 工具应随应用包一起提供。\n\
          macOS 期望路径: AppFlowy.app/Contents/Resources/marker/marker\n\
          Windows 期望路径: AppFlowy/Resources/marker/marker.exe\n\
          请重新安装应用或检查应用包完整性。",
          e
        )),
        description: "用于精准 PDF 导入的工具，将 PDF 转换为 Markdown 格式，保留格式和结构".to_string(),
      }
    }
  }
}

/// 解析 Marker 工具路径
fn resolve_marker_path(exe_path: &Path) -> Result<std::path::PathBuf, String> {
  #[cfg(target_os = "macos")]
  {
    // macOS 应用包结构：
    // AppFlowy.app/
    // └── Contents/
    //     ├── MacOS/AppFlowy (可执行文件)
    //     └── Resources/
    //         └── marker/
    //             └── marker (Marker 工具)
    
    let mut current = exe_path.to_path_buf();
    
    // 向上查找 .app 包
    while let Some(parent) = current.parent() {
      // 检查是否是 Contents 目录
      if parent.file_name().and_then(|n| n.to_str()) == Some("Contents") {
        // 检查父目录是否是 .app 包
        if let Some(grandparent) = parent.parent() {
          if grandparent.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.ends_with(".app"))
            .unwrap_or(false)
          {
            // 构建 Resources/marker/marker 路径
            let marker_path = parent.join("Resources").join("marker").join("marker");
            tracing::debug!("[工具检查] 构建的 macOS Marker 路径: {}", marker_path.display());
            return Ok(marker_path);
          }
        }
      }
      
      // 检查当前路径是否是 .app 包
      if current.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.ends_with(".app"))
        .unwrap_or(false)
      {
        // 构建 Contents/Resources/marker/marker 路径
        let marker_path = current.join("Contents").join("Resources").join("marker").join("marker");
        tracing::debug!("[工具检查] 构建的 macOS Marker 路径: {}", marker_path.display());
        return Ok(marker_path);
      }
      
      current = parent.to_path_buf();
      
      // 防止无限循环
      if current == std::path::PathBuf::from("/") {
        break;
      }
    }
    
    // 如果无法从 .app 包结构推断，尝试从可执行文件路径直接构建
    if let Some(parent) = exe_path.parent() {
      if parent.file_name().and_then(|n| n.to_str()) == Some("MacOS") {
        if let Some(contents_dir) = parent.parent() {
          let marker_path = contents_dir.join("Resources").join("marker").join("marker");
          tracing::debug!("[工具检查] 从 MacOS 目录推断的 Marker 路径: {}", marker_path.display());
          return Ok(marker_path);
        }
      }
    }
    
    Err(format!(
      "无法从可执行文件路径推断 macOS 应用包路径: {}\n\
      期望的应用包结构: AppFlowy.app/Contents/Resources/marker/marker",
      exe_path.display()
    ))
  }
  
  #[cfg(target_os = "windows")]
  {
    // Windows 应用目录结构：
    // AppFlowy/
    // ├── AppFlowy.exe (可执行文件)
    // └── Resources/
    //     └── marker/
    //         └── marker.exe (Marker 工具)
    
    let app_dir = exe_path.parent().ok_or_else(|| {
      format!("无法获取可执行文件的父目录: {}", exe_path.display())
    })?;
    
    // 构建 Resources/marker/marker.exe 路径
    let marker_path = app_dir.join("Resources").join("marker").join("marker.exe");
    tracing::debug!("[工具检查] 构建的 Windows Marker 路径: {}", marker_path.display());
    Ok(marker_path)
  }
  
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    Err(format!("当前平台 {} 不支持 Marker 工具查找", std::env::consts::OS))
  }
}

/// 验证 Marker 工具路径
fn verify_marker_path(path: &Path) -> Result<(), String> {
  // 检查文件是否存在
  if !path.exists() {
    return Err(format!(
      "Marker 工具未找到: {}\n\
      请确保 Marker 工具已正确打包到应用包中。",
      path.display()
    ));
  }
  
  // 检查是否是文件（而不是目录）
  if !path.is_file() {
    return Err(format!(
      "Marker 工具路径指向的不是文件: {}\n\
      请检查应用包中的 Marker 工具是否正确安装。",
      path.display()
    ));
  }
  
  // 在 Unix 系统上检查执行权限
  #[cfg(unix)]
  {
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;
    
    let metadata = path.metadata().map_err(|e| {
      format!("无法获取文件元数据 {}: {}", path.display(), e)
    })?;
    
    let permissions = metadata.permissions();
    let mode = permissions.mode();
    
    // 检查是否有执行权限（用户、组或其他）
    if mode & 0o111 == 0 {
      tracing::warn!(
        "[工具检查] Marker 工具可能没有执行权限: {} (权限: {:o})",
        path.display(),
        mode
      );
      // 尝试添加执行权限
      if let Err(e) = std::fs::set_permissions(path, Permissions::from_mode(mode | 0o111)) {
        return Err(format!(
          "Marker 工具没有执行权限，且无法添加执行权限: {}: {}",
          path.display(),
          e
        ));
      }
      tracing::info!("[工具检查] 已为 Marker 工具添加执行权限: {}", path.display());
    }
  }
  
  tracing::debug!("[工具检查] Marker 工具验证成功: {}", path.display());
  Ok(())
}

/// 获取 Marker 工具版本信息
fn get_marker_version(marker_path: &Path) -> Option<String> {
  use std::process::Command;
  
  // 尝试使用 --version 参数
  if let Ok(output) = Command::new(marker_path).arg("--version").output() {
    if output.status.success() {
      let version = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();
      if !version.is_empty() {
        return Some(version);
      }
    }
  }
  
  // 尝试使用 -v 参数
  if let Ok(output) = Command::new(marker_path).arg("-v").output() {
    if output.status.success() {
      let version = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();
      if !version.is_empty() {
        return Some(version);
      }
    }
  }
  
  None
}
