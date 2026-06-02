//! Active target application detection and paste-mode classification.

use serde::{Deserialize, Serialize};

/// Snapshot of the currently focused top-level window, as reported by a desktop
/// integration such as KWin.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveWindowInfo {
    pub caption: String,
    pub resource_class: String,
    pub resource_name: String,
    pub desktop_file: String,
    pub role: String,
    pub pid: u32,
}

/// Effective paste target for shortcut selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasteTarget {
    Standard,
    Terminal,
}

impl PasteTarget {
    pub fn paste_mode(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Terminal => "terminal",
        }
    }
}

/// Classifies a focused window into the shortcut family it expects.
pub fn classify(info: Option<&ActiveWindowInfo>) -> PasteTarget {
    let Some(info) = info else {
        return PasteTarget::Standard;
    };

    let identifiers = [
        info.resource_class.as_str(),
        info.resource_name.as_str(),
        info.desktop_file.as_str(),
        info.role.as_str(),
    ];

    if identifiers
        .iter()
        .any(|value| is_terminal_identifier(value))
    {
        PasteTarget::Terminal
    } else {
        PasteTarget::Standard
    }
}

fn is_terminal_identifier(value: &str) -> bool {
    let normalized = normalize_identifier(value);
    if normalized.is_empty() {
        return false;
    }

    matches!(
        normalized.as_str(),
        "alacritty"
            | "clankergrid"
            | "comgithubamezinddterm"
            | "commitchellhghostty"
            | "contour"
            | "coolretroterm"
            | "foot"
            | "gnometerminal"
            | "gnometerminalserver"
            | "guake"
            | "ioelementaryterminal"
            | "kgx"
            | "kitty"
            | "konsole"
            | "lxterminal"
            | "mateterminal"
            | "orggnomeconsole"
            | "orggnometerminal"
            | "orgkdekonsole"
            | "orgwezfurlongwezterm"
            | "qterminal"
            | "st"
            | "tabby"
            | "terminator"
            | "terminology"
            | "tilix"
            | "uxterm"
            | "wezterm"
            | "xfce4terminal"
            | "xterm"
            | "yakuake"
    )
}

