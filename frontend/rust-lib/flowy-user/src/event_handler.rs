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
  
  // 检查 Marker 工具（精准 PDF 导入工具）
  // 其他工具（poppler、tesseract、python、pdfminer 等）已不再需要
  let marker_info = check_marker_tool();
  tools.push(marker_info);
  
  // 检查 marker-pdf（实际执行转换的 Python 工具）
  let marker_pdf_info = check_marker_pdf_tool();
  tools.push(marker_pdf_info);
  
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
    model_status: None,
    check_logs: Vec::new(),
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
    model_status: None,
    check_logs: Vec::new(),
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
        model_status: None,
        check_logs: vec!["无法获取当前可执行文件路径".to_string()],
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
          tracing::info!("[工具检查] Marker 脚本验证成功: {}", path.display());
          
          // 检查 marker-pdf 是否真正安装（这是实际执行转换的工具）
          match check_marker_pdf_installed() {
            Ok(()) => {
              tracing::info!("[工具检查] marker-pdf 已安装，工具可用");
              
              // 尝试获取版本信息（如果 marker 支持 --version 或 -v）
              let mut marker_logs = Vec::new();
              marker_logs.push("开始检查 Marker 工具...".to_string());
              marker_logs.push(format!("✓ Marker 脚本验证成功: {}", path.display()));
              marker_logs.push("检查 marker-pdf 是否已安装...".to_string());
              marker_logs.push("✓ marker-pdf 已安装，工具可用".to_string());
              
              let version = get_marker_version(&path);
              if let Some(ref v) = version {
                marker_logs.push(format!("✓ 获取版本信息: {}", v));
              } else {
                marker_logs.push("⚠ 无法获取版本信息".to_string());
              }
              
              // 检查模型就绪度
              let (model_status, model_logs) = check_marker_models_ready_with_logs();
              marker_logs.extend(model_logs);
              
              let description = "用于精准 PDF 导入的工具，将 PDF 转换为 Markdown 格式，保留格式和结构".to_string();
              
              ImportToolInfoPB {
                name: "Marker (精准 PDF 导入)".to_string(),
                status: ImportToolStatusPB::ToolAvailable,
                version,
                path: Some(path.display().to_string()),
                install_instruction: None,
                description,
                model_status: Some(model_status),
                check_logs: marker_logs,
              }
            }
            Err(install_instruction) => {
              tracing::warn!("[工具检查] marker-pdf 未安装: {}", install_instruction);
              
              let mut marker_logs = Vec::new();
              marker_logs.push("开始检查 Marker 工具...".to_string());
              marker_logs.push(format!("✓ Marker 脚本验证成功: {}", path.display()));
              marker_logs.push("检查 marker-pdf 是否已安装...".to_string());
              marker_logs.push(format!("✗ marker-pdf 未安装: {}", install_instruction));
              
              ImportToolInfoPB {
                name: "Marker (精准 PDF 导入)".to_string(),
                status: ImportToolStatusPB::ToolNotInstalled,
                version: None,
                path: Some(path.display().to_string()),
                install_instruction: Some(install_instruction),
                description: "用于精准 PDF 导入的工具，将 PDF 转换为 Markdown 格式，保留格式和结构".to_string(),
                model_status: None,
                check_logs: marker_logs,
              }
            }
          }
        }
        Err(e) => {
          tracing::warn!("[工具检查] Marker 工具验证失败: {}", e);
          let mut marker_logs = Vec::new();
          marker_logs.push("开始检查 Marker 工具...".to_string());
          marker_logs.push(format!("✗ Marker 工具验证失败: {}", e));
          
          ImportToolInfoPB {
            name: "Marker (精准 PDF 导入)".to_string(),
            status: ImportToolStatusPB::ToolUnavailable,
            version: None,
            path: Some(path.display().to_string()),
            install_instruction: Some(format!("Marker 工具存在但无法使用: {}\n请检查文件权限或重新安装应用。", e)),
            description: "用于精准 PDF 导入的工具，将 PDF 转换为 Markdown 格式，保留格式和结构".to_string(),
            model_status: None,
            check_logs: marker_logs,
          }
        }
      }
    }
    Err(e) => {
      tracing::warn!("[工具检查] Marker 工具未找到: {}", e);
      let mut marker_logs = Vec::new();
      marker_logs.push("开始检查 Marker 工具...".to_string());
      marker_logs.push(format!("✗ Marker 工具未找到: {}", e));
      
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
        model_status: None,
        check_logs: marker_logs,
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
    //         └── marker.exe 或 marker.bat (Marker 工具)
    
    let app_dir = exe_path.parent().ok_or_else(|| {
      format!("无法获取可执行文件的父目录: {}", exe_path.display())
    })?;
    
    // 首先尝试 marker.exe
    let marker_exe_path = app_dir.join("Resources").join("marker").join("marker.exe");
    if marker_exe_path.exists() {
      tracing::debug!("[工具检查] 构建的 Windows Marker 路径: {}", marker_exe_path.display());
      return Ok(marker_exe_path);
    }
    
    // 如果 marker.exe 不存在，尝试 marker.bat
    let marker_bat_path = app_dir.join("Resources").join("marker").join("marker.bat");
    if marker_bat_path.exists() {
      tracing::debug!("[工具检查] 构建的 Windows Marker 路径: {}", marker_bat_path.display());
      return Ok(marker_bat_path);
    }
    
    // 如果都不存在，返回 marker.exe 路径（用于错误提示）
    tracing::debug!("[工具检查] 构建的 Windows Marker 路径: {}", marker_exe_path.display());
    Ok(marker_exe_path)
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

/// 检查 marker-pdf 是否真正安装
/// 
/// marker 脚本只是一个包装器，实际执行转换需要 marker-pdf（通过 pipx 安装）。
/// 此函数检查 marker-pdf 是否已安装，支持 macOS 和 Windows 平台。
fn check_marker_pdf_installed() -> Result<(), String> {
  // 根据平台确定 marker-pdf 的安装路径
  let marker_pdf_path = get_marker_pdf_path()?;
  
  tracing::debug!("[工具检查] 检查 marker-pdf 路径: {}", marker_pdf_path.display());
  
  // 检查 marker-pdf 是否存在
  if marker_pdf_path.exists() && marker_pdf_path.is_file() {
    tracing::info!("[工具检查] marker-pdf 已安装: {}", marker_pdf_path.display());
    return Ok(());
  }
  
  // marker-pdf 未安装，生成安装指引
  let install_instruction = get_marker_pdf_install_instruction()?;
  Err(install_instruction)
}

/// 获取 marker-pdf 的安装路径（根据平台）
fn get_marker_pdf_path() -> Result<std::path::PathBuf, String> {
  #[cfg(target_os = "macos")]
  {
    // macOS: ~/.local/pipx/venvs/marker-pdf/bin/marker_single
    let home_dir = std::env::var("HOME").map_err(|_| "无法获取 HOME 目录".to_string())?;
    Ok(std::path::Path::new(&home_dir)
      .join(".local")
      .join("pipx")
      .join("venvs")
      .join("marker-pdf")
      .join("bin")
      .join("marker_single"))
  }
  
  #[cfg(target_os = "windows")]
  {
    // Windows: %LOCALAPPDATA%\pipx\venvs\marker-pdf\Scripts\marker_single.exe
    // 或者: %USERPROFILE%\.local\pipx\venvs\marker-pdf\Scripts\marker_single.exe
    let localappdata = std::env::var("LOCALAPPDATA")
      .or_else(|_| std::env::var("USERPROFILE").map(|p| format!("{}\\AppData\\Local", p)))
      .map_err(|_| "无法获取 LOCALAPPDATA 或 USERPROFILE 目录".to_string())?;
    
    // 首先尝试 LOCALAPPDATA
    let path1 = std::path::Path::new(&localappdata)
      .join("pipx")
      .join("venvs")
      .join("marker-pdf")
      .join("Scripts")
      .join("marker_single.exe");
    
    if path1.exists() {
      return Ok(path1);
    }
    
    // 尝试 USERPROFILE\.local\pipx
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
      let path2 = std::path::Path::new(&userprofile)
        .join(".local")
        .join("pipx")
        .join("venvs")
        .join("marker-pdf")
        .join("Scripts")
        .join("marker_single.exe");
      
      if path2.exists() {
        return Ok(path2);
      }
    }
    
    // 返回默认路径（用于检查）
    Ok(path1)
  }
  
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    Err(format!("当前平台 {} 不支持 marker-pdf 检查", std::env::consts::OS))
  }
}

