use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::interface::error::CommandError;

#[tauri::command]
pub async fn show_os_notification(
    app: AppHandle,
    title: String,
    body: Option<String>,
    activation_url: Option<String>,
) -> anyhow::Result<(), CommandError> {
    #[cfg(target_os = "windows")]
    {
        if show_windows_notification(&app, &title, body.as_deref(), activation_url.as_deref())
            .is_ok()
        {
            return Ok(());
        }
    }

    let mut builder = app.notification().builder().title(title);
    if let Ok(icon_path) = app.path().resolve("icons/32x32.png", BaseDirectory::Resource) {
        builder = builder.icon(icon_path.to_string_lossy().to_string());
    }
    if let Some(body) = body {
        builder = builder.body(body);
    }
    builder.show().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn show_windows_notification(
    app: &AppHandle,
    title: &str,
    body: Option<&str>,
    activation_url: Option<&str>,
) -> Result<(), anyhow::Error> {
    use tauri_plugin_shell::ShellExt;
    use tauri_winrt_notification::{Duration, Toast};

    let app_id = app.config().identifier.clone();
    let app_handle = app.clone();
    let activation_url = activation_url
        .filter(|url| !url.trim().is_empty())
        .map(std::borrow::ToOwned::to_owned);

    let mut toast = Toast::new(&app_id)
        .title(title)
        .duration(Duration::Short);

    if let Some(body) = body.filter(|value| !value.is_empty()) {
        toast = toast.text1(body);
    }

    if let Some(activation_url) = activation_url {
        toast = toast.on_activated(move |_| {
            #[allow(deprecated)]
            let shell = app_handle.shell();
            #[allow(deprecated)]
            if let Err(err) = shell.open(activation_url.clone(), None) {
                log::error!("failed to open notification activation url: {err}");
            }
            Ok(())
        });
    }

    toast.show().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(())
}
