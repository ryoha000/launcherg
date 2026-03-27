#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use std::sync::Arc;

    use domain::file::LnkMetadata;
    use domain::repository::work_lnk::WorkLnk;
    use domain::service::work_registration::MockWorkRegistrationService;
    use domain::windows::shell_link::MockShellLink;
    use domain::works::{Work, WorkDetails};
    use tempfile::tempdir;

    use crate::repositorymock::{TestRepositories, TestRepositoryManager};
    use crate::windowsmock::MockWindowsExtMock as MockWindowsExt;
    use crate::work::{RemoteLaunchReasonCode, WorkUseCase};

    fn build_work_details(work_id: &str) -> WorkDetails {
        WorkDetails {
            work: Work {
                id: domain::StrId::new(work_id.to_string()),
                title: "title".into(),
            },
            dmm: None,
            dlsite: None,
            erogamescape_id: None,
            erogamescape_information: None,
            latest_download_path: None,
            like: None,
            install_at: None,
            original_path: None,
            last_play_at: None,
            registered_at: None,
            thumbnail_size: None,
        }
    }

    fn build_use_case(
        repos: TestRepositories,
        windows: MockWindowsExt,
    ) -> WorkUseCase<TestRepositoryManager, TestRepositories, MockWindowsExt, MockWorkRegistrationService>
    {
        WorkUseCase::new(
            Arc::new(TestRepositoryManager::new(repos)),
            Arc::new(windows),
            Arc::new(MockWorkRegistrationService::new()),
        )
    }

    #[tokio::test]
    async fn launch_remote_work_work_not_foundを返す() {
        let repos = TestRepositories::default();
        {
            let mut work = repos.work.lock().await;
            work.expect_find_details_by_work_id()
                .times(1)
                .returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(None) }));
        }

        let mut windows = MockWindowsExt::new();
        let shell_link = MockShellLink::new();
        windows
            .expect_shell_link()
            .return_const(shell_link);

        let uc = build_use_case(repos, windows);
        let ack = uc.launch_remote_work("missing-work".into()).await;

        assert!(!ack.ok);
        assert_eq!(ack.reason_code, RemoteLaunchReasonCode::WorkNotFound);
        assert_eq!(ack.message, RemoteLaunchReasonCode::WorkNotFound.message());
        assert_eq!(ack.detail, None);
    }

    #[tokio::test]
    async fn launch_remote_work_shortcut_not_foundを返す() {
        let repos = TestRepositories::default();
        {
            let mut work = repos.work.lock().await;
            work.expect_find_details_by_work_id()
                .times(1)
                .returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(Some(build_work_details("empty-work"))) }));
        }
        {
            let mut lnk = repos.work_lnk.lock().await;
            lnk.expect_list_by_work_id()
                .times(1)
                .returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(Vec::new()) }));
        }

        let mut windows = MockWindowsExt::new();
        let shell_link = MockShellLink::new();
        windows
            .expect_shell_link()
            .return_const(shell_link);

        let uc = build_use_case(repos, windows);
        let ack = uc.launch_remote_work("empty-work".into()).await;

        assert!(!ack.ok);
        assert_eq!(ack.reason_code, RemoteLaunchReasonCode::ShortcutNotFound);
        assert_eq!(ack.message, RemoteLaunchReasonCode::ShortcutNotFound.message());
        assert_eq!(ack.detail, None);
    }

    #[tokio::test]
    async fn launch_remote_work_target_not_installedを返す() {
        let tempdir = tempdir().expect("tempdir");
        let lnk_path = tempdir.path().join("missing-target.lnk");
        let target_path = tempdir.path().join("missing-target.exe");
        let _ = std::fs::File::create(&lnk_path).expect("lnk file");

        let repos = TestRepositories::default();
        {
            let mut work = repos.work.lock().await;
            work.expect_find_details_by_work_id()
                .times(1)
                .returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(Some(build_work_details("missing-target-work"))) }));
        }
        {
            let mut lnk = repos.work_lnk.lock().await;
            let lnk_path_value = lnk_path.display().to_string();
            lnk.expect_list_by_work_id()
                .times(1)
                .returning(move |_| {
                    let lnk_path_value = lnk_path_value.clone();
                    Box::pin(async move {
                        Ok::<_, anyhow::Error>(vec![WorkLnk {
                            id: domain::Id::new(1),
                            work_id: domain::StrId::new("missing-target-work".into()),
                            lnk_path: lnk_path_value,
                        }])
                    })
                });
        }

        let mut windows = MockWindowsExt::new();
        let mut shell_link = MockShellLink::new();
        shell_link
            .expect_get_lnk_metadatas()
            .times(1)
            .returning({
                let lnk_path = lnk_path.clone();
                let target_path = target_path.clone();
                move |paths| {
                    assert_eq!(paths, vec![lnk_path.display().to_string()]);
                    let mut map = std::collections::HashMap::new();
                    map.insert(
                        lnk_path.display().to_string(),
                        LnkMetadata {
                            path: target_path.display().to_string(),
                            icon: String::new(),
                        },
                    );
                    Ok(map)
                }
            });
        shell_link.expect_execute_lnk().times(0);
        windows
            .expect_shell_link()
            .return_const(shell_link);

        let uc = build_use_case(repos, windows);
        let ack = uc.launch_remote_work("missing-target-work".into()).await;

        assert!(!ack.ok);
        assert_eq!(ack.reason_code, RemoteLaunchReasonCode::TargetNotInstalled);
        assert_eq!(ack.message, RemoteLaunchReasonCode::TargetNotInstalled.message());
        assert_eq!(
            ack.detail,
            Some("shortcut target path does not exist: ".to_string() + &target_path.display().to_string())
        );
    }

    #[tokio::test]
    async fn launch_remote_work_launch_failedを返す() {
        let tempdir = tempdir().expect("tempdir");
        let lnk_path = tempdir.path().join("launch-failed.lnk");
        let target_path = tempdir.path().join("launch-failed.exe");
        let _ = std::fs::File::create(&lnk_path).expect("lnk file");
        let _ = std::fs::File::create(&target_path).expect("target file");

        let repos = TestRepositories::default();
        {
            let mut work = repos.work.lock().await;
            work.expect_find_details_by_work_id()
                .times(1)
                .returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(Some(build_work_details("launch-failed-work"))) }));
        }
        {
            let mut lnk = repos.work_lnk.lock().await;
            let lnk_path_value = lnk_path.display().to_string();
            lnk.expect_list_by_work_id()
                .times(1)
                .returning(move |_| {
                    let lnk_path_value = lnk_path_value.clone();
                    Box::pin(async move {
                        Ok::<_, anyhow::Error>(vec![WorkLnk {
                            id: domain::Id::new(1),
                            work_id: domain::StrId::new("launch-failed-work".into()),
                            lnk_path: lnk_path_value,
                        }])
                    })
                });
            lnk.expect_find_by_id()
                .times(1)
                .returning({
                    let lnk_path = lnk_path.clone();
                    move |_| {
                        let lnk_path = lnk_path.display().to_string();
                        Box::pin(async move {
                            Ok::<_, anyhow::Error>(Some(WorkLnk {
                                id: domain::Id::new(1),
                                work_id: domain::StrId::new("launch-failed-work".into()),
                                lnk_path,
                            }))
                        })
                    }
                });
        }

        let mut windows = MockWindowsExt::new();
        let mut shell_link = MockShellLink::new();
        shell_link
            .expect_get_lnk_metadatas()
            .times(1)
            .returning({
                let lnk_path = lnk_path.clone();
                let target_path = target_path.clone();
                move |paths| {
                    assert_eq!(paths, vec![lnk_path.display().to_string()]);
                    let mut map = std::collections::HashMap::new();
                    map.insert(
                        lnk_path.display().to_string(),
                        LnkMetadata {
                            path: target_path.display().to_string(),
                            icon: String::new(),
                        },
                    );
                    Ok(map)
                }
            });
        shell_link
            .expect_execute_lnk()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("ShellExecuteExW failed")));
        windows
            .expect_shell_link()
            .return_const(shell_link);

        let uc = build_use_case(repos, windows);
        let ack = uc.launch_remote_work("launch-failed-work".into()).await;

        assert!(!ack.ok);
        assert_eq!(ack.reason_code, RemoteLaunchReasonCode::LaunchFailed);
        assert_eq!(ack.message, RemoteLaunchReasonCode::LaunchFailed.message());
        assert_eq!(ack.detail, Some("ShellExecuteExW failed".into()));
    }

    #[tokio::test]
    async fn launch_remote_work_launchedを返す() {
        let tempdir = tempdir().expect("tempdir");
        let lnk_path = tempdir.path().join("launched.lnk");
        let target_path = tempdir.path().join("launched.exe");
        let _ = std::fs::File::create(&lnk_path).expect("lnk file");
        let _ = std::fs::File::create(&target_path).expect("target file");

        let repos = TestRepositories::default();
        {
            let mut work = repos.work.lock().await;
            work.expect_find_details_by_work_id()
                .times(1)
                .returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(Some(build_work_details("launched-work"))) }));
            work.expect_update_last_play_at_by_work_id()
                .times(1)
                .returning(|_, _| Box::pin(async { Ok::<_, anyhow::Error>(()) }));
        }
        {
            let mut lnk = repos.work_lnk.lock().await;
            let lnk_path_value = lnk_path.display().to_string();
            lnk.expect_list_by_work_id()
                .times(1)
                .returning(move |_| {
                    let lnk_path_value = lnk_path_value.clone();
                    Box::pin(async move {
                        Ok::<_, anyhow::Error>(vec![WorkLnk {
                            id: domain::Id::new(1),
                            work_id: domain::StrId::new("launched-work".into()),
                            lnk_path: lnk_path_value,
                        }])
                    })
                });
            lnk.expect_find_by_id()
                .times(1)
                .returning({
                    let lnk_path = lnk_path.clone();
                    move |_| {
                        let lnk_path = lnk_path.display().to_string();
                        Box::pin(async move {
                            Ok::<_, anyhow::Error>(Some(WorkLnk {
                                id: domain::Id::new(1),
                                work_id: domain::StrId::new("launched-work".into()),
                                lnk_path,
                            }))
                        })
                    }
                });
        }

        let mut windows = MockWindowsExt::new();
        let mut shell_link = MockShellLink::new();
        shell_link
            .expect_get_lnk_metadatas()
            .times(1)
            .returning({
                let lnk_path = lnk_path.clone();
                let target_path = target_path.clone();
                move |paths| {
                    assert_eq!(paths, vec![lnk_path.display().to_string()]);
                    let mut map = std::collections::HashMap::new();
                    map.insert(
                        lnk_path.display().to_string(),
                        LnkMetadata {
                            path: target_path.display().to_string(),
                            icon: String::new(),
                        },
                    );
                    Ok(map)
                }
            });
        shell_link
            .expect_execute_lnk()
            .times(1)
            .returning(|_, _| Ok(Some(1234)));
        windows
            .expect_shell_link()
            .return_const(shell_link);

        let uc = build_use_case(repos, windows);
        let ack = uc.launch_remote_work("launched-work".into()).await;

        assert!(ack.ok);
        assert_eq!(ack.reason_code, RemoteLaunchReasonCode::Launched);
        assert_eq!(ack.message, RemoteLaunchReasonCode::Launched.message());
        assert_eq!(ack.detail, Some("process id: 1234".into()));
    }
}
