mod config;
mod input;
mod market;
mod matcher;
mod pet;

use config::{AppConfig, AppLocale, PositionMode, TriggerCommand, TriggerModifier};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, RwLock,
    },
};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, State,
};
use tauri_plugin_autostart::ManagerExt;

#[cfg(target_os = "macos")]
#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOHIDRequestAccess(request_type: u32) -> bool;
}

#[cfg(target_os = "macos")]
#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {
    static NSAppKitVersionNumber: f64;
}

#[derive(Clone)]
struct AppState {
    config: Arc<RwLock<AppConfig>>,
    config_path: PathBuf,
    paused: Arc<AtomicBool>,
    recording_command: Arc<AtomicBool>,
    listener_running: Arc<AtomicBool>,
    listener_error: Arc<RwLock<Option<String>>>,
    trigger_count: Arc<AtomicU64>,
    overlay_render_count: Arc<AtomicU64>,
    overlay_ready: Arc<AtomicBool>,
}

#[derive(Clone)]
struct TrayMenuState {
    status: MenuItem<tauri::Wry>,
    settings: MenuItem<tauri::Wry>,
    update: MenuItem<tauri::Wry>,
    test: MenuItem<tauri::Wry>,
    pause: MenuItem<tauri::Wry>,
    autostart: CheckMenuItem<tauri::Wry>,
    position: Submenu<tauri::Wry>,
    position_top_center: CheckMenuItem<tauri::Wry>,
    position_random: CheckMenuItem<tauri::Wry>,
    quit: MenuItem<tauri::Wry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeStatus {
    paused: bool,
    listener_running: bool,
    listener_error: Option<String>,
    platform: &'static str,
    input_scope: &'static str,
    trigger_count: u64,
    overlay_render_count: u64,
    overlay_ready: bool,
}

#[tauri::command]
fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    state
        .config
        .read()
        .map(|value| value.clone())
        .map_err(|_| "配置读取锁已损坏".into())
}

