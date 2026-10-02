//! Native Wayland shortcuts. The compositor owns and consumes the binding;
//! XWayland key snooping cannot suppress input in a native Wayland terminal.

use std::collections::HashMap;
use std::io::Write;
use std::time::Duration;

use futures_util::StreamExt;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};
use zbus::{Connection, Proxy};

use crate::domain::{AppError, Result};

const DESTINATION: &str = "org.freedesktop.portal.Desktop";
const PATH: &str = "/org/freedesktop/portal/desktop";
const INTERFACE: &str = "org.freedesktop.portal.GlobalShortcuts";
const APP_ID: &str = "dev.jay.voice-transcribe";
const SHORTCUT_ID: &str = "dictate";
type Properties = HashMap<String, OwnedValue>;
type Shortcuts = Vec<(String, Properties)>;
type InputShortcuts<'a> = Vec<(&'a str, HashMap<&'a str, Value<'a>>)>;

pub fn is_wayland() -> bool {
    uses_portal(
        std::env::var("XDG_SESSION_TYPE").ok().as_deref(),
        std::env::var("WAYLAND_DISPLAY").ok().as_deref(),
    )
}

fn uses_portal(session_type: Option<&str>, display: Option<&str>) -> bool {
    // GDK_BACKEND=x11 only controls the webview, not the desktop session.
    match session_type {
        Some(value) if value.eq_ignore_ascii_case("wayland") => true,
        Some(value) if value.eq_ignore_ascii_case("x11") => false,
        _ => display.is_some_and(|value| !value.is_empty()),
    }
}

fn portal_error(error: impl std::fmt::Display) -> AppError {
    AppError::SettingsInvalid(format!("Wayland shortcut: {error}"))
}

fn preferred_trigger(hotkey: &str) -> Result<String> {
    use tauri_plugin_global_shortcut::Modifiers;
    let shortcut: tauri_plugin_global_shortcut::Shortcut = hotkey.parse().map_err(portal_error)?;
    let mut output = Vec::new();
    for (modifier, name) in [
        (Modifiers::CONTROL, "CTRL"),
        (Modifiers::ALT, "ALT"),
        (Modifiers::SHIFT, "SHIFT"),
        (Modifiers::SUPER, "LOGO"),
    ] {
        if shortcut.mods.contains(modifier) {
            output.push(name.to_string());
        }
    }
    let code = format!("{:?}", shortcut.key);
    let key = if let Some(letter) = code.strip_prefix("Key") {
        letter.to_ascii_lowercase()
    } else if let Some(digit) = code.strip_prefix("Digit") {
        digit.to_string()
    } else {
        match code.as_str() {
            "Space" => "space",
            "Enter" | "NumpadEnter" => "Return",
            "Backspace" => "BackSpace",
            "ArrowUp" => "Up",
            "ArrowDown" => "Down",
            "ArrowLeft" => "Left",
            "ArrowRight" => "Right",
            "Backquote" => "grave",
            "Minus" => "minus",
            "Equal" => "equal",
            "BracketLeft" => "bracketleft",
            "BracketRight" => "bracketright",
            "Backslash" | "IntlBackslash" => "backslash",
            "Semicolon" => "semicolon",
            "Quote" => "apostrophe",
            "Comma" => "comma",
            "Period" => "period",
            "Slash" => "slash",
            "CapsLock" => "Caps_Lock",
            "NumLock" => "Num_Lock",
            "ScrollLock" => "Scroll_Lock",
            "PrintScreen" => "Print",
            "PageUp" => "Prior",
            "PageDown" => "Next",
            "NumpadAdd" => "KP_Add",
            "NumpadSubtract" => "KP_Subtract",
            "NumpadMultiply" => "KP_Multiply",
            "NumpadDivide" => "KP_Divide",
            "NumpadDecimal" => "KP_Decimal",
            "Escape" | "Tab" | "Home" | "End" | "Insert" | "Delete" | "Pause" => &code,
            value if value.starts_with('F') && value[1..].parse::<u8>().is_ok() => &code,
            value if value.starts_with("Numpad") && value[6..].parse::<u8>().is_ok() => {
                output.push(format!("KP_{}", &value[6..]));
                return Ok(output.join("+"));
            }
            _ => return Err(portal_error(format!("Unsupported preferred key: {code}"))),
        }
        .to_string()
    };
    output.push(key);
    Ok(output.join("+"))
}