fn normalize_identifier(value: &str) -> String {
    value
        .trim()
        .trim_end_matches(".desktop")
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

#[cfg(target_os = "linux")]
mod linux {
    use std::fs;
    use std::process::Command;
    use std::sync::Arc;
    use std::time::Duration;

    use parking_lot::Mutex;
    use zbus::interface;

    use super::ActiveWindowInfo;

    pub const DBUS_SERVICE: &str = "com.clanker.Yap";
    pub const DBUS_PATH: &str = "/com/clanker/Yap/ActiveWindow";
    pub const DBUS_INTERFACE: &str = "com.clanker.Yap.ActiveWindow";
    const KWIN_SCRIPT_NAME: &str = "clanker-yap-active-window";

    struct ActiveWindowReceiver {
        active_window: Arc<Mutex<Option<ActiveWindowInfo>>>,
    }

    #[interface(name = "com.clanker.Yap.ActiveWindow")]
    impl ActiveWindowReceiver {
        #[zbus(name = "ActiveWindowChanged")]
        fn active_window_changed(
            &self,
            caption: &str,
            resource_class: &str,
            resource_name: &str,
            desktop_file: &str,
            role: &str,
            pid: i32,
        ) {
            let info = ActiveWindowInfo {
                caption: caption.to_string(),
                resource_class: resource_class.to_string(),
                resource_name: resource_name.to_string(),
                desktop_file: desktop_file.to_string(),
                role: role.to_string(),
                pid: pid.max(0) as u32,
            };

            tracing::debug!(
                resource_class = %info.resource_class,
                resource_name = %info.resource_name,
                desktop_file = %info.desktop_file,
                pid = info.pid,
                "Active window updated"
            );

            *self.active_window.lock() = Some(info);
        }
    }

    pub fn start_tracking(active_window: Arc<Mutex<Option<ActiveWindowInfo>>>) {
        spawn_dbus_receiver(active_window);
        spawn_kwin_helper_loader();
    }

    fn spawn_dbus_receiver(active_window: Arc<Mutex<Option<ActiveWindowInfo>>>) {
        tauri::async_runtime::spawn(async move {
            if let Err(error) = run_dbus_receiver(active_window).await {
                tracing::warn!(error = %error, "Active-window D-Bus receiver unavailable");
            }
        });
    }

    async fn run_dbus_receiver(
        active_window: Arc<Mutex<Option<ActiveWindowInfo>>>,
    ) -> zbus::Result<()> {
        let receiver = ActiveWindowReceiver { active_window };
        let _connection = zbus::connection::Builder::session()?
            .name(DBUS_SERVICE)?
            .serve_at(DBUS_PATH, receiver)?
            .build()
            .await?;

        tracing::info!(
            service = DBUS_SERVICE,
            path = DBUS_PATH,
            interface = DBUS_INTERFACE,
            "Active-window D-Bus receiver started"
        );

        std::future::pending::<()>().await;
        Ok(())
    }

    fn spawn_kwin_helper_loader() {
        std::thread::spawn(|| {
            std::thread::sleep(Duration::from_millis(350));
            if let Err(error) = load_kwin_helper() {
                tracing::warn!(error = %error, "KWin active-window helper unavailable");
            }
        });
    }

    fn load_kwin_helper() -> std::io::Result<()> {
        if !is_kde_session() {
            tracing::debug!("Skipping KWin helper load outside a KDE session");
            return Ok(());
        }

        let Some(qdbus) = qdbus_command() else {
            tracing::debug!("Skipping KWin helper load because qdbus was not found");
            return Ok(());
        };

        let script_path = std::env::temp_dir().join("clanker-yap-active-window.js");
        fs::write(&script_path, kwin_script_source())?;

        let _ = Command::new(qdbus)
            .args([
                "org.kde.KWin",
                "/Scripting",
                "org.kde.kwin.Scripting.unloadScript",
                KWIN_SCRIPT_NAME,
            ])
            .output();

        let load_output = Command::new(qdbus)
            .arg("org.kde.KWin")
            .arg("/Scripting")
            .arg("org.kde.kwin.Scripting.loadScript")
            .arg(&script_path)
            .arg(KWIN_SCRIPT_NAME)
            .output()?;

        if !load_output.status.success() {
            tracing::warn!(
                stderr = %String::from_utf8_lossy(&load_output.stderr),
                "KWin helper script load failed"
            );
            return Ok(());
        }

        let start_output = Command::new(qdbus)
            .args(["org.kde.KWin", "/Scripting", "org.kde.kwin.Scripting.start"])
            .output()?;

        if start_output.status.success() {
            tracing::info!("KWin active-window helper loaded");
        } else {
            tracing::warn!(
                stderr = %String::from_utf8_lossy(&start_output.stderr),
                "KWin helper script start failed"
            );
        }

        Ok(())
    }

    pub fn unload_kwin_helper() {
        let Some(qdbus) = qdbus_command() else {
            return;
        };

        let _ = Command::new(qdbus)
            .args([
                "org.kde.KWin",
                "/Scripting",
                "org.kde.kwin.Scripting.unloadScript",
                KWIN_SCRIPT_NAME,
            ])
            .output();
    }

    fn is_kde_session() -> bool {
        std::env::var("XDG_CURRENT_DESKTOP")
            .map(|desktop| desktop.to_ascii_lowercase().contains("kde"))
            .unwrap_or(false)
            || std::env::var("KDE_FULL_SESSION")
                .map(|value| value.eq_ignore_ascii_case("true") || value == "1")
                .unwrap_or(false)
    }

    fn qdbus_command() -> Option<&'static str> {
        ["qdbus6", "qdbus"]
            .into_iter()
            .find(|command| Command::new(command).arg("--version").output().is_ok())
    }

    fn kwin_script_source() -> String {
        format!(
            r#"
var CLANKER_SERVICE = "{service}";
var CLANKER_PATH = "{path}";
var CLANKER_INTERFACE = "{interface}";

function clankerString(value) {{
    if (value === undefined || value === null) {{
        return "";
    }}
    return String(value);
}}

function clankerPid(value) {{
    var pid = Number(value);
    if (!isFinite(pid) || pid < 0) {{
        return 0;
    }}
    return Math.floor(pid);
}}

function clankerRole(window) {{
    if (!window) {{
        return "";
    }}
    if (window.windowRole !== undefined && window.windowRole !== null) {{
        return String(window.windowRole);
    }}
    if (window.role !== undefined && window.role !== null) {{
        return String(window.role);
    }}
    return "";
}}

function clankerPublishActiveWindow(window) {{
    var active = window;
    if (active === undefined) {{
        active = workspace.activeWindow;
    }}

    if (!active) {{
        callDBus(
            CLANKER_SERVICE,
            CLANKER_PATH,
            CLANKER_INTERFACE,
            "ActiveWindowChanged",
            "", "", "", "", "", 0
        );
        return;
    }}

    callDBus(
        CLANKER_SERVICE,
        CLANKER_PATH,
        CLANKER_INTERFACE,
        "ActiveWindowChanged",
        clankerString(active.caption),
        clankerString(active.resourceClass),
        clankerString(active.resourceName),
        clankerString(active.desktopFileName),
        clankerRole(active),
        clankerPid(active.pid)
    );
}}

workspace.windowActivated.connect(clankerPublishActiveWindow);
clankerPublishActiveWindow(workspace.activeWindow);
"#,
            service = DBUS_SERVICE,
            path = DBUS_PATH,
            interface = DBUS_INTERFACE
        )
    }
}

#[cfg(target_os = "linux")]
pub use linux::{start_tracking, unload_kwin_helper, DBUS_INTERFACE, DBUS_PATH, DBUS_SERVICE};

#[cfg(test)]
mod tests {
    use super::*;

    fn info(resource_class: &str, resource_name: &str, desktop_file: &str) -> ActiveWindowInfo {
        ActiveWindowInfo {
            caption: String::new(),
            resource_class: resource_class.to_string(),
            resource_name: resource_name.to_string(),
            desktop_file: desktop_file.to_string(),
            role: String::new(),
            pid: 0,
        }
    }

    #[test]
    fn missing_window_uses_standard_paste() {
        assert_eq!(classify(None), PasteTarget::Standard);
    }

    #[test]
    fn known_terminal_resource_class_uses_terminal_paste() {
        assert_eq!(
            classify(Some(&info("konsole", "konsole", "org.kde.konsole"))),
            PasteTarget::Terminal
        );
    }

    #[test]
    fn known_terminal_desktop_file_uses_terminal_paste() {
        assert_eq!(
            classify(Some(&info("", "", "com.mitchellh.ghostty.desktop"))),
            PasteTarget::Terminal
        );
    }

    #[test]
    fn normal_app_uses_standard_paste() {
        assert_eq!(
            classify(Some(&info("firefox", "Navigator", "firefox"))),
            PasteTarget::Standard
        );
    }

    #[test]
    fn clanker_grid_uses_terminal_paste() {
        assert_eq!(
            classify(Some(&info("clanker-grid", "clanker-grid", "clanker-grid"))),
            PasteTarget::Terminal
        );
    }
}
