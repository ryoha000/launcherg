use std::path::PathBuf;

use anyhow::Result;
#[cfg(not(windows))]
use interprocess::local_socket::{GenericFilePath, ToFsName};
#[cfg(windows)]
use interprocess::local_socket::{GenericNamespaced, ToNsName};
use interprocess::local_socket::{ListenerOptions, Name};

#[derive(Debug)]
pub struct ListenerConfig {
    pub options: ListenerOptions<'static>,
    pub cleanup_path: Option<PathBuf>,
}

pub struct AppSignalEndpoint;

impl AppSignalEndpoint {
    #[cfg(not(windows))]
    const DIR_NAME: &'static str = "launcherg";
    #[cfg(not(windows))]
    const FILE_NAME: &'static str = "app-signal.sock";
    #[cfg(windows)]
    const PIPE_NAME: &'static str = "launcherg_app_signal";

    pub fn prepare_listener() -> Result<ListenerConfig> {
        #[cfg(windows)]
        {
            let name = Self::namespaced_name()?;
            let options = ListenerOptions::new().name(name);
            Ok(ListenerConfig {
                options,
                cleanup_path: None,
            })
        }

        #[cfg(not(windows))]
        {
            let (name, path) = Self::path_name()?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if path.exists() {
                std::fs::remove_file(&path).or_else(|err| {
                    if err.kind() == std::io::ErrorKind::NotFound {
                        Ok(())
                    } else {
                        Err(err)
                    }
                })?;
            }

            let options = ListenerOptions::new().name(name.clone());

            Ok(ListenerConfig {
                options,
                cleanup_path: Some(path),
            })
        }
    }

    pub fn connect_name() -> Result<Name<'static>> {
        #[cfg(windows)]
        {
            Self::namespaced_name()
        }

        #[cfg(not(windows))]
        {
            Ok(Self::path_name()?.0)
        }
    }

    #[cfg(windows)]
    fn namespaced_name() -> Result<Name<'static>> {
        // Isolated data directories must not send test notifications to the user's app.
        let name = match std::env::var_os("LAUNCHERG_DATA_DIR").map(PathBuf::from) {
            Some(path) if path.is_absolute() => {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                path.to_string_lossy().to_lowercase().hash(&mut hasher);
                format!("{}_{:016x}", Self::PIPE_NAME, hasher.finish())
            }
            _ => Self::PIPE_NAME.to_string(),
        };
        Ok(name.to_ns_name::<GenericNamespaced>()?.into_owned())
    }

    #[cfg(not(windows))]
    fn path_name() -> Result<(Name<'static>, PathBuf)> {
        let mut base = std::env::temp_dir();
        base.push(Self::DIR_NAME);
        let path = base.join(Self::FILE_NAME);
        let name = path.to_fs_name::<GenericFilePath>()?.into_owned();
        Ok((name, path))
    }
}

#[cfg(test)]
#[path = "endpoint_test.rs"]
mod endpoint_test;