fn shortcut_label(shortcuts: &Shortcuts) -> Option<String> {
    shortcuts.iter().find_map(|(id, properties)| {
        if id != SHORTCUT_ID {
            return None;
        }
        properties
            .get("trigger_description")
            .and_then(|value| <&str>::try_from(value).ok())
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    })
}

/// Portal application identity for an unsandboxed AppImage. Only installs
/// metadata, never a helper executable or a system package.
fn ensure_desktop_identity() -> Result<()> {
    let applications = dirs::data_dir()
        .ok_or_else(|| portal_error("Application data directory unavailable"))?
        .join("applications");
    std::fs::create_dir_all(&applications)?;
    let path = applications.join(format!("{APP_ID}.desktop"));
    match std::fs::OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut file) => file.write_all(
            b"[Desktop Entry]\nType=Application\nName=Clanker Yap\nComment=Local voice dictation\nExec=voice-transcribe\nNoDisplay=true\n",
        )?,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

#[derive(Debug)]
pub enum ShortcutEvent {
    Pressed,
    Released,
    Changed(Option<String>),
    Unavailable(String),
}

pub struct PortalSession {
    connection: Connection,
    path: OwnedObjectPath,
    version: u32,
    task: tauri::async_runtime::JoinHandle<()>,
}

impl PortalSession {
    pub fn configuration(&self) -> (Connection, OwnedObjectPath, u32) {
        (self.connection.clone(), self.path.clone(), self.version)
    }