/// 获取 marker-pdf 的安装指引（根据平台）
fn get_marker_pdf_install_instruction() -> Result<String, String> {
  use std::process::Command;
  
  #[cfg(target_os = "macos")]
  {
    // 检测是否安装了 Homebrew
    let brew_available = Command::new("which")
      .arg("brew")
      .output()
      .map(|output| output.status.success())
      .unwrap_or(false);
    
    if !brew_available {
      return Ok(format!(
        "marker-pdf 未安装。\n\n\
        Marker 工具需要使用 Homebrew 来安装 marker-pdf。\n\n\
        请先安装 Homebrew：\n\
        1. 打开终端，运行：\n\
           /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"\n\n\
        2. 或者访问 https://brew.sh 查看安装说明\n\n\
        3. 安装 Homebrew 后，运行：\n\
           brew install jpeg libpng freetype openjpeg libtiff webp\n\
           brew install pipx\n\
           pipx install marker-pdf"
      ));
    }
    
    Ok(format!(
      "marker-pdf 未安装。\n\n\
      安装方法（使用 Homebrew）：\n\
      1. 首先安装 Pillow 编译所需的依赖库：\n\
         brew install jpeg libpng freetype openjpeg libtiff webp\n\
      2. 安装 pipx：\n\
         brew install pipx\n\
      3. 安装 marker-pdf：\n\
         pipx install marker-pdf\n\n\
      验证安装：\n\
      安装完成后，运行以下命令验证：\n\
      pipx list  # 应该看到 marker-pdf"
    ))
  }
  
  #[cfg(target_os = "windows")]
  {
    // Windows 下检查 pipx 是否可用
    let pipx_available = Command::new("where")
      .arg("pipx")
      .output()
      .map(|output| output.status.success())
      .unwrap_or(false);
    
    // 检查 Python 是否可用
    let python_available = Command::new("where")
      .arg("python")
      .output()
      .map(|output| output.status.success())
      .unwrap_or(false);
    
    if !python_available {
      return Ok(format!(
        "marker-pdf 未安装。\n\n\
        首先需要安装 Python 3.8+：\n\n\
        方法 1：使用 WinGet 安装（推荐，Windows 10/11 自带）\n\
          winget install Python.Python.3.12\n\n\
        方法 2：手动安装\n\
        1. 访问 https://www.python.org/downloads/ 下载并安装 Python\n\
        2. 安装时勾选 \"Add Python to PATH\"\n\n\
        安装 Python 后，再安装 marker-pdf：\n\
        1. 打开命令提示符或 PowerShell\n\
        2. 运行: pip install --user pipx\n\
        3. 运行: pipx install marker-pdf"
      ));
    }
    
    if !pipx_available {
      return Ok(format!(
        "marker-pdf 未安装。\n\n\
        pipx 未安装。请先安装 pipx：\n\n\
        方法 1：使用 WinGet 安装（推荐，Windows 10/11 自带）\n\
        注意：WinGet 会自动安装 Python 作为 pipx 的依赖\n\
          winget install pipx\n\n\
        方法 2：使用 pip 安装（需要先安装 Python）\n\
        1. 打开命令提示符或 PowerShell\n\
        2. 运行: pip install --user pipx\n\
        3. 将 pipx 添加到 PATH（如果尚未添加）：\n\
           - 添加到用户 PATH: %USERPROFILE%\\AppData\\Roaming\\Python\\Python3X\\Scripts\n\
           - 或添加到用户 PATH: %USERPROFILE%\\.local\\bin\n\n\
        安装 pipx 后，运行: pipx install marker-pdf\n\n\
        验证安装：\n\
        安装完成后，运行: pipx list  # 应该看到 marker-pdf"
      ));
    }
    
    Ok(format!(
      "marker-pdf 未安装。\n\n\
      安装方法：\n\
      1. 打开命令提示符或 PowerShell（以管理员身份运行）\n\
      2. 运行: pipx install marker-pdf\n\n\
      注意：Windows 下 Pillow 使用预编译包，通常不需要手动安装依赖库。\n\n\
      验证安装：\n\
      安装完成后，运行以下命令验证：\n\
      pipx list  # 应该看到 marker-pdf"
    ))
  }
  
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    Err(format!("当前平台 {} 不支持 marker-pdf 安装指引", std::env::consts::OS))
  }
}

/// 检查 marker-pdf 模型就绪度（带日志）
/// 
/// 检查 PDF 识别所需的所有模型是否已下载并可用。
/// 返回模型就绪度状态描述和检查日志。
fn check_marker_models_ready_with_logs() -> (String, Vec<String>) {
  let mut logs = Vec::new();
  logs.push("开始检查模型就绪度...".to_string());
  tracing::info!("[工具检查] 开始检查 marker-pdf 模型就绪度");
  
  // 获取缓存目录路径
  let (hf_cache_dir, surya_cache_dir) = get_model_cache_dirs();
  logs.push(format!("Hugging Face 缓存目录: {}", hf_cache_dir));
  logs.push(format!("Surya OCR 缓存目录: {}", surya_cache_dir));
  
  let mut hf_models_found = 0;
  let mut surya_models_found = 0;
  let mut hf_size = 0u64;
  let mut surya_size = 0u64;
  
  // 递归检查目录中的模型文件
  fn count_model_files(dir: &std::path::Path, count: &mut u32, size: &mut u64, max_depth: u32) {
    use std::fs;
    
    if max_depth == 0 {
      return;
    }
    
    if let Ok(entries) = fs::read_dir(dir) {
      for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
          if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            // 检查是否是模型文件
            if ext_str == "safetensors" || ext_str == "bin" || ext_str == "pt" || ext_str == "pth" || ext_str == "onnx" {
              *count += 1;
              if let Ok(metadata) = fs::metadata(&path) {
                *size += metadata.len();
              }
            }
          }
        } else if path.is_dir() {
          // 递归检查子目录（限制深度避免过深）
          count_model_files(&path, count, size, max_depth - 1);
        }
      }
    }
  }
  
  // 检查 Hugging Face 模型
  logs.push("检查 Hugging Face 模型...".to_string());
  let hf_path = std::path::PathBuf::from(&hf_cache_dir);
  if hf_path.exists() {
    logs.push(format!("✓ Hugging Face 缓存目录存在: {}", hf_path.display()));
    
    // 首先检查 hub 目录（Hugging Face 模型的主要存储位置）
    let hub_path = hf_path.join("hub");
    if hub_path.exists() {
      logs.push(format!("✓ hub 目录存在: {}", hub_path.display()));
      count_model_files(&hub_path, &mut hf_models_found, &mut hf_size, 5);
      logs.push(format!("  在 hub 目录中找到 {} 个模型文件，总大小: {:.2} GB", hf_models_found, hf_size as f64 / (1024.0 * 1024.0 * 1024.0)));
    } else {
      logs.push(format!("⚠ hub 目录不存在: {} (这是正常的，首次使用时会自动创建)", hub_path.display()));
    }
    
    // 也检查 transformers 目录（Transformers 库的缓存）
    let transformers_path = hf_path.join("transformers");
    if transformers_path.exists() {
      logs.push(format!("✓ transformers 目录存在: {}", transformers_path.display()));
      let mut tf_models = 0u32;
      let mut tf_size = 0u64;
      count_model_files(&transformers_path, &mut tf_models, &mut tf_size, 5);
      if tf_models > 0 {
        hf_models_found += tf_models;
        hf_size += tf_size;
        logs.push(format!("  在 transformers 目录中找到 {} 个模型文件，总大小: {:.2} GB", tf_models, tf_size as f64 / (1024.0 * 1024.0 * 1024.0)));
      }
    }
    
    // 如果 hub 和 transformers 目录都不存在或没有找到模型，检查整个缓存目录
    // （某些情况下模型可能存储在其他位置）
    if hf_models_found == 0 {
      logs.push("  在 hub 和 transformers 目录中未找到模型，检查整个缓存目录...".to_string());
      let mut all_models = 0u32;
      let mut all_size = 0u64;
      count_model_files(&hf_path, &mut all_models, &mut all_size, 3);
      if all_models > 0 {
        hf_models_found = all_models;
        hf_size = all_size;
        logs.push(format!("  在整个缓存目录中找到 {} 个模型文件，总大小: {:.2} GB", hf_models_found, hf_size as f64 / (1024.0 * 1024.0 * 1024.0)));
      } else {
        logs.push("  在整个缓存目录中未找到模型文件".to_string());
        logs.push("  说明: 首次运行 PDF 转换时会自动下载模型到 hub 目录".to_string());
      }
    }
    
    if hf_models_found > 0 {
      logs.push(format!("✓ Hugging Face 模型总计: {} 个文件 ({:.2} GB)", hf_models_found, hf_size as f64 / (1024.0 * 1024.0 * 1024.0)));
    }
  } else {
    logs.push(format!("✗ Hugging Face 缓存目录不存在: {}", hf_path.display()));
    logs.push("  说明: 首次运行 PDF 转换时会自动创建此目录并下载模型".to_string());
  }
  
  // 检查 Surya OCR 模型
  // 基于本机检查结果，定义必需的 marker 模型文件列表
  // 这些是 marker-pdf 正常工作所需的核心模型
  struct RequiredModel {
    path: &'static str,
    min_size_bytes: u64,  // 最小文件大小（允许一定误差）
    description: &'static str,
  }
  
  let required_models: Vec<RequiredModel> = vec![
    RequiredModel {
      path: "layout/2025_09_23/model.safetensors",
      min_size_bytes: 1_400_000_000,  // 约 1.3GB，允许误差
      description: "布局识别模型",
    },
    RequiredModel {
      path: "text_recognition/2025_09_23/model.safetensors",
      min_size_bytes: 1_400_000_000,  // 约 1.3GB，允许误差
      description: "文本识别模型",
    },
    RequiredModel {
      path: "ocr_error_detection/2025_02_18/model.safetensors",
      min_size_bytes: 250_000_000,  // 约 258MB，允许误差
      description: "OCR 错误检测模型",
    },
    RequiredModel {
      path: "table_recognition/2025_02_18/model.safetensors",
      min_size_bytes: 200_000_000,  // 约 201MB，允许误差
      description: "表格识别模型",
    },
    RequiredModel {
      path: "text_detection/2025_05_07/model.safetensors",
      min_size_bytes: 70_000_000,  // 约 73MB，允许误差
      description: "文本检测模型",
    },
  ];
  
  logs.push("检查 Surya OCR 模型...".to_string());
  let surya_path = std::path::PathBuf::from(&surya_cache_dir);
  
  // 先初始化变量，用于在判断模型是否就绪时使用
  let mut found_required_models_count = 0u32;
  
  if surya_path.exists() {
    logs.push(format!("✓ Surya OCR 缓存目录存在: {}", surya_path.display()));
    
    // 检查必需的模型文件
    let mut missing_models = Vec::new();
    let mut found_model_details = Vec::new();
    
    for model in &required_models {
      let model_path = surya_path.join(model.path);
      if model_path.exists() {
        if let Ok(metadata) = std::fs::metadata(&model_path) {
          let file_size = metadata.len();
          if file_size >= model.min_size_bytes {
            found_required_models_count += 1;
            found_model_details.push(format!(
              "  ✓ {}: {:.2} GB ({})",
              model.description,
              file_size as f64 / (1024.0 * 1024.0 * 1024.0),
              model.path
            ));
            surya_models_found += 1;
            surya_size += file_size;
          } else {
            missing_models.push(format!(
              "  ✗ {}: 文件存在但大小不足 ({} < {} 字节)",
              model.description,
              file_size,
              model.min_size_bytes
            ));
          }
        } else {
          missing_models.push(format!("  ✗ {}: 无法读取文件元数据 ({})", model.description, model.path));
        }
      } else {
        missing_models.push(format!("  ✗ {}: 文件不存在 ({})", model.description, model.path));
      }
    }
    
    // 也统计所有模型文件（包括非必需的）
    let mut all_models_count = 0u32;
    let mut all_models_size = 0u64;
    count_model_files(&surya_path, &mut all_models_count, &mut all_models_size, 5);
    
    logs.push(format!("必需模型检查: {}/{} 个模型已就绪", found_required_models_count, required_models.len()));
    for detail in &found_model_details {
      logs.push(detail.clone());
    }
    if !missing_models.is_empty() {
      logs.push("缺失的模型:".to_string());
      for missing in &missing_models {
        logs.push(missing.clone());
      }
    }
    logs.push(format!("总计: {} 个模型文件，总大小: {:.2} GB", all_models_count, all_models_size as f64 / (1024.0 * 1024.0 * 1024.0)));
  } else {
    logs.push(format!("✗ Surya OCR 缓存目录不存在: {}", surya_path.display()));
  }
  
  let total_models = hf_models_found + surya_models_found;
  let total_size = hf_size + surya_size;
  let size_gb = total_size as f64 / (1024.0 * 1024.0 * 1024.0);
  let hf_size_gb = hf_size as f64 / (1024.0 * 1024.0 * 1024.0);
  let surya_size_gb = surya_size as f64 / (1024.0 * 1024.0 * 1024.0);
  
  logs.push(format!("总计: {} 个模型文件，总大小: {:.2} GB", total_models, size_gb));
  
  tracing::info!(
    "[工具检查] 模型检查完成 - Hugging Face: {} 个 ({:.2} GB), Surya OCR: {} 个 ({:.2} GB), 总大小: {:.2} GB",
    hf_models_found,
    hf_size_gb,
    surya_models_found,
    surya_size_gb,
    size_gb
  );
  
  // 生成状态描述
  // 判断模型是否完整：基于必需的 Surya OCR 模型文件检查
  let is_hf_ready = hf_models_found > 0 && hf_size_gb > 0.1; // Hugging Face 模型至少 100MB
  
  // 检查必需的 Surya OCR 模型（5个核心模型）
  let required_surya_models = required_models.len();
  let is_surya_ready = found_required_models_count >= required_surya_models as u32 && surya_size_gb >= 2.5; // 至少 2.5GB（5个模型的总大小约 3.2GB）
  
  // 判断模型是否完整：
  // 1. 如果所有必需的 Surya OCR 模型都存在，认为已就绪
  // 2. 如果模型总大小 >= 3.0 GB，也认为已就绪（允许版本差异）
  // 3. 否则需要检查具体文件
  let is_fully_ready = if found_required_models_count >= required_surya_models as u32 && surya_size_gb >= 2.5 {
    // 所有必需的 Surya OCR 模型都已就绪
    true
  } else if size_gb >= 3.0 {
    // 模型总大小足够（约 3.2GB），认为已就绪（允许版本差异或额外模型）
    true
  } else {
    // 模型不完整，需要检查具体文件
    is_hf_ready && is_surya_ready && total_models >= 3 && size_gb >= 1.0
  };
  
  let status_msg = if total_models == 0 {
    format!(
      "模型状态: ⚠️ 未下载\n\
      - Hugging Face 模型: 未找到\n\
      - Surya OCR 模型: 未找到\n\
      - 缓存目录: {}\n\
      - 首次运行 PDF 转换时会自动下载模型（约 2-3GB，需要 10-30 分钟）",
      hf_cache_dir
    )
  } else if !is_fully_ready {
    // 模型不完整：缺少 Hugging Face 或 Surya OCR 模型，或数量/大小不足
    let hf_status = if is_hf_ready {
      format!("{} 个文件 ({:.2} GB)", hf_models_found, hf_size_gb)
    } else {
      "未找到".to_string()
    };
    let surya_status = if is_surya_ready {
      format!("{} 个文件 ({:.2} GB)", surya_models_found, surya_size_gb)
    } else {
      "未找到".to_string()
    };
    
    format!(
      "模型状态: ⚠️ 部分就绪\n\
      - Hugging Face 模型: {}\n\
      - Surya OCR 模型: {}\n\
      - 总大小: {:.2} GB\n\
      - 缓存目录: {}\n\
      - 模型不完整，首次运行 PDF 转换时可能需要下载更多模型",
      hf_status,
      surya_status,
      size_gb,
      hf_cache_dir
    )
  } else {
    format!(
      "模型状态: ✅ 已就绪\n\
      - Hugging Face 模型: {} 个文件 ({:.2} GB)\n\
      - Surya OCR 模型: {} 个文件 ({:.2} GB)\n\
      - 总大小: {:.2} GB\n\
      - 缓存目录: {}\n\
      - 所有必需的模型已下载，PDF 转换可直接使用",
      hf_models_found,
      hf_size_gb,
      surya_models_found,
      surya_size_gb,
      size_gb,
      hf_cache_dir
    )
  };
  
  (status_msg, logs)
}

