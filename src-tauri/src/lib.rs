use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};

#[tauri::command]
fn show_card_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("card") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn hide_card_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("card") {
        let _ = window.hide();
    }
}

#[tauri::command]
fn show_stats_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("stats") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn setup_tray<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let show_stats_i = MenuItem::with_id(app, "show_stats", "统计", true, None::<&str>)?;
    let pause_i = MenuItem::with_id(app, "pause", "暂停 1 小时", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    
    let menu = Menu::with_items(app, &[&show_stats_i, &pause_i, &quit_i])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show_stats" => {
                if let Some(window) = app.get_webview_window("stats") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "pause" => {
                println!("暂停 1 小时");
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                println!("托盘图标被点击");
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            setup_tray(app)?;
            
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            show_card_window,
            hide_card_window,
            show_stats_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