    pub async fn connect(
        hotkey: &str,
        on_event: impl Fn(ShortcutEvent) + Send + Sync + 'static,
    ) -> Result<Self> {
        let trigger = preferred_trigger(hotkey)?;
        ensure_desktop_identity()?;
        let connection = Connection::session().await.map_err(portal_error)?;
        let registry = Proxy::new(
            &connection,
            DESTINATION,
            PATH,
            "org.freedesktop.host.portal.Registry",
        )
        .await
        .map_err(portal_error)?;
        registry
            .call::<_, _, ()>("Register", &(APP_ID, HashMap::<&str, Value<'_>>::new()))
            .await
            .map_err(portal_error)?;
        let proxy = Proxy::new_owned(connection.clone(), DESTINATION, PATH, INTERFACE)
            .await
            .map_err(portal_error)?;
        let version: u32 = proxy.get_property("version").await.map_err(portal_error)?;
        // Subscribe before creating/binding; a fast response must not be lost.
        // One stream preserves press/release order, including quick taps.
        let signals = proxy.receive_all_signals().await.map_err(portal_error)?;
        let mut create_options = HashMap::new();
        create_options.insert("session_handle_token", Value::from(token()));
        let result = request(&connection, &proxy, "CreateSession", create_options, None).await?;
        let path: OwnedObjectPath = result
            .get("session_handle")
            .and_then(|value| <&str>::try_from(value).ok())
            .ok_or_else(|| portal_error("Missing session handle"))?
            .try_into()
            .map_err(portal_error)?;
        let mut properties = HashMap::new();
        properties.insert("description", Value::from("Hold to dictate"));
        properties.insert("preferred_trigger", Value::from(trigger));
        let shortcuts = vec![(SHORTCUT_ID, properties)];
        let bound = request(
            &connection,
            &proxy,
            "BindShortcuts",
            HashMap::new(),
            Some((&path, &shortcuts)),
        )
        .await;
        let bound = match bound {
            Ok(result) => result,
            Err(error) => {
                close_session(&connection, &path).await;
                return Err(error);
            }
        };
        let shortcuts: Shortcuts = bound
            .get("shortcuts")
            .and_then(|value| value.try_clone().ok())
            .and_then(|value| value.try_into().ok())
            .ok_or_else(|| portal_error("Missing bound shortcuts"))?;
        on_event(ShortcutEvent::Changed(shortcut_label(&shortcuts)));
        let session_path = path.clone();
        let session = Proxy::new_owned(
            connection.clone(),
            DESTINATION,
            path.clone(),
            "org.freedesktop.portal.Session",
        )
        .await
        .map_err(portal_error)?;
        let closed = session
            .receive_signal("Closed")
            .await
            .map_err(portal_error)?;
        let events = futures_util::stream::select(
            signals.map(|message| (false, message)),
            closed.map(|message| (true, message)),
        );
        let task = tauri::async_runtime::spawn(async move {
            futures_util::pin_mut!(events);
            while let Some((closed, message)) = events.next().await {
                if closed {
                    on_event(ShortcutEvent::Unavailable(
                        "Desktop shortcut session closed. Restart Clanker Yap to reconnect.".into(),
                    ));
                    return;
                }
                let header = message.header();
                let member = header.member().map(|name| name.as_str());
                match member {
                    Some("ShortcutsChanged") => {
                        if let Ok((event_path, shortcuts)) =
                            message.body().deserialize::<(OwnedObjectPath, Shortcuts)>()
                        {
                            if event_path == session_path {
                                on_event(ShortcutEvent::Changed(shortcut_label(&shortcuts)));
                            }
                        }
                    }
                    Some("Activated" | "Deactivated") => {
                        if let Ok((event_path, id, _, _)) =
                            message
                                .body()
                                .deserialize::<(OwnedObjectPath, String, u64, Properties)>()
                        {
                            if matches_binding(&event_path, &session_path, &id) {
                                on_event(if member == Some("Activated") {
                                    ShortcutEvent::Pressed
                                } else {
                                    ShortcutEvent::Released
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
            on_event(ShortcutEvent::Unavailable(
                "Desktop shortcut service disconnected. Restart Clanker Yap to reconnect.".into(),
            ));
        });
        Ok(Self {
            connection,
            path,
            version,
            task,
        })
    }
}

impl Drop for PortalSession {
    fn drop(&mut self) {
        self.task.abort();
        let connection = self.connection.clone();
        let path = self.path.clone();
        tauri::async_runtime::spawn(async move {
            close_session(&connection, &path).await;
        });
    }
}

fn matches_binding(path: &OwnedObjectPath, expected: &OwnedObjectPath, id: &str) -> bool {
    path == expected && id == SHORTCUT_ID
}

async fn close_session(connection: &Connection, path: &OwnedObjectPath) {
    if let Ok(proxy) = Proxy::new(
        connection,
        DESTINATION,
        path.as_str(),
        "org.freedesktop.portal.Session",
    )
    .await
    {
        let _ = proxy.call::<_, _, ()>("Close", &()).await;
    }
}

pub async fn configure(connection: Connection, path: OwnedObjectPath, version: u32) -> Result<()> {
    if version < 2 {
        return Err(portal_error(
            "Change this binding in your desktop's keyboard shortcut settings.",
        ));
    }
    let proxy = Proxy::new(&connection, DESTINATION, PATH, INTERFACE)
        .await
        .map_err(portal_error)?;
    proxy
        .call::<_, _, ()>(
            "ConfigureShortcuts",
            &(path, "", HashMap::<&str, Value<'_>>::new()),
        )
        .await
        .map_err(portal_error)
}

fn token() -> String {
    format!("yap{}", uuid::Uuid::new_v4().simple())
}

/// Request paths are predictable: subscribe before invoking the portal method.
async fn request(
    connection: &Connection,
    portal: &Proxy<'_>,
    method: &str,
    mut options: HashMap<&str, Value<'_>>,
    binding: Option<(&OwnedObjectPath, &InputShortcuts<'_>)>,
) -> Result<Properties> {
    let token = token();
    let sender = connection
        .unique_name()
        .ok_or_else(|| portal_error("Missing D-Bus name"))?
        .as_str()
        .trim_start_matches(':')
        .replace('.', "_");
    let expected = format!("/org/freedesktop/portal/desktop/request/{sender}/{token}");
    let proxy = Proxy::new(
        connection,
        DESTINATION,
        expected.as_str(),
        "org.freedesktop.portal.Request",
    )
    .await
    .map_err(portal_error)?;
    let mut responses = proxy
        .receive_signal("Response")
        .await
        .map_err(portal_error)?;
    options.insert("handle_token", Value::from(token));
    let returned: OwnedObjectPath = if let Some((path, shortcuts)) = binding {
        portal.call(method, &(path, shortcuts, "", options)).await
    } else {
        portal.call(method, &(options,)).await
    }
    .map_err(portal_error)?;
    if returned.as_str() != expected {
        return Err(portal_error("Unexpected portal request path"));
    }
    let response = tokio::time::timeout(Duration::from_secs(300), responses.next()).await;
    let message = match response {
        Ok(Some(message)) => message,
        _ => {
            let _ = proxy.call::<_, _, ()>("Close", &()).await;
            return Err(portal_error(
                "Shortcut setup timed out or the portal disconnected",
            ));
        }
    };
    let (code, result): (u32, Properties) = message.body().deserialize().map_err(portal_error)?;
    match code {
        0 => Ok(result),
        1 => Err(portal_error("Shortcut setup cancelled")),
        _ => Err(portal_error("The desktop could not bind this shortcut")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBus(std::process::Child);

    impl Drop for TestBus {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    struct FakePortal {
        response_code: std::sync::Arc<std::sync::atomic::AtomicU32>,
    }

    #[zbus::interface(name = "org.freedesktop.portal.GlobalShortcuts")]
    impl FakePortal {
        async fn create_session(
            &self,
            options: Properties,
            #[zbus(connection)] connection: &Connection,
            #[zbus(header)] header: zbus::message::Header<'_>,
        ) -> zbus::fdo::Result<OwnedObjectPath> {
            let token = options
                .get("handle_token")
                .and_then(|value| <&str>::try_from(value).ok())
                .ok_or_else(|| zbus::fdo::Error::InvalidArgs("Missing token".into()))?;
            let sender = header
                .sender()
                .ok_or_else(|| zbus::fdo::Error::InvalidArgs("Missing sender".into()))?;
            let sender_path = sender.as_str().trim_start_matches(':').replace('.', "_");
            let path: OwnedObjectPath =
                format!("/org/freedesktop/portal/desktop/request/{sender_path}/{token}")
                    .try_into()
                    .map_err(|error: zbus::zvariant::Error| {
                        zbus::fdo::Error::Failed(error.to_string())
                    })?;
            let mut result = Properties::new();
            result.insert(
                "session_handle".into(),
                Value::from("/test/session").try_into().map_err(
                    |error: zbus::zvariant::Error| zbus::fdo::Error::Failed(error.to_string()),
                )?,
            );
            // Deliberately emit before replying to exercise the subscription race.
            connection
                .emit_signal(
                    Some(sender.as_str()),
                    path.as_str(),
                    "org.freedesktop.portal.Request",
                    "Response",
                    &(
                        self.response_code
                            .load(std::sync::atomic::Ordering::Relaxed),
                        result,
                    ),
                )
                .await
                .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))?;
            Ok(path)
        }
    }

    #[test]
    fn portal_request_handles_immediate_response_and_cancellation() {
        use std::io::BufRead;
        let mut child = std::process::Command::new("dbus-daemon")
            .args(["--session", "--nofork", "--print-address=1"])
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("private test bus");
        let stdout = child.stdout.take().expect("bus stdout");
        let _bus = TestBus(child);
        let mut address = String::new();
        std::io::BufReader::new(stdout)
            .read_line(&mut address)
            .expect("bus address");
        tauri::async_runtime::block_on(async {
            let code = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
            let _server = zbus::connection::Builder::address(address.trim())
                .expect("address")
                .name(DESTINATION)
                .expect("name")
                .serve_at(
                    PATH,
                    FakePortal {
                        response_code: code.clone(),
                    },
                )
                .expect("interface")
                .build()
                .await
                .expect("server");
            let client = zbus::connection::Builder::address(address.trim())
                .expect("address")
                .build()
                .await
                .expect("client");
            let portal = Proxy::new(&client, DESTINATION, PATH, INTERFACE)
                .await
                .expect("proxy");
            let result = request(&client, &portal, "CreateSession", HashMap::new(), None)
                .await
                .expect("response");
            assert_eq!(
                <&str>::try_from(result.get("session_handle").expect("handle")).expect("string"),
                "/test/session"
            );
            code.store(1, std::sync::atomic::Ordering::Relaxed);
            let error = request(&client, &portal, "CreateSession", HashMap::new(), None)
                .await
                .expect_err("cancelled");
            assert!(error.to_string().contains("cancelled"));
        });
    }

    /// Checks the real desktop protocol without binding a key or opening a dialog.
    #[test]
    #[ignore = "requires a running desktop with the GlobalShortcuts portal"]
    fn native_portal_session_creation_smoke() {
        tauri::async_runtime::block_on(async {
            ensure_desktop_identity().expect("desktop metadata");
            let connection = Connection::session().await.expect("session bus");
            let registry = Proxy::new(
                &connection,
                DESTINATION,
                PATH,
                "org.freedesktop.host.portal.Registry",
            )
            .await
            .expect("registry");
            registry
                .call::<_, _, ()>("Register", &(APP_ID, HashMap::<&str, Value<'_>>::new()))
                .await
                .expect("application identity registration");
            let portal = Proxy::new(&connection, DESTINATION, PATH, INTERFACE)
                .await
                .expect("portal");
            let version: u32 = portal.get_property("version").await.expect("version");
            assert!(version >= 1);
            let mut options = HashMap::new();
            options.insert("session_handle_token", Value::from(token()));
            let response = request(&connection, &portal, "CreateSession", options, None)
                .await
                .expect("session creation");
            let path: OwnedObjectPath =
                <&str>::try_from(response.get("session_handle").expect("handle"))
                    .expect("string")
                    .try_into()
                    .expect("path");
            close_session(&connection, &path).await;
        });
    }

    #[test]
    fn wayland_uses_portal_even_with_an_x11_webview() {
        assert!(uses_portal(Some("wayland"), Some("wayland-0")));
        assert!(uses_portal(None, Some("wayland-0")));
        assert!(!uses_portal(Some("x11"), Some("wayland-0")));
        assert!(!uses_portal(None, None));
    }

    #[test]
    fn translates_saved_hotkeys_to_xdg_triggers() {
        assert_eq!(preferred_trigger("Alt+Q").expect("valid"), "ALT+q");
        assert_eq!(
            preferred_trigger("CmdOrCtrl+Shift+V").expect("valid"),
            "CTRL+SHIFT+v"
        );
        assert_eq!(
            preferred_trigger("Ctrl+Space").expect("valid"),
            "CTRL+space"
        );
        assert_eq!(preferred_trigger("Alt+F8").expect("valid"), "ALT+F8");
        assert_eq!(preferred_trigger("Ctrl+KeyQ").expect("valid"), "CTRL+q");
        assert_eq!(preferred_trigger("Alt+PageUp").expect("valid"), "ALT+Prior");
        assert!(preferred_trigger("not a shortcut").is_err());
    }

    #[test]
    fn unrelated_portal_events_cannot_start_or_stop_capture() {
        let path: OwnedObjectPath = "/session/yap".try_into().expect("valid");
        let other: OwnedObjectPath = "/session/other".try_into().expect("valid");
        assert!(matches_binding(&path, &path, "dictate"));
        assert!(!matches_binding(&other, &path, "dictate"));
        assert!(!matches_binding(&path, &path, "another-action"));
    }

    #[test]
    fn shortcut_label_uses_only_the_desktop_binding_for_dictation() {
        let mut properties = Properties::new();
        properties.insert(
            "trigger_description".into(),
            Value::from("Alt+Q").try_into().expect("owned"),
        );
        assert_eq!(
            shortcut_label(&vec![("dictate".into(), properties)]),
            Some("Alt+Q".into())
        );
        assert_eq!(shortcut_label(&Vec::new()), None);
    }
}