/// 检查 marker-pdf 模型就绪度
/// 
/// 检查 PDF 识别所需的所有模型是否已下载并可用。
/// 返回模型就绪度状态描述。
fn check_marker_models_ready() -> String {
  let (status, _) = check_marker_models_ready_with_logs();
  status
}

/// 下载 Marker 模型
/// 
/// 通过执行 marker 命令处理一个测试 PDF 来触发模型下载。
/// 这是最可靠的方法，因为 marker-pdf 会在首次运行时自动下载所需的模型。
#[tracing::instrument(level = "info", skip_all, err)]
pub async fn download_marker_models(
  data: AFPluginData<DownloadMarkerModelsPB>,
) -> DataResult<ModelDownloadProgressPB, FlowyError> {
  use crate::entities::{ModelDownloadProgressPB, ModelDownloadStatusPB};
  use std::io::Write;
  use std::path::Path;
  use tempfile::{NamedTempFile, tempdir};
  use tokio::time::Duration;
  
  let request = data.into_inner();
  tracing::info!("[模型下载] 开始下载 Marker 模型，force: {}", request.force);
  
  // 检查 marker 工具是否可用
  let exe_path = match std::env::current_exe() {
    Ok(path) => path,
    Err(e) => {
      tracing::error!("[模型下载] 无法获取当前可执行文件路径: {}", e);
      return data_result_ok(ModelDownloadProgressPB {
        status: ModelDownloadStatusPB::ModelDownloadFailed,
        progress: 0.0,
        current_model: None,
        message: format!("无法获取当前可执行文件路径: {}", e),
        downloaded_bytes: 0,
        total_bytes: None,
      });
    }
  };
  
  let marker_path = match resolve_marker_path(&exe_path) {
    Ok(path) => path,
    Err(e) => {
      tracing::error!("[模型下载] Marker 工具不可用: {}", e);
      return data_result_ok(ModelDownloadProgressPB {
        status: ModelDownloadStatusPB::ModelDownloadFailed,
        progress: 0.0,
        current_model: None,
        message: format!("Marker 工具不可用: {}", e),
        downloaded_bytes: 0,
        total_bytes: None,
      });
    }
  };
  
  tracing::info!("[模型下载] Marker 工具路径: {}", marker_path.display());
  
  // 检查模型是否已存在（如果不强制下载）
  if !request.force {
    let (status_msg, _) = check_marker_models_ready_with_logs();
    if status_msg.contains("✅ 已就绪") {
      tracing::info!("[模型下载] 模型已存在，跳过下载");
      return data_result_ok(ModelDownloadProgressPB {
        status: ModelDownloadStatusPB::ModelDownloadCompleted,
        progress: 1.0,
        current_model: None,
        message: "模型已存在，无需下载".to_string(),
        downloaded_bytes: 0,
        total_bytes: None,
      });
    }
  }
  
  // 创建一个最小的测试 PDF 文件
  // PDF 文件头：%PDF-1.4\n
  // 这是一个最小的有效 PDF 文件
  let test_pdf_content = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>\nendobj\n4 0 obj\n<< /Length 44 >>\nstream\nBT\n/F1 12 Tf\n100 700 Td\n(Test) Tj\nET\nendstream\nendobj\nxref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000206 00000 n \ntrailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n300\n%%EOF";
  
  // 创建临时 PDF 文件
  // 重要：必须保持 temp_pdf_file 的生命周期，直到命令执行完成
  // 否则临时文件会被自动删除，导致 marker 工具无法找到文件
  let temp_pdf_file = match NamedTempFile::new() {
    Ok(mut file) => {
      if let Err(e) = file.write_all(test_pdf_content) {
        tracing::error!("[模型下载] 无法写入测试 PDF: {}", e);
        return data_result_ok(ModelDownloadProgressPB {
          status: ModelDownloadStatusPB::ModelDownloadFailed,
          progress: 0.0,
          current_model: None,
          message: format!("无法创建测试 PDF: {}", e),
          downloaded_bytes: 0,
          total_bytes: None,
        });
      }
      if let Err(e) = file.flush() {
        tracing::error!("[模型下载] 无法刷新测试 PDF: {}", e);
        return data_result_ok(ModelDownloadProgressPB {
          status: ModelDownloadStatusPB::ModelDownloadFailed,
          progress: 0.0,
          current_model: None,
          message: format!("无法刷新测试 PDF: {}", e),
          downloaded_bytes: 0,
          total_bytes: None,
        });
      }
      // 获取文件路径，但保持文件句柄的生命周期
      tracing::info!("[模型下载] 创建测试 PDF: {}", file.path().display());
      file
    },
    Err(e) => {
      tracing::error!("[模型下载] 无法创建临时文件: {}", e);
      return data_result_ok(ModelDownloadProgressPB {
        status: ModelDownloadStatusPB::ModelDownloadFailed,
        progress: 0.0,
        current_model: None,
        message: format!("无法创建临时文件: {}", e),
        downloaded_bytes: 0,
        total_bytes: None,
      });
    }
  };
  
  // 获取文件路径（文件会在 temp_pdf_file 被 drop 时删除）
  let temp_pdf = temp_pdf_file.path().to_path_buf();
  tracing::info!("[模型下载] 测试 PDF 路径: {}", temp_pdf.display());
  
  // 创建临时输出目录
  // 重要：必须保持 temp_output_dir 的生命周期，直到命令执行完成
  let temp_output_dir = match tempdir() {
    Ok(dir) => {
      let path = dir.path().to_path_buf();
      tracing::info!("[模型下载] 创建临时输出目录: {}", path.display());
      (path, dir)
    },
    Err(e) => {
      tracing::error!("[模型下载] 无法创建临时输出目录: {}", e);
      return data_result_ok(ModelDownloadProgressPB {
        status: ModelDownloadStatusPB::ModelDownloadFailed,
        progress: 0.0,
        current_model: None,
        message: format!("无法创建临时输出目录: {}", e),
        downloaded_bytes: 0,
        total_bytes: None,
      });
    }
  };
  let temp_output = temp_output_dir.0;
  
  // 设置环境变量（确保模型下载到正确的目录）
  let (hf_cache_dir, surya_cache_dir) = get_model_cache_dirs();
  
  // 记录初始模型文件大小（用于估算进度）
  let initial_hf_size = get_dir_size(&hf_cache_dir);
  let initial_surya_size = get_dir_size(&surya_cache_dir);
  
  // 执行 marker 命令来触发模型下载
  // 使用 tokio::process::Command 以便异步执行
  let mut cmd = tokio::process::Command::new(&marker_path);
  cmd.arg(&temp_pdf);
  cmd.arg("--output_dir");
  cmd.arg(&temp_output);
  cmd.arg("--output_format");
  cmd.arg("markdown");
  
  // 设置环境变量
  cmd.env("HF_HOME", &hf_cache_dir);
  cmd.env("HF_HUB_CACHE", &hf_cache_dir);
  cmd.env("HUGGINGFACE_HUB_CACHE", &hf_cache_dir);
  cmd.env("TRANSFORMERS_CACHE", &hf_cache_dir);
  cmd.env("SURYA_MODEL_CACHE_DIR", &surya_cache_dir);
  cmd.env("PYTORCH_ENABLE_MPS_FALLBACK", "1");
  cmd.env("PYTORCH_MPS_FORCE_CPU", "1");
  cmd.env("PYTORCH_MPS_DISABLE", "1");
  cmd.env("TORCH_DEVICE", "cpu");
  
  tracing::info!("[模型下载] 开始执行 marker 命令触发模型下载...");
  tracing::info!("[模型下载] Marker 工具路径: {}", marker_path.display());
  tracing::info!("[模型下载] 测试 PDF 路径: {}", temp_pdf.display());
  tracing::info!("[模型下载] 输出目录: {}", temp_output.display());
  tracing::info!("[模型下载] Hugging Face 缓存目录: {}", hf_cache_dir);
  tracing::info!("[模型下载] Surya OCR 缓存目录: {}", surya_cache_dir);
  tracing::info!("[模型下载] 初始模型大小 - HF: {} 字节, Surya: {} 字节", initial_hf_size, initial_surya_size);
  
  // 记录完整的命令（用于调试）
  tracing::debug!(
    "[模型下载] 执行命令: {} {} --output_dir {} --output_format markdown",
    marker_path.display(),
    temp_pdf.display(),
    temp_output.display()
  );
  
  // 立即返回初始进度信息（让用户知道下载已开始）
  // 注意：由于这是单次 API 调用，我们无法实时推送更新
  // 但可以在消息中包含详细的执行信息
  
  // 执行命令（设置较长的超时时间，因为首次下载模型可能需要 10-30 分钟）
  let timeout_duration = Duration::from_secs(1800); // 30 分钟
  let start_time = std::time::Instant::now();
  
  // 构建初始消息
  let initial_message = format!(
    "开始下载模型...\n\n执行信息:\n- Marker 工具路径: {}\n- Hugging Face 缓存: {}\n- Surya OCR 缓存: {}\n- 初始大小: HF={:.2} GB, Surya={:.2} GB\n\n正在执行 marker 命令，这可能需要 10-30 分钟...",
    marker_path.display(),
    hf_cache_dir,
    surya_cache_dir,
    initial_hf_size as f64 / (1024.0 * 1024.0 * 1024.0),
    initial_surya_size as f64 / (1024.0 * 1024.0 * 1024.0)
  );
  
  // 使用 spawn 以便能够流式读取输出
  cmd.stdout(std::process::Stdio::piped());
  cmd.stderr(std::process::Stdio::piped());
  
  let mut child = match cmd.spawn() {
    Ok(child) => child,
    Err(e) => {
      tracing::error!("[模型下载] 无法启动 marker 命令: {}", e);
      return data_result_ok(ModelDownloadProgressPB {
        status: ModelDownloadStatusPB::ModelDownloadFailed,
        progress: 0.0,
        current_model: None,
        message: format!("无法启动 marker 命令: {}", e),
        downloaded_bytes: 0,
        total_bytes: None,
      });
    }
  };
  
  // 获取 stdout 和 stderr
  let stdout = match child.stdout.take() {
    Some(stdout) => stdout,
    None => {
      tracing::error!("[模型下载] 无法获取 marker 命令的 stdout");
      return data_result_ok(ModelDownloadProgressPB {
        status: ModelDownloadStatusPB::ModelDownloadFailed,
        progress: 0.0,
        current_model: None,
        message: "无法获取 marker 命令的 stdout".to_string(),
        downloaded_bytes: 0,
        total_bytes: None,
      });
    }
  };
  let stderr = match child.stderr.take() {
    Some(stderr) => stderr,
    None => {
      tracing::error!("[模型下载] 无法获取 marker 命令的 stderr");
      return data_result_ok(ModelDownloadProgressPB {
        status: ModelDownloadStatusPB::ModelDownloadFailed,
        progress: 0.0,
        current_model: None,
        message: "无法获取 marker 命令的 stderr".to_string(),
        downloaded_bytes: 0,
        total_bytes: None,
      });
    }
  };
  
  // 收集输出日志
  let logs_arc = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
  let logs_clone_stdout = logs_arc.clone();
  let logs_clone_stderr = logs_arc.clone();
  
  // 在后台任务中读取 stdout
  let stdout_handle = tokio::spawn(async move {
    use tokio::io::AsyncBufReadExt;
    use tokio::io::BufReader;
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
      let trimmed = line.trim();
      if !trimmed.is_empty() {
        tracing::debug!("[模型下载 stdout] {}", trimmed);
        if let Ok(mut logs) = logs_clone_stdout.lock() {
          logs.push(format!("[Marker] {}", trimmed));
        }
      }
      line.clear();
    }
  });
  
  // 在后台任务中读取 stderr（通常包含错误和进度信息）
  let stderr_handle = tokio::spawn(async move {
    use tokio::io::AsyncBufReadExt;
    use tokio::io::BufReader;
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();
    while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
      let trimmed = line.trim();
      if !trimmed.is_empty() {
        // stderr 通常包含错误信息，使用 error 级别记录
        tracing::error!("[模型下载 stderr] {}", trimmed);
        if let Ok(mut logs) = logs_clone_stderr.lock() {
          logs.push(format!("[Marker Error] {}", trimmed));
        }
      }
      line.clear();
    }
  });
  
  // 等待命令完成
  let wait_result = tokio::time::timeout(timeout_duration, child.wait()).await;
  
  // 等待输出读取完成
  let _ = tokio::join!(stdout_handle, stderr_handle);
  
  // 获取收集的日志
  let output_logs: Vec<String> = if let Ok(logs) = logs_arc.lock() {
    logs.clone()
  } else {
    Vec::new()
  };
  
  match wait_result {
    Ok(Ok(status)) => {
      let elapsed = start_time.elapsed();
      tracing::info!("[模型下载] marker 命令执行完成，耗时: {:?}", elapsed);
      
      // 检查模型文件大小变化
      let mut current_hf_size = get_dir_size(&hf_cache_dir);
      let mut current_surya_size = get_dir_size(&surya_cache_dir);
      let mut downloaded_hf = current_hf_size.saturating_sub(initial_hf_size);
      let mut downloaded_surya = current_surya_size.saturating_sub(initial_surya_size);
      let mut total_downloaded = downloaded_hf + downloaded_surya;
      
      if status.success() {
        // marker 命令执行成功，但模型可能还在下载中
        // 等待一小段时间，让模型下载完成（如果还在进行中）
        if total_downloaded == 0 && initial_hf_size == 0 {
          // 如果初始时没有模型，且下载后仍然没有新文件，等待一下再检查
          tracing::info!("[模型下载] 检测到可能正在下载模型，等待 3 秒后重新检查...");
          tokio::time::sleep(Duration::from_secs(3)).await;
          
          // 重新检查模型文件大小
          current_hf_size = get_dir_size(&hf_cache_dir);
          current_surya_size = get_dir_size(&surya_cache_dir);
          downloaded_hf = current_hf_size.saturating_sub(initial_hf_size);
          downloaded_surya = current_surya_size.saturating_sub(initial_surya_size);
          total_downloaded = downloaded_hf + downloaded_surya;
          
          if total_downloaded > 0 {
            tracing::info!("[模型下载] 检测到新的模型文件下载: {} 字节", total_downloaded);
          }
        }
        
        // 检查模型是否已下载
        let (status_msg, logs) = check_marker_models_ready_with_logs();
        let models_ready = status_msg.contains("✅ 已就绪");
        
        // 获取当前模型总大小（用于更准确的进度计算）
        let current_hf_size = get_dir_size(&hf_cache_dir);
        let current_surya_size = get_dir_size(&surya_cache_dir);
        let current_total_size = current_hf_size + current_surya_size;
        
        // 计算进度：基于模型文件大小变化和模型就绪状态
        let progress = if models_ready {
          1.0
        } else if total_downloaded > 0 {
          // 如果有下载，但未完全就绪，根据下载量估算进度
          // 假设完整模型需要约 2-3 GB，根据已下载量估算
          let estimated_total = 2_500_000_000u64; // 2.5 GB
          let calculated_progress = total_downloaded as f64 / estimated_total as f64;
          // 如果下载量超过预期，进度可以超过 1.0，但未完全就绪时限制为 0.99
          calculated_progress.min(0.99)
        } else {
          // 如果没有检测到下载，但命令执行成功，可能是模型已存在
          // 基于当前模型总大小计算进度
          let estimated_total = 2_500_000_000u64; // 2.5 GB
          if current_total_size > 0 {
            // 基于实际模型大小计算进度
            let calculated_progress = current_total_size as f64 / estimated_total as f64;
            // 如果模型大小超过预期，进度可以超过 1.0，但未完全就绪时限制为 0.99
            let size_based_progress = calculated_progress.min(0.99);
            // 如果模型大小 >= 1.0 GB，认为至少完成了 80%
            // 如果模型大小 >= 0.5 GB，认为至少完成了 60%
            // 否则基于大小比例计算
            if current_total_size >= 1_000_000_000 {
              size_based_progress.max(0.8)
            } else if current_total_size >= 500_000_000 {
              size_based_progress.max(0.6)
            } else {
              size_based_progress.max(0.3)
            }
          } else {
            // 没有模型文件，可能下载失败或模型已存在但检查逻辑有问题
            // 但命令执行成功，说明工具可用，可能是模型在其他位置或已存在
            0.5
          }
        };
        
        // 构建详细的消息
        // 显示当前模型总大小（如果本次没有下载，显示已存在的模型大小）
        let display_size = if total_downloaded > 0 {
          total_downloaded
        } else {
          current_total_size
        };
        let mut message = format!(
          "模型下载完成（耗时: {:.1} 秒）\n当前模型大小: {:.2} GB\n进度: {:.0}%\n\n执行日志:\n",
          elapsed.as_secs_f64(),
          display_size as f64 / (1024.0 * 1024.0 * 1024.0),
          progress * 100.0
        );
        
        // 添加最近的日志（最多 20 行）
        let recent_logs: Vec<String> = output_logs.iter().rev().take(20).rev().cloned().collect();
        if !recent_logs.is_empty() {
          message.push_str(&recent_logs.join("\n"));
        } else {
          message.push_str("（Marker 工具未输出日志）");
        }
        
        // 使用当前模型总大小作为已下载字节数（更准确反映实际情况）
        let reported_bytes = if total_downloaded > 0 {
          total_downloaded
        } else {
          current_total_size
        };
        
        if models_ready {
          tracing::info!("[模型下载] 模型下载成功");
          data_result_ok(ModelDownloadProgressPB {
            status: ModelDownloadStatusPB::ModelDownloadCompleted,
            progress: 1.0,
            current_model: None,
            message,
            downloaded_bytes: reported_bytes,
            total_bytes: Some(reported_bytes.max(1)),
          })
        } else {
          // 模型未完全就绪，根据进度和实际情况决定状态
          // 判断是否有实际的下载活动：如果 total_downloaded 很小（< 100MB），说明可能是模型已存在
          let has_actual_download = total_downloaded > 100_000_000; // 100 MB
          
          // 如果 marker 命令执行完成，且没有检测到新的下载活动，说明下载已经停止
          // 此时不应该显示"正在下载"，而应该根据实际情况显示状态
          let final_status = if !has_actual_download && current_total_size >= 2_000_000_000 {
            // 没有新的下载活动，但模型大小 >= 2.0 GB，说明模型已存在且完整
            // 即使检查显示不完整，如果大小足够，应该认为模型是完整的（因为 marker 可以正常工作）
            tracing::info!("[模型下载] marker 命令执行完成，模型已存在（大小: {:.2} GB），认为模型已就绪", 
              current_total_size as f64 / (1024.0 * 1024.0 * 1024.0));
            ModelDownloadStatusPB::ModelDownloadCompleted
          } else if !has_actual_download && current_total_size >= 1_500_000_000 {
            // 没有新的下载活动，但模型大小已经很大（1.5-2.0 GB），说明模型已存在但不完整
            // 标记为"已完成"但提示模型可能不完整
            tracing::warn!("[模型下载] marker 命令执行完成，模型已存在但检查显示未完全就绪（大小: {:.2} GB），可能缺少某些模型文件", 
              current_total_size as f64 / (1024.0 * 1024.0 * 1024.0));
            ModelDownloadStatusPB::ModelDownloadCompleted
          } else if progress >= 0.9 && current_total_size >= 1_500_000_000 && has_actual_download {
            // 有下载活动，进度很高，但检查显示未完全就绪
            // 可能还在下载中，标记为"下载中"
            tracing::warn!("[模型下载] 模型大小足够但检查显示未完全就绪（进度: {:.0}%，大小: {:.2} GB），可能还在下载中", 
              progress * 100.0, current_total_size as f64 / (1024.0 * 1024.0 * 1024.0));
            ModelDownloadStatusPB::ModelDownloadDownloading
          } else if progress >= 0.9 {
            // 进度很高，但没有检测到新的下载活动，说明下载已完成但模型可能不完整
            tracing::warn!("[模型下载] marker 命令执行完成，模型可能已下载但检查显示未完全就绪（进度: {:.0}%）", progress * 100.0);
            ModelDownloadStatusPB::ModelDownloadCompleted
          } else if progress >= 0.5 {
            // 部分完成，但未完全就绪
            tracing::warn!("[模型下载] marker 命令执行成功，但模型可能未完全下载（进度: {:.0}%）", progress * 100.0);
            // 如果没有新的下载活动，说明下载已停止，不应该显示"正在下载"
            if has_actual_download {
              ModelDownloadStatusPB::ModelDownloadDownloading
            } else {
              ModelDownloadStatusPB::ModelDownloadCompleted
            }
          } else {
            // 下载可能失败
            tracing::warn!("[模型下载] marker 命令执行成功，但模型下载可能失败（进度: {:.0}%）", progress * 100.0);
            ModelDownloadStatusPB::ModelDownloadFailed
          };
          
          let status_message = if !has_actual_download && current_total_size >= 2_000_000_000 {
            format!("marker 命令执行完成，模型文件已存在（{:.2} GB），模型已就绪，可以正常使用",
              current_total_size as f64 / (1024.0 * 1024.0 * 1024.0))
          } else if !has_actual_download && current_total_size >= 1_500_000_000 {
            format!("marker 命令执行完成，模型文件已存在（{:.2} GB），但检查显示未完全就绪。可能缺少某些必需的模型文件（如 Surya OCR 模型）。模型可能已下载但不完整，请检查模型状态或重新尝试下载",
              current_total_size as f64 / (1024.0 * 1024.0 * 1024.0))
          } else if progress >= 0.9 && current_total_size >= 1_500_000_000 && has_actual_download {
            "模型文件已下载大部分，但检查显示未完全就绪。可能还在下载中，或缺少某些必需的模型文件（如 Surya OCR 模型），请检查模型状态或等待下载完成".to_string()
          } else if progress >= 0.9 {
            "模型下载基本完成，但检查显示未完全就绪。可能缺少某些必需的模型文件，请检查模型状态".to_string()
          } else if progress >= 0.5 {
            if has_actual_download {
              "模型部分下载，可能还在下载中，请等待或重新尝试".to_string()
            } else {
              "模型部分下载，但未检测到新的下载活动。模型可能已存在但不完整，请检查模型状态或重新尝试下载".to_string()
            }
          } else {
            "模型下载可能失败，请检查网络连接或重新尝试".to_string()
          };
          
          // 估算总大小（基于预期完整模型大小）
          let estimated_total = 2_500_000_000u64; // 2.5 GB
          let total_bytes_estimate = if reported_bytes > 0 {
            Some(reported_bytes.max(estimated_total))
          } else {
            Some(estimated_total)
          };
          
          data_result_ok(ModelDownloadProgressPB {
            status: final_status,
            progress,
            current_model: None,
            message: format!("{}\n\n注意: {}", message, status_message),
            downloaded_bytes: reported_bytes,
            total_bytes: total_bytes_estimate,
          })
        }
      } else {
        let exit_code = status.code().unwrap_or(-1);
        
        // 分离错误日志和普通日志
        let error_logs: Vec<String> = output_logs
          .iter()
          .filter(|log| log.contains("Error") || log.contains("error") || log.contains("ERROR") || log.contains("Exception") || log.contains("Traceback"))
          .cloned()
          .collect();
        
        let recent_logs: Vec<String> = output_logs.iter().rev().take(30).rev().cloned().collect();
        
        let mut message = format!(
          "模型下载失败（耗时: {:.1} 秒）\n退出码: {}\n\n",
          elapsed.as_secs_f64(),
          exit_code
        );
        
        // 优先显示错误日志
        if !error_logs.is_empty() {
          message.push_str("错误信息:\n");
          for error_log in error_logs.iter().take(10) {
            message.push_str(error_log);
            message.push('\n');
          }
          message.push_str("\n");
        }
        
        // 显示所有日志
        if !recent_logs.is_empty() {
          message.push_str("执行日志:\n");
          message.push_str(&recent_logs.join("\n"));
        } else {
          message.push_str("（Marker 工具未输出日志）\n\n可能的原因：\n1. marker 工具执行出错\n2. 缺少必要的 Python 依赖\n3. 网络连接问题\n4. 权限问题\n5. 测试 PDF 文件创建失败");
        }
        
        tracing::error!(
          "[模型下载] marker 命令执行失败 - 退出码: {}, 耗时: {:?}, 已下载: {} 字节, 日志行数: {}, 错误日志数: {}",
          exit_code,
          elapsed,
          total_downloaded,
          output_logs.len(),
          error_logs.len()
        );
        
        // 如果日志为空，记录更多调试信息
        if output_logs.is_empty() {
          tracing::warn!(
            "[模型下载] marker 工具未输出任何日志 - 这可能表示工具启动失败或立即退出"
          );
        } else if !error_logs.is_empty() {
          // 记录关键错误信息
          for error_log in error_logs.iter().take(5) {
            tracing::error!("[模型下载] 关键错误: {}", error_log);
          }
        }
        
        data_result_ok(ModelDownloadProgressPB {
          status: ModelDownloadStatusPB::ModelDownloadFailed,
          progress: 0.0,
          current_model: None,
          message,
          downloaded_bytes: total_downloaded,
          total_bytes: None,
        })
      }
    },
    Ok(Err(e)) => {
      let elapsed = start_time.elapsed();
      tracing::error!(
        "[模型下载] 无法等待 marker 命令: {} (耗时: {:?})",
        e,
        elapsed
      );
      
      // 尝试获取已收集的日志
      let output_logs: Vec<String> = if let Ok(logs) = logs_arc.lock() {
        logs.clone()
      } else {
        Vec::new()
      };
      
      let mut message = format!(
        "无法等待 marker 命令执行完成\n错误: {}\n耗时: {:.1} 秒\n\n",
        e,
        elapsed.as_secs_f64()
      );
      
      if !output_logs.is_empty() {
        message.push_str("执行日志:\n");
        let recent_logs: Vec<String> = output_logs.iter().rev().take(20).rev().cloned().collect();
        message.push_str(&recent_logs.join("\n"));
      } else {
        message.push_str("（Marker 工具未输出日志）");
      }
      
      data_result_ok(ModelDownloadProgressPB {
        status: ModelDownloadStatusPB::ModelDownloadFailed,
        progress: 0.0,
        current_model: None,
        message,
        downloaded_bytes: 0,
        total_bytes: None,
      })
    },
    Err(_) => {
      let elapsed = start_time.elapsed();
      tracing::error!(
        "[模型下载] marker 命令执行超时（超过 30 分钟，实际耗时: {:?}）",
        elapsed
      );
      
      // 检查是否有部分下载
      let current_hf_size = get_dir_size(&hf_cache_dir);
      let current_surya_size = get_dir_size(&surya_cache_dir);
      let downloaded_hf = current_hf_size.saturating_sub(initial_hf_size);
      let downloaded_surya = current_surya_size.saturating_sub(initial_surya_size);
      let total_downloaded = downloaded_hf + downloaded_surya;
      
      let mut message = format!(
        "模型下载超时（超过 30 分钟）\n实际耗时: {:.1} 秒\n已下载: {:.2} GB\n\n请检查：\n1. 网络连接是否稳定\n2. 下载速度是否过慢\n3. 是否需要使用代理\n\n执行日志:\n",
        elapsed.as_secs_f64(),
        total_downloaded as f64 / (1024.0 * 1024.0 * 1024.0)
      );
      
      let recent_logs: Vec<String> = output_logs.iter().rev().take(20).rev().cloned().collect();
      if !recent_logs.is_empty() {
        message.push_str(&recent_logs.join("\n"));
      } else {
        message.push_str("（Marker 工具未输出日志）");
      }
      
      data_result_ok(ModelDownloadProgressPB {
        status: ModelDownloadStatusPB::ModelDownloadFailed,
        progress: 0.0,
        current_model: None,
        message,
        downloaded_bytes: total_downloaded,
        total_bytes: None,
      })
    }
  }
}

