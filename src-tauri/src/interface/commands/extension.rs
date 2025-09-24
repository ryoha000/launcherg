use std::sync::Arc;
use tauri::AppHandle;

use crate::interface::error::CommandError;
use crate::interface::module::{Modules, ModulesExt};
use domain::extension::SyncStatus;
use domain::extension::ExtensionConfig;

#[tauri::command]
pub async fn get_sync_status(
    modules: tauri::State<'_, Arc<Modules>>,
) -> anyhow::Result<SyncStatus, CommandError> {
    let status = modules
        .extension_manager_use_case()
        .check_extension_connection()
        .await
        .map_err(|e| anyhow::anyhow!("拡張機能の接続確認に失敗: {}", e))?;

    Ok(status)
}

#[tauri::command]
pub async fn set_extension_config(
    config: ExtensionConfig,
    modules: tauri::State<'_, Arc<Modules>>,
) -> anyhow::Result<String, CommandError> {
    let result = modules
        .extension_manager_use_case()
        .set_extension_config(&config)
        .await
        .map_err(|e| anyhow::anyhow!("拡張機能設定の更新に失敗: {}", e))?;

    Ok(result)
}

#[tauri::command]
pub async fn generate_extension_package(
    handle: AppHandle,
) -> anyhow::Result<usecase::extension_installer::ExtensionPackageInfo, CommandError> {
    use usecase::extension_installer::ExtensionInstallerUseCase;

    let installer = ExtensionInstallerUseCase::new(Arc::new(handle));
    let package_info = installer
        .generate_extension_package()
        .await
        .map_err(|e| anyhow::anyhow!("拡張機能パッケージの生成に失敗: {}", e))?;

    Ok(package_info)
}

#[tauri::command]
pub async fn setup_native_messaging_host(
    handle: AppHandle,
    extension_id: Option<String>,
) -> anyhow::Result<String, CommandError> {
    use usecase::extension_installer::ExtensionInstallerUseCase;

    let installer = ExtensionInstallerUseCase::new(Arc::new(handle));
    let result = installer
        .setup_native_messaging_host(extension_id)
        .await
        .map_err(|e| anyhow::anyhow!("Native Messaging Hostのセットアップに失敗: {}", e))?;

    Ok(result)
}

#[tauri::command]
pub async fn get_extension_package_info(
    handle: AppHandle,
) -> anyhow::Result<Option<usecase::extension_installer::ExtensionPackageInfo>, CommandError> {
    use usecase::extension_installer::ExtensionInstallerUseCase;

    let installer = ExtensionInstallerUseCase::new(Arc::new(handle));

    if installer.is_package_available() {
        let manifest_info = installer
            .get_extension_manifest_info()
            .await
            .map_err(|e| anyhow::anyhow!("拡張機能情報の取得に失敗: {}", e))?;
        let package_path = installer.get_package_path();
        let _package_size = installer
            .get_package_size()
            .map_err(|e| anyhow::anyhow!("パッケージサイズの取得に失敗: {}", e))?;

        Ok(Some(usecase::extension_installer::ExtensionPackageInfo {
            version: manifest_info.version.clone(),
            package_path: package_path.to_string_lossy().to_string(),
            manifest_info,
        }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn copy_extension_for_development(
    handle: AppHandle,
) -> anyhow::Result<String, CommandError> {
    use usecase::extension_installer::ExtensionInstallerUseCase;

    let installer = ExtensionInstallerUseCase::new(Arc::new(handle));
    let dev_path = installer
        .copy_extension_for_development()
        .await
        .map_err(|e| anyhow::anyhow!("開発用拡張機能のコピーに失敗: {}", e))?;

    Ok(dev_path)
}

#[tauri::command]
pub async fn get_dev_extension_info(
    handle: AppHandle,
) -> anyhow::Result<Option<String>, CommandError> {
    use usecase::extension_installer::ExtensionInstallerUseCase;

    let installer = ExtensionInstallerUseCase::new(Arc::new(handle));

    if installer.is_dev_extension_available() {
        let dev_path = installer.get_dev_extension_path();
        Ok(Some(dev_path.to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn check_registry_keys(
    handle: AppHandle,
) -> anyhow::Result<Vec<usecase::extension_installer::RegistryKeyInfo>, CommandError> {
    use usecase::extension_installer::ExtensionInstallerUseCase;

    let installer = ExtensionInstallerUseCase::new(Arc::new(handle));
    let result = installer
        .check_registry_keys()
        .map_err(|e| anyhow::anyhow!("Failed to check registry keys: {}", e))?;

    Ok(result)
}

#[tauri::command]
pub async fn remove_registry_keys(handle: AppHandle) -> anyhow::Result<Vec<String>, CommandError> {
    use usecase::extension_installer::ExtensionInstallerUseCase;

    let installer = ExtensionInstallerUseCase::new(Arc::new(handle));
    let result = installer
        .remove_registry_keys()
        .map_err(|e| anyhow::anyhow!("Failed to remove registry keys: {}", e))?;

    Ok(result)
}


