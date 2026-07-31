//! macOS menu-bar tray widget with live CPU / RAM / watts.

use std::sync::Arc;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};

use crate::commands::AppState;

pub fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Afficher", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Masquer", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

    let Some(icon) = app.default_window_icon().cloned() else {
        eprintln!("tray: no default window icon");
        return Ok(());
    };

    let _tray = TrayIconBuilder::with_id("main")
        .icon(icon)
        .menu(&menu)
        .tooltip("Monitoring Systeme")
        .title("…")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "hide" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    if w.is_visible().unwrap_or(false) {
                        let _ = w.hide();
                    } else {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

pub fn update_tray_title(app: &AppHandle, state: &AppState) {
    let tray_cfg = state.settings.get().tray_display;

    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_visible(tray_cfg.enabled);
    }

    // Without a menu-bar icon, keep the main window available so the user
    // is not stranded with neither tray nor UI.
    if !tray_cfg.enabled {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.show();
        }
        return;
    }

    let cpu = (*state.provider_state.last_cpu.lock()).unwrap_or(0.0);
    let ram = (*state.provider_state.last_ram.lock()).unwrap_or(0.0);
    let watts = state
        .provider_state
        .last_energy
        .lock()
        .as_ref()
        .map(|e| e.watts)
        .unwrap_or(0.0);

    let mut parts: Vec<String> = Vec::new();
    let mut tip_parts: Vec<String> = Vec::new();
    if tray_cfg.cpu {
        parts.push(format!("{:.0}%", cpu));
        tip_parts.push(format!("CPU {cpu:.0}%"));
    }
    if tray_cfg.memory {
        parts.push(format!("{:.0}%", ram));
        tip_parts.push(format!("RAM {ram:.0}%"));
    }
    if tray_cfg.energy {
        parts.push(format!("{:.0}W", watts));
        tip_parts.push(format!("{watts:.1} W"));
    }

    let title = if parts.is_empty() {
        String::new()
    } else {
        parts.join(" · ")
    };
    let tip = if tip_parts.is_empty() {
        "Monitoring Systeme".into()
    } else {
        tip_parts.join(" · ")
    };

    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_title(Some(title.as_str()));
        let _ = tray.set_tooltip(Some(tip.as_str()));
    }
}

/// Spawn a lightweight background sampler for tray + 24 h history when UI is hidden.
pub fn spawn_background_sampler(app: AppHandle, state: Arc<AppState>) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(15));
            let provider = Arc::clone(&state.provider);
            let _ = provider.energy();
            if let Ok(cpu) = provider.cpu() {
                *state.provider_state.last_cpu.lock() = Some(cpu.usage);
            }
            if let Ok(mem) = provider.memory() {
                *state.provider_state.last_ram.lock() = Some(mem.percent);
            }
            update_tray_title(&app, &state);
            state.provider_state.energy_history.lock().save_to_disk();
        }
    });
}