/// 计算目录大小（字节）
fn get_dir_size(dir_path: &str) -> u64 {
  use std::fs;
  use std::path::PathBuf;
  
  let path = PathBuf::from(dir_path);
  if !path.exists() || !path.is_dir() {
    return 0;
  }
  
  let mut total_size = 0u64;
  if let Ok(entries) = fs::read_dir(&path) {
    for entry in entries.flatten() {
      let entry_path = entry.path();
      if entry_path.is_file() {
        if let Ok(metadata) = entry_path.metadata() {
          total_size += metadata.len();
        }
      } else if entry_path.is_dir() {
        // 递归计算子目录大小（限制深度避免过深）
        total_size += get_dir_size_recursive(&entry_path, 5);
      }
    }
  }
  
  total_size
}

/// 递归计算目录大小（带深度限制）
fn get_dir_size_recursive(path: &std::path::Path, max_depth: u32) -> u64 {
  use std::fs;
  
  if max_depth == 0 {
    return 0;
  }
  
  let mut total_size = 0u64;
  if let Ok(entries) = fs::read_dir(path) {
    for entry in entries.flatten() {
      let entry_path = entry.path();
      if entry_path.is_file() {
        if let Ok(metadata) = entry_path.metadata() {
          total_size += metadata.len();
        }
      } else if entry_path.is_dir() {
        total_size += get_dir_size_recursive(&entry_path, max_depth - 1);
      }
    }
  }
  
  total_size
}

