mockall::mock! {
    pub WindowsExtMock {}

    impl domain::windows::WindowsExt for WindowsExtMock {
        type ProcessWindows = domain::windows::process::MockProcessWindows;
        type ShellLink = domain::windows::shell_link::MockShellLink;

        fn process(&self) -> &domain::windows::process::MockProcessWindows;
        fn shell_link(&self) -> &domain::windows::shell_link::MockShellLink;
    }
}