#[tauri::command]
fn save_config(
    config: AppConfig,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<AppConfig, String> {
    let config = config.normalize_and_validate()?;
    config::save(&state.config_path, &config)?;
    *state
        .config
        .write()
        .map_err(|_| "配置写入锁已损坏".to_string())? = config.clone();
    refresh_tray(&app);
    Ok(config)
}

#[tauri::command]
fn list_codex_pets(app: AppHandle) -> Vec<pet::PetChoice> {
    pet::discover(&app)
}

#[tauri::command]
fn select_pet(
    manifest_path: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<AppConfig, String> {
    let selected = manifest_path
        .as_deref()
        .map(pet::validate_manifest_path)
        .transpose()?;
    let mut config = state
        .config
        .read()
        .map_err(|_| "配置读取锁已损坏".to_string())?
        .clone();
    config.pet_manifest_path = selected;
    config::save(&state.config_path, &config)?;
    *state
        .config
        .write()
        .map_err(|_| "配置写入锁已损坏".to_string())? = config.clone();
    refresh_tray(&app);
    Ok(config)
}

#[tauri::command]
fn get_pet(state: State<'_, AppState>) -> Result<pet::PetPayload, String> {
    let path = state
        .config
        .read()
        .map_err(|_| "配置读取锁已损坏".to_string())?
        .pet_manifest_path
        .clone();
    pet::selected_payload(path.as_deref())
}

#[tauri::command]
fn get_pet_thumbnail(manifest_path: Option<String>) -> Result<pet::PetThumbnail, String> {
    let selected = manifest_path
        .as_deref()
        .map(pet::validate_manifest_path)
        .transpose()?;
    pet::selected_thumbnail(selected.as_deref())
}

#[tauri::command]
fn get_pet_preview(manifest_path: Option<String>) -> Result<pet::PetPayload, String> {
    pet::preview_payload(manifest_path.as_deref())
}

#[tauri::command]
fn set_paused(paused: bool, app: AppHandle, state: State<'_, AppState>) -> bool {
    state.paused.store(paused, Ordering::Relaxed);
    let _ = app.emit("pause-changed", paused);
    refresh_tray(&app);
    paused
}

#[tauri::command]
fn set_command_recording(recording: bool, state: State<'_, AppState>) {
    state.recording_command.store(recording, Ordering::Relaxed);
}

#[tauri::command]
fn get_autostart_enabled(app: AppHandle) -> Result<bool, String> {
    app.autolaunch()
        .is_enabled()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn set_autostart_enabled(enabled: bool, app: AppHandle) -> Result<bool, String> {
    if enabled {
        app.autolaunch().enable()
    } else {
        app.autolaunch().disable()
    }
    .map_err(|error| error.to_string())?;
    refresh_tray(&app);
    Ok(enabled)
}

#[tauri::command]
fn get_runtime_status(state: State<'_, AppState>) -> RuntimeStatus {
    let english = state
        .config
        .read()
        .is_ok_and(|config| config.locale == AppLocale::En);
    RuntimeStatus {
        paused: state.paused.load(Ordering::Relaxed),
        listener_running: state.listener_running.load(Ordering::Relaxed),
        listener_error: state
            .listener_error
            .read()
            .ok()
            .and_then(|value| value.clone()),
        platform: std::env::consts::OS,
        input_scope: if english {
            "Only recent command steps are kept; input fields, screens, and the clipboard are never read"
        } else {
            "只保留最近的指令步骤；不读取输入框、屏幕或剪贴板"
        },
        trigger_count: state.trigger_count.load(Ordering::Relaxed),
        overlay_render_count: state.overlay_render_count.load(Ordering::Relaxed),
        overlay_ready: state.overlay_ready.load(Ordering::Relaxed),
    }
}

#[tauri::command]
fn request_input_monitoring_access() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        // kIOHIDRequestTypeListenEvent. Unlike IOHIDCheckAccess, this registers
        // the current app with TCC and presents the system authorization flow.
        IOHIDRequestAccess(1)
    }

    #[cfg(not(target_os = "macos"))]
    true
}

#[tauri::command]
fn test_trigger(app: AppHandle) -> Result<(), String> {
    input::emit_trigger(&app, "preview")
}

#[tauri::command]
fn report_overlay_rendered(state: State<'_, AppState>) {
    state.overlay_render_count.fetch_add(1, Ordering::Relaxed);
}

#[tauri::command]
fn report_overlay_ready(state: State<'_, AppState>) {
    state.overlay_ready.store(true, Ordering::Relaxed);
}

#[tauri::command]
fn show_overlay(app: AppHandle) -> Result<(), String> {
    let overlay = app
        .get_webview_window("overlay")
        .ok_or_else(|| "Overlay window is unavailable".to_string())?;
    configure_overlay(&overlay, true).map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_market_pets(
    refresh: Option<bool>,
    app: AppHandle,
) -> Result<market::MarketCatalog, String> {
    market::list(&app, refresh.unwrap_or(false)).await
}

#[tauri::command]
async fn install_market_pet(
    id: String,
    app: AppHandle,
) -> Result<market::MarketInstallResult, String> {
    market::install(&app, &id).await
}

#[tauri::command]
async fn install_top_market_pets(app: AppHandle) -> Result<market::MarketInstallSummary, String> {
    market::install_all(&app).await
}

#[tauri::command]
fn import_pet_zip(
    archive_path: String,
    app: AppHandle,
) -> Result<market::MarketInstallResult, String> {
    market::import_zip(&app, &archive_path)
}

fn install_tray(app: &tauri::App) -> tauri::Result<()> {
    let config = app
        .state::<AppState>()
        .config
        .read()
        .map(|value| value.clone())
        .unwrap_or_default();
    let english = config.locale == AppLocale::En;
    let status = MenuItem::with_id(
        app,
        "status",
        if english { "Starting" } else { "正在启动" },
        false,
        None::<&str>,
    )?;
    let separator_1 = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(
        app,
        "settings",
        if english {
            "Open settings…"
        } else {
            "打开设置…"
        },
        true,
        None::<&str>,
    )?;
    let test = MenuItem::with_id(
        app,
        "test",
        if english {
            "Test animation"
        } else {
            "测试动画"
        },
        true,
        None::<&str>,
    )?;
    let update = MenuItem::with_id(
        app,
        "update",
        if english {
            "Check for updates…"
        } else {
            "检查更新…"
        },
        true,
        None::<&str>,
    )?;
    let pause = MenuItem::with_id(
        app,
        "pause",
        if english { "Pause" } else { "暂停" },
        true,
        None::<&str>,
    )?;
    let autostart = CheckMenuItem::with_id(
        app,
        "autostart",
        if english {
            "Launch at login"
        } else {
            "开机自动启动"
        },
        true,
        app.autolaunch().is_enabled().unwrap_or(false),
        None::<&str>,
    )?;
    let position_top_center = CheckMenuItem::with_id(
        app,
        "position-top-center",
        if english {
            "Top center"
        } else {
            "顶部居中"
        },
        true,
        config.position.mode == PositionMode::TopCenter,
        None::<&str>,
    )?;
    let position_random = CheckMenuItem::with_id(
        app,
        "position-random",
        if english { "Random" } else { "随机位置" },
        true,
        config.position.mode == PositionMode::Random,
        None::<&str>,
    )?;
    let position = Submenu::with_items(
        app,
        if english { "Position" } else { "出现位置" },
        true,
        &[&position_top_center, &position_random],
    )?;
    let separator_2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(
        app,
        "quit",
        if english {
            "Quit Con Pet"
        } else {
            "退出 Con Pet"
        },
        true,
        None::<&str>,
    )?;
    let menu = Menu::with_items(
        app,
        &[
            &status,
            &separator_1,
            &settings,
            &update,
            &test,
            &pause,
            &autostart,
            &position,
            &separator_2,
            &quit,
        ],
    )?;
    app.manage(TrayMenuState {
        status,
        settings,
        update,
        test,
        pause,
        autostart,
        position,
        position_top_center,
        position_random,
        quit,
    });
    let mut builder = TrayIconBuilder::with_id("pc-pet-tray")
        .menu(&menu)
        .tooltip("Con Pet")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => show_settings(app),
            "update" => {
                show_settings(app);
                let _ = app.emit("check-update-requested", ());
            }
            "test" => {
                let _ = input::emit_trigger(app, "tray");
            }
            "pause" => {
                let state = app.state::<AppState>();
                let paused = !state.paused.load(Ordering::Relaxed);
                state.paused.store(paused, Ordering::Relaxed);
                let _ = app.emit("pause-changed", paused);
                refresh_tray(app);
            }
            "autostart" => {
                let enabled = app.autolaunch().is_enabled().unwrap_or(false);
                let result = if enabled {
                    app.autolaunch().disable()
                } else {
                    app.autolaunch().enable()
                };
                if result.is_ok() {
                    let _ = app.emit("autostart-changed", !enabled);
                }
                refresh_tray(app);
            }
            "position-top-center" => {
                let _ = set_quick_position(app, PositionMode::TopCenter);
            }
            "position-random" => {
                let _ = set_quick_position(app, PositionMode::Random);
            }
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    refresh_tray(app.handle());
    Ok(())
}

fn set_quick_position(app: &AppHandle, mode: PositionMode) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut config = state
        .config
        .read()
        .map_err(|_| "配置读取锁已损坏".to_string())?
        .clone();
    config.position.mode = mode;
    config::save(&state.config_path, &config)?;
    *state
        .config
        .write()
        .map_err(|_| "配置写入锁已损坏".to_string())? = config.clone();
    let _ = app.emit("config-changed", config);
    refresh_tray(app);
    Ok(())
}

pub(crate) fn refresh_tray(app: &AppHandle) {
    let state = app.state::<AppState>();
    let Ok(config) = state.config.read().map(|value| value.clone()) else {
        return;
    };
    let tray = app.state::<TrayMenuState>();
    let paused = state.paused.load(Ordering::Relaxed);
    let listener_running = state.listener_running.load(Ordering::Relaxed);
    let listener_failed = state
        .listener_error
        .read()
        .ok()
        .is_some_and(|value| value.is_some());
    let english = config.locale == AppLocale::En;
    let trigger_label = config
        .trigger_command
        .as_ref()
        .map(|command| {
            format_trigger_command(
                command,
                cfg!(target_os = "macos"),
                cfg!(target_os = "windows"),
            )
        })
        .unwrap_or_else(|| "M → J".into());
    let status = if !config.enabled {
        if english {
            format!("Disabled · Command: {trigger_label}")
        } else {
            format!("已停用 · 指令：{trigger_label}")
        }
    } else if listener_failed {
        if english {
            format!("Listener failed · Command: {trigger_label}")
        } else {
            format!("监听失败 · 指令：{trigger_label}")
        }
    } else if paused {
        if english {
            format!("Paused · Command: {trigger_label}")
        } else {
            format!("已暂停 · 指令：{trigger_label}")
        }
    } else if listener_running {
        if english {
            format!("Listening · Command: {trigger_label}")
        } else {
            format!("监听中 · 指令：{trigger_label}")
        }
    } else if english {
        format!("Starting · Command: {trigger_label}")
    } else {
        format!("正在启动 · 指令：{trigger_label}")
    };
    let _ = tray.status.set_text(status);
    let _ = tray.settings.set_text(if english {
        "Open settings…"
    } else {
        "打开设置…"
    });
    let _ = tray.test.set_text(if english {
        "Test animation"
    } else {
        "测试动画"
    });
    let _ = tray.update.set_text(if english {
        "Check for updates…"
    } else {
        "检查更新…"
    });
    let _ = tray.pause.set_text(if paused && english {
        "Resume"
    } else if paused {
        "恢复"
    } else if english {
        "Pause"
    } else {
        "暂停"
    });
    let _ = tray.autostart.set_text(if english {
        "Launch at login"
    } else {
        "开机自动启动"
    });
    let _ = tray
        .position
        .set_text(if english { "Position" } else { "出现位置" });
    let _ = tray.position_top_center.set_text(if english {
        "Top center"
    } else {
        "顶部居中"
    });
    let _ = tray
        .position_random
        .set_text(if english { "Random" } else { "随机位置" });
    let _ = tray.quit.set_text(if english {
        "Quit Con Pet"
    } else {
        "退出 Con Pet"
    });
    let _ = tray
        .autostart
        .set_checked(app.autolaunch().is_enabled().unwrap_or(false));
    let _ = tray
        .position_top_center
        .set_checked(config.position.mode == PositionMode::TopCenter);
    let _ = tray
        .position_random
        .set_checked(config.position.mode == PositionMode::Random);
}

fn format_trigger_command(command: &TriggerCommand, macos: bool, windows: bool) -> String {
    let mut steps = command
        .steps
        .iter()
        .take(6)
        .map(|step| {
            let mut tokens = step
                .modifiers
                .iter()
                .map(|modifier| match modifier {
                    TriggerModifier::Primary if macos => "⌘",
                    TriggerModifier::Primary => "Ctrl",
                    TriggerModifier::Control if macos => "⌃",
                    TriggerModifier::Control => "Control",
                    TriggerModifier::Alt if macos => "⌥",
                    TriggerModifier::Alt => "Alt",
                    TriggerModifier::Shift if macos => "⇧",
                    TriggerModifier::Shift => "Shift",
                    TriggerModifier::Meta if windows => "Win",
                    TriggerModifier::Meta => "Meta",
                })
                .collect::<Vec<_>>();
            tokens.push(step.key.display_label());
            tokens.join(" + ")
        })
        .collect::<Vec<_>>();
    if command.steps.len() > 6 {
        steps.push("…".into());
    }
    steps.join(" → ")
}

fn show_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(target_os = "macos")]
const APPKIT_VERSION_MACOS_13: f64 = 2299.0;