/// 获取模型缓存目录路径
fn get_model_cache_dirs() -> (String, String) {
  #[cfg(target_os = "macos")]
  {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    (
      format!("{}/Library/Caches/huggingface", home),
      format!("{}/Library/Caches/datalab/models", home),
    )
  }
  
  #[cfg(target_os = "windows")]
  {
    let userprofile = std::env::var("USERPROFILE").unwrap_or_else(|_| "%USERPROFILE%".to_string());
    let localappdata = std::env::var("LOCALAPPDATA")
      .unwrap_or_else(|_| format!("{}\\AppData\\Local", userprofile));
    (
      format!("{}\\.cache\\huggingface", userprofile),
      format!("{}\\datalab\\models", localappdata),
    )
  }
  
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    (
      format!("{}/.cache/huggingface", home),
      format!("{}/.cache/datalab/models", home),
    )
  }
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

/// 检查 marker-pdf 工具状态
/// 
/// marker-pdf 是实际执行 PDF 转换的 Python 工具，通过 pipx 安装。
/// 此函数检查 marker-pdf 是否已安装，并返回详细的工具信息。
fn check_marker_pdf_tool() -> crate::entities::ImportToolInfoPB {
  use crate::entities::{ImportToolInfoPB, ImportToolStatusPB};
  use std::process::Command;
  
  let mut check_logs = Vec::new();
  check_logs.push("开始检查 marker-pdf 工具...".to_string());
  
  tracing::info!("[工具检查] 开始检查 marker-pdf 工具");
  
  // 获取 marker-pdf 的安装路径
  let marker_pdf_path = match get_marker_pdf_path() {
    Ok(path) => {
      check_logs.push(format!("✓ 确定 marker-pdf 期望路径: {}", path.display()));
      path
    },
    Err(e) => {
      let error_msg = format!("✗ 无法获取 marker-pdf 路径: {}", e);
      check_logs.push(error_msg.clone());
      tracing::warn!("[工具检查] {}", error_msg);
      
      let install_instruction = get_marker_pdf_install_instruction()
        .unwrap_or_else(|_| "请参考 marker-pdf 安装文档".to_string());
      
      return ImportToolInfoPB {
        name: "marker-pdf (PDF 转换引擎)".to_string(),
        status: ImportToolStatusPB::ToolUnknown,
        version: None,
        path: None,
        install_instruction: Some(format!(
          "无法确定 marker-pdf 安装路径: {}\n\n{}",
          e,
          install_instruction
        )),
        description: "实际执行 PDF 转换的 Python 工具，通过 pipx 安装。将 PDF 转换为 Markdown 格式，保留格式和结构。".to_string(),
        model_status: None,
        check_logs,
      };
    }
  };
  
  tracing::debug!("[工具检查] marker-pdf 期望路径: {}", marker_pdf_path.display());
  
  // 检查 marker-pdf 是否存在
  check_logs.push(format!("检查文件是否存在: {}", marker_pdf_path.display()));
  
  if marker_pdf_path.exists() && marker_pdf_path.is_file() {
    check_logs.push("✓ 文件存在且是有效文件".to_string());
    tracing::info!("[工具检查] marker-pdf 已安装: {}", marker_pdf_path.display());
    
    // 检查文件权限
    #[cfg(unix)]
    {
      use std::fs::Permissions;
      use std::os::unix::fs::PermissionsExt;
      if let Ok(metadata) = marker_pdf_path.metadata() {
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        check_logs.push(format!("文件权限: {:o}", mode));
        if mode & 0o111 != 0 {
          check_logs.push("✓ 文件具有执行权限".to_string());
        } else {
          check_logs.push("⚠ 文件缺少执行权限".to_string());
        }
      }
    }
    
    // 尝试获取版本信息
    check_logs.push("尝试获取版本信息...".to_string());
    let version = get_marker_pdf_version_with_logs(&marker_pdf_path, &mut check_logs);
    
    // 检查模型就绪度
    check_logs.push("检查模型就绪度...".to_string());
    let (model_status, model_logs) = check_marker_models_ready_with_logs();
    check_logs.extend(model_logs);
    
    let description = "实际执行 PDF 转换的 Python 工具，通过 pipx 安装。将 PDF 转换为 Markdown 格式，保留格式和结构。".to_string();
    
    ImportToolInfoPB {
      name: "marker-pdf (PDF 转换引擎)".to_string(),
      status: ImportToolStatusPB::ToolAvailable,
      version,
      path: Some(marker_pdf_path.display().to_string()),
      install_instruction: None,
      description,
      model_status: Some(model_status),
      check_logs,
    }
  } else {
    check_logs.push("✗ 文件不存在或不是有效文件".to_string());
    tracing::warn!("[工具检查] marker-pdf 未安装: {}", marker_pdf_path.display());
    
    // 检查父目录是否存在
    if let Some(parent) = marker_pdf_path.parent() {
      if parent.exists() {
        check_logs.push(format!("✓ 父目录存在: {}", parent.display()));
      } else {
        check_logs.push(format!("✗ 父目录不存在: {}", parent.display()));
      }
    }
    
    // 获取安装指引
    let install_instruction = get_marker_pdf_install_instruction()
      .unwrap_or_else(|_| "请参考 marker-pdf 安装文档".to_string());
    
    ImportToolInfoPB {
      name: "marker-pdf (PDF 转换引擎)".to_string(),
      status: ImportToolStatusPB::ToolNotInstalled,
      version: None,
      path: Some(marker_pdf_path.display().to_string()),
      install_instruction: Some(install_instruction),
      description: "实际执行 PDF 转换的 Python 工具，通过 pipx 安装。将 PDF 转换为 Markdown 格式，保留格式和结构。".to_string(),
      model_status: None,
      check_logs,
    }
  }
}

