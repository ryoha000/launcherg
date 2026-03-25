pub mod process;
pub mod shell_link;

pub trait WindowsExt {
    type ProcessWindows: process::ProcessWindows;
    type ShellLink: shell_link::ShellLink;

    fn process(&self) -> &Self::ProcessWindows;
    fn shell_link(&self) -> &Self::ShellLink;
}