#[cfg(target_os = "macos")]
fn overlay_collection_behavior(appkit_version: f64) -> u64 {
    // CanJoinAllSpaces | Stationary | FullScreenAuxiliary.
    let mut behavior = (1 << 0) | (1 << 4) | (1 << 8);
    if appkit_version >= APPKIT_VERSION_MACOS_13 {
        // CanJoinAllApplications is the cross-app fullscreen/Stage Manager
        // behavior intended for floating windows and system overlays.
        behavior |= 1 << 18;
    }
    behavior
}

#[cfg(target_os = "macos")]
fn configure_overlay(window: &tauri::WebviewWindow, order_front: bool) -> tauri::Result<()> {
    use objc::{
        msg_send,
        runtime::{Object, NO},
        sel, sel_impl,
    };

    let pointer = window.ns_window()? as usize;
    let appkit_version = unsafe { NSAppKitVersionNumber };
    let behavior = overlay_collection_behavior(appkit_version);
    window.run_on_main_thread(move || unsafe {
        let pointer = pointer as *mut Object;
        // Screen-saver level keeps the animation above normal and fullscreen
        // app windows without using the system shielding/lock-screen level.
        let _: () = msg_send![pointer, setLevel: 1000_i64];
        let _: () = msg_send![pointer, setCollectionBehavior: behavior];
        let _: () = msg_send![pointer, setHidesOnDeactivate: NO];
        let _: () = msg_send![pointer, setCanHide: NO];
        if order_front {
            // Unlike Tauri's normal show path, this works while another app is
            // active and does not make the click-through overlay key.
            let _: () = msg_send![pointer, orderFrontRegardless];
        }
    })
}