/// 获取 marker-pdf 的版本信息（带日志）
fn get_marker_pdf_version_with_logs(marker_pdf_path: &Path, logs: &mut Vec<String>) -> Option<String> {
  use std::process::Command;
  
  // 通过 pipx 虚拟环境中的 Python 使用 importlib.metadata（最可靠的方法）
  logs.push("  通过 pipx 虚拟环境 Python 使用 importlib.metadata 获取版本...".to_string());
  
  // 查找 Python 解释器（在 marker_single 的同一目录或父目录）
  let python_candidates = if let Some(bin_dir) = marker_pdf_path.parent() {
    vec![
      bin_dir.join("python"),
      bin_dir.join("python3"),
      #[cfg(target_os = "windows")]
      bin_dir.join("python.exe"),
      #[cfg(target_os = "windows")]
      bin_dir.join("python3.exe"),
    ]
  } else {
    vec![]
  };
  
  for python_path in &python_candidates {
    if python_path.exists() {
      logs.push(format!("  找到 Python: {}", python_path.display()));
      // 使用 importlib.metadata 获取版本（Python 3.8+）
      match Command::new(python_path)
        .arg("-c")
        .arg("try:\n    import importlib.metadata\n    print(importlib.metadata.version('marker-pdf'))\nexcept Exception as e:\n    print(f'error: {e}')")
        .output()
      {
        Ok(output) => {
          let stdout = String::from_utf8_lossy(&output.stdout);
          let stderr = String::from_utf8_lossy(&output.stderr);
          let stdout_trimmed = stdout.trim();
          
          if output.status.success() && !stdout_trimmed.is_empty() && !stdout_trimmed.starts_with("error:") {
            let version = stdout_trimmed.to_string();
            logs.push(format!("✓ 通过 importlib.metadata 获取版本: {}", version));
            tracing::debug!("[工具检查] marker-pdf 版本 (importlib.metadata): {}", version);
            return Some(version);
          } else {
            if !stderr.trim().is_empty() {
              logs.push(format!("  ✗ importlib.metadata 方式失败: {}", stderr.trim()));
            } else if stdout_trimmed.starts_with("error:") {
              logs.push(format!("  ✗ importlib.metadata 失败: {}", stdout_trimmed));
            }
          }
        },
        Err(e) => {
          logs.push(format!("  ✗ 无法执行 Python 命令: {}", e));
        }
      }
      break; // 只尝试第一个找到的 Python
    }
  }
  
  // 方法5: 通过 pipx list 命令解析版本（备用方法）
  logs.push("  尝试通过 pipx list 命令获取版本...".to_string());
  match Command::new("pipx").arg("list").output() {
    Ok(output) => {
      let stdout = String::from_utf8_lossy(&output.stdout);
      if output.status.success() {
        // 解析输出，查找 marker-pdf 的版本
        // 格式通常是: "   package marker-pdf 1.10.1, installed using Python 3.14.0"
        for line in stdout.lines() {
          if line.contains("marker-pdf") {
            // 尝试提取版本号（格式：package marker-pdf VERSION,）
            if let Some(version_start) = line.find("marker-pdf") {
              let after_marker = &line[version_start + "marker-pdf".len()..];
              // 跳过空格，查找版本号
              let version_part: String = after_marker
                .chars()
                .skip_while(|c| c.is_whitespace())
                .take_while(|c| !c.is_whitespace() && *c != ',')
                .collect();
              
              // 验证是否是有效的版本号格式（包含数字和点）
              if !version_part.is_empty() && version_part.chars().any(|c| c.is_ascii_digit()) {
                logs.push(format!("✓ 通过 pipx list 获取版本: {}", version_part));
                tracing::debug!("[工具检查] marker-pdf 版本 (pipx list): {}", version_part);
                return Some(version_part);
              }
            }
          }
        }
        logs.push("  ✗ pipx list 输出中未找到 marker-pdf 版本信息".to_string());
      } else {
        logs.push(format!("  ✗ pipx list 命令执行失败 (退出码: {})", 
          output.status.code().unwrap_or(-1)));
      }
    },
    Err(e) => {
      logs.push(format!("  ✗ 无法执行 pipx 命令: {}", e));
    }
  }
  
  // 方法6: 尝试通过 Python 模块方式获取版本（原有方法，作为最后尝试）
  logs.push("  尝试通过 Python 模块获取版本...".to_string());
  
  for python_path in python_candidates {
    if python_path.exists() {
      logs.push(format!("  找到 Python: {}", python_path.display()));
      match Command::new(&python_path)
        .arg("-c")
        .arg("try:\n    import marker\n    print(getattr(marker, '__version__', 'unknown'))\nexcept Exception as e:\n    print(f'error: {e}')")
        .output()
      {
        Ok(output) => {
          let stdout = String::from_utf8_lossy(&output.stdout);
          let stderr = String::from_utf8_lossy(&output.stderr);
          let stdout_trimmed = stdout.trim();
          
          if output.status.success() && !stdout_trimmed.is_empty() && !stdout_trimmed.starts_with("error:") && stdout_trimmed != "unknown" {
            let version = stdout_trimmed.to_string();
            logs.push(format!("✓ 通过 Python 模块获取版本: {}", version));
            return Some(version);
          } else {
            if !stderr.trim().is_empty() {
              logs.push(format!("  ✗ Python 模块方式失败: {}", stderr.trim()));
            } else if stdout_trimmed.starts_with("error:") {
              logs.push(format!("  ✗ Python 模块导入失败: {}", stdout_trimmed));
            }
          }
        },
        Err(e) => {
          logs.push(format!("  ✗ 无法执行 Python 命令: {}", e));
        }
      }
      break; // 只尝试第一个找到的 Python
    }
  }
  
  // 如果没找到同目录的 Python，尝试使用系统 Python 并设置正确的路径
  logs.push("  尝试使用系统 Python...".to_string());
  let system_python = if cfg!(target_os = "windows") { "python" } else { "python3" };
  
  // 尝试从 marker_pdf_path 推断 site-packages 路径
  if let Some(venv_path) = marker_pdf_path.parent()
    .and_then(|p| p.parent())
    .and_then(|p| p.parent())
  {
    // 构建可能的 site-packages 路径
    let site_packages_pattern = if cfg!(target_os = "windows") {
      let path = venv_path.join("lib").join("site-packages");
      if path.exists() {
        Some(path)
      } else {
        None
      }
    } else {
      // Unix-like: 查找 lib/python*/site-packages
      let mut found_path = None;
      if let Ok(entries) = std::fs::read_dir(venv_path.join("lib")) {
        for entry in entries.flatten() {
          let path = entry.path();
          if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
              if dir_name.starts_with("python") {
                let site_packages = path.join("site-packages");
                if site_packages.exists() {
                  found_path = Some(site_packages);
                  break;
                }
              }
            }
          }
        }
      }
      found_path
    };
    
    if let Some(site_packages) = site_packages_pattern {
      let site_packages_str = site_packages.to_string_lossy().to_string();
      logs.push(format!("  尝试使用 site-packages: {}", site_packages_str));
      
      let python_code = format!(
        "try:\n    import sys\n    sys.path.insert(0, r'{}')\n    import marker\n    print(getattr(marker, '__version__', 'unknown'))\nexcept Exception as e:\n    print(f'error: {{e}}')",
        site_packages_str.replace('\\', "\\\\")
      );
      
      match Command::new(system_python).arg("-c").arg(&python_code).output() {
        Ok(output) => {
          let stdout = String::from_utf8_lossy(&output.stdout);
          let stderr = String::from_utf8_lossy(&output.stderr);
          let stdout_trimmed = stdout.trim();
          
          if output.status.success() && !stdout_trimmed.is_empty() && !stdout_trimmed.starts_with("error:") && stdout_trimmed != "unknown" {
            let version = stdout_trimmed.to_string();
            logs.push(format!("✓ 通过系统 Python 获取版本: {}", version));
            return Some(version);
          } else {
            if !stderr.trim().is_empty() {
              logs.push(format!("  ✗ 系统 Python 方式失败: {}", stderr.trim()));
            }
          }
        },
        Err(e) => {
          logs.push(format!("  ✗ 无法执行系统 Python: {}", e));
        }
      }
    }
  }
  
  logs.push("⚠ 无法获取版本信息（所有方法均失败，但这不影响工具使用）".to_string());
  logs.push("  说明: marker_single 可能不支持标准版本参数，这是正常的".to_string());
  logs.push("  工具仍然可以正常使用，只是无法显示版本号".to_string());
  tracing::debug!("[工具检查] 无法获取 marker-pdf 版本信息");
  None
}

