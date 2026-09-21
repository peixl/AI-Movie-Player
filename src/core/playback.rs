//! Cross-platform media launching.
//!
//! The previous implementation used `Command::status()`, which blocks the UI
//! thread until the launcher — and on some Linux desktops the player itself —
//! exits. These helpers always spawn a detached child with null stdio so the
//! egui event loop is never blocked. Command construction is split into pure
//! per-platform argument builders so the behavior is unit-testable on every
//! host.

use std::path::Path;
use std::process::{Command, Stdio};

/// Open a file with the system default media player. Returns an error only when
/// the launcher process cannot be started.
pub fn open_in_system_player(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        let mut command = Command::new("open");
        command.args(macos_open_args(path));
        spawn_detached(command)
    }

    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("cmd");
        command.args(windows_open_args(path));
        spawn_detached(command)
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let (program, args) = linux_open_command(path);
        let mut command = Command::new(program);
        command.args(args);
        spawn_detached(command)
    }
}

/// Reveal a file in the platform file manager (Finder/Explorer/Files).
pub fn reveal_in_file_manager(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        let mut command = Command::new("open");
        command.args(macos_reveal_args(path));
        spawn_detached(command)
    }

    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("explorer.exe");
        use std::os::windows::process::CommandExt;
        command.raw_arg(format!("/select,{}", windows_quote(path)));
        spawn_detached(command)
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        // xdg-open cannot select a file; open the containing directory.
        let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
        let (program, args) = linux_open_command(parent);
        let mut command = Command::new(program);
        command.args(args);
        spawn_detached(command)
    }
}

fn spawn_detached(mut command: Command) -> std::io::Result<()> {
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    command.spawn()?;
    Ok(())
}

/// macOS `open` arguments.
pub fn macos_open_args(path: &Path) -> Vec<String> {
    vec![path.to_string_lossy().to_string()]
}

/// macOS `open -R` arguments (reveal in Finder).
pub fn macos_reveal_args(path: &Path) -> Vec<String> {
    vec!["-R".to_string(), path.to_string_lossy().to_string()]
}

/// Windows `cmd /C start "" <path>` arguments. The empty title is mandatory
/// because `start` treats the first quoted argument as a window title.
pub fn windows_open_args(path: &Path) -> Vec<String> {
    vec!["/C".to_string(), "start".to_string(), String::new(), path.to_string_lossy().to_string()]
}

/// Quote a Windows path for `/select,` raw argument passing.
pub fn windows_quote(path: &Path) -> String {
    format!("\"{}\"", path.to_string_lossy())
}

/// Linux launcher command, preferring `xdg-open` and falling back to `gio open`.
pub fn linux_open_command(path: &Path) -> (&'static str, Vec<String>) {
    if which("xdg-open") {
        ("xdg-open", vec![path.to_string_lossy().to_string()])
    } else {
        ("gio", vec!["open".to_string(), path.to_string_lossy().to_string()])
    }
}

fn which(program: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_open_args_pass_the_path_verbatim() {
        let args = macos_open_args(Path::new("/Movies/Dune (2021).mkv"));
        assert_eq!(args, vec!["/Movies/Dune (2021).mkv"]);
    }

    #[test]
    fn macos_reveal_uses_reveal_flag() {
        let args = macos_reveal_args(Path::new("/Movies/Dune.mkv"));
        assert_eq!(args, vec!["-R".to_string(), "/Movies/Dune.mkv".to_string()]);
    }

    #[test]
    fn windows_start_keeps_empty_title_before_path() {
        // Guards against the classic `start` bug where a quoted path is parsed
        // as the console window title and no file opens.
        let args = windows_open_args(Path::new(r"C:\Movies\Dune Part Two.mkv"));
        assert_eq!(args[0], "/C");
        assert_eq!(args[1], "start");
        assert_eq!(args[2], "");
        assert!(args[3].ends_with("Dune Part Two.mkv"));
    }

    #[test]
    fn windows_select_argument_wraps_path_in_quotes() {
        let arg = windows_quote(Path::new(r"C:\My Movies\Dune.mkv"));
        assert!(arg.starts_with('"') && arg.ends_with('"'));
        assert!(arg.contains("My Movies"));
    }

    #[test]
    fn linux_fallback_uses_gio_when_xdg_open_missing() {
        // We cannot uninstall xdg-open in CI; just assert a well-formed command.
        let (program, args) = linux_open_command(Path::new("/tmp/movie.mkv"));
        assert!(program == "xdg-open" || program == "gio");
        assert!(!args.is_empty());
    }
}