#[cfg(not(target_os = "macos"))]
fn configure_overlay(window: &tauri::WebviewWindow, order_front: bool) -> tauri::Result<()> {
    if order_front {
        window.show()?;
    }
    Ok(())
}

#[cfg(all(test, target_os = "macos"))]
mod overlay_tests {
    use super::{overlay_collection_behavior, APPKIT_VERSION_MACOS_13};

    const CAN_JOIN_ALL_APPLICATIONS: u64 = 1 << 18;

    #[test]
    fn enables_cross_app_fullscreen_behavior_on_macos_13_and_newer() {
        let before_macos_13 = overlay_collection_behavior(APPKIT_VERSION_MACOS_13 - 0.1);
        let macos_13 = overlay_collection_behavior(APPKIT_VERSION_MACOS_13);

        assert_eq!(before_macos_13 & CAN_JOIN_ALL_APPLICATIONS, 0);
        assert_ne!(macos_13 & CAN_JOIN_ALL_APPLICATIONS, 0);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_macos_permissions::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.handle()
                .set_activation_policy(tauri::ActivationPolicy::Accessory)?;

            let config_dir = app.path().app_config_dir()?;
            let config_path = config_dir.join("config.json");
            let state = AppState {
                config: Arc::new(RwLock::new(config::load(&config_path))),
                config_path,
                paused: Arc::new(AtomicBool::new(false)),
                recording_command: Arc::new(AtomicBool::new(false)),
                listener_running: Arc::new(AtomicBool::new(false)),
                listener_error: Arc::new(RwLock::new(None)),
                trigger_count: Arc::new(AtomicU64::new(0)),
                overlay_render_count: Arc::new(AtomicU64::new(0)),
                overlay_ready: Arc::new(AtomicBool::new(false)),
            };
            app.manage(state.clone());
            install_tray(app)?;

            if let Some(overlay) = app.get_webview_window("overlay") {
                overlay.set_ignore_cursor_events(true)?;
                configure_overlay(&overlay, false)?;
            }
            input::start(app.handle().clone(), state);
            if std::env::args().any(|argument| argument == "--background") {
                if let Some(main) = app.get_webview_window("main") {
                    main.hide()?;
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if matches!(event, tauri::WindowEvent::Focused(false)) {
                    window
                        .state::<AppState>()
                        .recording_command
                        .store(false, Ordering::Relaxed);
                }
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    window
                        .state::<AppState>()
                        .recording_command
                        .store(false, Ordering::Relaxed);
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            list_codex_pets,
            select_pet,
            get_pet,
            get_pet_thumbnail,
            get_pet_preview,
            set_paused,
            set_command_recording,
            get_autostart_enabled,
            set_autostart_enabled,
            get_runtime_status,
            request_input_monitoring_access,
            test_trigger,
            report_overlay_rendered,
            report_overlay_ready,
            show_overlay,
            list_market_pets,
            install_market_pet,
            install_top_market_pets,
            import_pet_zip,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Con Pet")
        .run(|_app, _event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen {
                has_visible_windows,
                ..
            } = _event
            {
                if !has_visible_windows {
                    show_settings(_app);
                }
            }
        });
}

#[cfg(test)]
mod trigger_command_label_tests {
    use super::{
        config::{CommandKey, TriggerChord, TriggerCommand, TriggerModifier},
        format_trigger_command,
    };

    #[test]
    fn formats_platform_primary_modifier_and_ordered_steps() {
        let copy_then_space = TriggerCommand {
            version: 1,
            steps: vec![
                TriggerChord::new(CommandKey::KeyC, vec![TriggerModifier::Primary]),
                TriggerChord::plain(CommandKey::Space),
            ],
        };

        assert_eq!(
            format_trigger_command(&copy_then_space, true, false),
            "⌘ + C → Space"
        );
        assert_eq!(
            format_trigger_command(&copy_then_space, false, true),
            "Ctrl + C → Space"
        );
    }
}