/// 获取 marker-pdf 的版本信息
fn get_marker_pdf_version(marker_pdf_path: &Path) -> Option<String> {
  let mut logs = Vec::new();
  get_marker_pdf_version_with_logs(marker_pdf_path, &mut logs)
}

/// 安装缺失的工具
#[tracing::instrument(level = "info", skip_all, err)]
pub async fn install_missing_tools(
  data: AFPluginData<crate::entities::InstallMissingToolsPB>,
) -> DataResult<crate::entities::InstallToolProgressPB, FlowyError> {
  use crate::entities::{InstallToolProgressPB, InstallToolStatusPB};
  use std::process::Command;
  use std::time::Duration;
  use tokio::time::sleep;
  
  let request = data.into_inner();
  tracing::info!("[工具安装] 开始安装缺失工具: {:?}", request.tool_names);
  
  let mut logs = Vec::new();
  logs.push(format!("开始安装工具: {:?}", request.tool_names));
  
  // 检查需要安装哪些工具
  let tools_status = check_import_tools_status().await?;
  let mut tools_to_install = Vec::new();
  
  for tool_name in &request.tool_names {
    if let Some(tool) = tools_status.tools.iter().find(|t| t.name == *tool_name) {
      if tool.status == crate::entities::ImportToolStatusPB::ToolNotInstalled {
        tools_to_install.push(tool_name.clone());
      }
    }
  }
  
  if tools_to_install.is_empty() {
    logs.push("所有工具已安装，无需安装".to_string());
    return data_result_ok(InstallToolProgressPB {
      tool_name: "all".to_string(),
      status: InstallToolStatusPB::InstallToolCompleted,
      progress: 1.0,
      message: "所有工具已安装".to_string(),
      logs,
    });
  }
  
  logs.push(format!("需要安装的工具: {:?}", tools_to_install));
  
  #[cfg(target_os = "macos")]
  {
    // macOS 安装流程：brew -> pipx -> marker-pdf
    let mut current_progress = 0.0;
    let total_steps = tools_to_install.len() as f64;
    
    // 检查并安装 brew
    if tools_to_install.contains(&"brew".to_string()) || 
       tools_to_install.iter().any(|t| t.contains("marker")) {
      logs.push("检查 Homebrew...".to_string());
      let brew_available = Command::new("which")
        .arg("brew")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
      
      if !brew_available {
        logs.push("Homebrew 未安装，开始安装...".to_string());
        logs.push("注意: Homebrew 安装需要用户交互，请按照提示操作".to_string());
        
        // 尝试安装 Homebrew
        let install_script = "/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"";
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        
        logs.push(format!("执行: {}", install_script));
        
        // 注意：Homebrew 安装需要用户交互，这里只能提供指引
        return data_result_ok(InstallToolProgressPB {
          tool_name: "brew".to_string(),
          status: InstallToolStatusPB::InstallToolFailed,
          progress: 0.0,
          message: "Homebrew 安装需要用户交互，无法自动安装。请打开终端运行以下命令：\n/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"".to_string(),
          logs,
        });
      } else {
        logs.push("✓ Homebrew 已安装".to_string());
      }
    }
    
    // 安装 pipx（如果需要）
    if tools_to_install.contains(&"pipx".to_string()) || 
       tools_to_install.iter().any(|t| t.contains("marker")) {
      logs.push("检查 pipx...".to_string());
      let pipx_available = Command::new("which")
        .arg("pipx")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
      
      if !pipx_available {
        logs.push("pipx 未安装，开始安装...".to_string());
        current_progress += 1.0 / total_steps;
        
        // 使用 shell 执行 brew install pipx
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        let cmd = if shell.contains("zsh") {
          "source ~/.zprofile 2>/dev/null || source ~/.zshrc 2>/dev/null || true; brew install pipx"
        } else if shell.contains("bash") {
          "source ~/.bash_profile 2>/dev/null || source ~/.bashrc 2>/dev/null || true; brew install pipx"
        } else {
          "brew install pipx"
        };
        
        logs.push(format!("执行: {}", cmd));
        
        match Command::new(&shell)
          .arg("-c")
          .arg(cmd)
          .output()
        {
          Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            logs.push(format!("stdout: {}", stdout));
            if !stderr.is_empty() {
              logs.push(format!("stderr: {}", stderr));
            }
            
            if output.status.success() {
              logs.push("✓ pipx 安装成功".to_string());
            } else {
              logs.push(format!("✗ pipx 安装失败: {}", stderr));
              return data_result_ok(InstallToolProgressPB {
                tool_name: "pipx".to_string(),
                status: InstallToolStatusPB::InstallToolFailed,
                progress: current_progress,
                message: format!("pipx 安装失败: {}", stderr),
                logs,
              });
            }
          }
          Err(e) => {
            logs.push(format!("✗ pipx 安装出错: {}", e));
            return data_result_ok(InstallToolProgressPB {
              tool_name: "pipx".to_string(),
              status: InstallToolStatusPB::InstallToolFailed,
              progress: current_progress,
              message: format!("pipx 安装出错: {}", e),
              logs,
            });
          }
        }
      } else {
        logs.push("✓ pipx 已安装".to_string());
      }
    }
    
    // 安装 marker-pdf 的依赖（如果需要）
    if tools_to_install.iter().any(|t| t.contains("marker")) {
      logs.push("安装 marker-pdf 依赖库...".to_string());
      current_progress += 1.0 / total_steps;
      
      let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
      let deps_cmd = if shell.contains("zsh") {
        "source ~/.zprofile 2>/dev/null || source ~/.zshrc 2>/dev/null || true; brew install jpeg libpng freetype openjpeg libtiff webp"
      } else if shell.contains("bash") {
        "source ~/.bash_profile 2>/dev/null || source ~/.bashrc 2>/dev/null || true; brew install jpeg libpng freetype openjpeg libtiff webp"
      } else {
        "brew install jpeg libpng freetype openjpeg libtiff webp"
      };
      
      logs.push(format!("执行: {}", deps_cmd));
      
      match Command::new(&shell)
        .arg("-c")
        .arg(deps_cmd)
        .output()
      {
        Ok(output) => {
          let stdout = String::from_utf8_lossy(&output.stdout);
          let stderr = String::from_utf8_lossy(&output.stderr);
          if !stdout.trim().is_empty() {
            logs.push(format!("stdout: {}", stdout.trim()));
          }
          if !stderr.trim().is_empty() && !stderr.contains("Warning") {
            logs.push(format!("stderr: {}", stderr.trim()));
          }
          
          if output.status.success() {
            logs.push("✓ 依赖库安装成功".to_string());
          } else {
            // 依赖库可能已经安装，继续
            logs.push("⚠ 依赖库安装可能已存在或失败，继续安装 marker-pdf".to_string());
          }
        }
        Err(e) => {
          logs.push(format!("⚠ 依赖库安装出错: {}，继续安装 marker-pdf", e));
        }
      }
    }
    
    // 安装 marker-pdf（如果需要）
    if tools_to_install.iter().any(|t| t.contains("marker")) {
      logs.push("安装 marker-pdf...".to_string());
      current_progress += 1.0 / total_steps;
      
      let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
      let pipx_cmd = if shell.contains("zsh") {
        "source ~/.zprofile 2>/dev/null || source ~/.zshrc 2>/dev/null || true; pipx install marker-pdf"
      } else if shell.contains("bash") {
        "source ~/.bash_profile 2>/dev/null || source ~/.bashrc 2>/dev/null || true; pipx install marker-pdf"
      } else {
        "pipx install marker-pdf"
      };
      
      logs.push(format!("执行: {}", pipx_cmd));
      logs.push("注意: marker-pdf 安装可能需要几分钟时间，请耐心等待...".to_string());
      
      match Command::new(&shell)
        .arg("-c")
        .arg(pipx_cmd)
        .output()
      {
        Ok(output) => {
          let stdout = String::from_utf8_lossy(&output.stdout);
          let stderr = String::from_utf8_lossy(&output.stderr);
          if !stdout.trim().is_empty() {
            logs.push(format!("stdout: {}", stdout.trim()));
          }
          if !stderr.trim().is_empty() && !stderr.contains("Warning") {
            logs.push(format!("stderr: {}", stderr.trim()));
          }
          
          if output.status.success() {
            logs.push("✓ marker-pdf 安装成功".to_string());
            current_progress = 1.0;
            
            return data_result_ok(InstallToolProgressPB {
              tool_name: "marker-pdf".to_string(),
              status: InstallToolStatusPB::InstallToolCompleted,
              progress: current_progress,
              message: "所有工具安装完成".to_string(),
              logs,
            });
          } else {
            logs.push(format!("✗ marker-pdf 安装失败: {}", stderr));
            return data_result_ok(InstallToolProgressPB {
              tool_name: "marker-pdf".to_string(),
              status: InstallToolStatusPB::InstallToolFailed,
              progress: current_progress,
              message: format!("marker-pdf 安装失败: {}", stderr),
              logs,
            });
          }
        }
        Err(e) => {
          logs.push(format!("✗ marker-pdf 安装出错: {}", e));
          return data_result_ok(InstallToolProgressPB {
            tool_name: "marker-pdf".to_string(),
            status: InstallToolStatusPB::InstallToolFailed,
            progress: current_progress,
            message: format!("marker-pdf 安装出错: {}", e),
            logs,
          });
        }
      }
    }
    
    // 所有工具安装完成
    data_result_ok(InstallToolProgressPB {
      tool_name: "all".to_string(),
      status: InstallToolStatusPB::InstallToolCompleted,
      progress: 1.0,
      message: "所有工具安装完成".to_string(),
      logs,
    })
  }
  
  #[cfg(not(target_os = "macos"))]
  {
    logs.push(format!("当前平台 {} 不支持自动安装工具", std::env::consts::OS));
    data_result_ok(InstallToolProgressPB {
      tool_name: "unknown".to_string(),
      status: InstallToolStatusPB::InstallToolFailed,
      progress: 0.0,
      message: format!("当前平台 {} 不支持自动安装工具", std::env::consts::OS),
      logs,
    })
  }
}
