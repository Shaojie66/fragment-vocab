use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, Runtime,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

mod commands;
mod db;
mod idle;

// 编译期内嵌词库
const IELTS_CORE_WORDBOOK: &str = include_str!("../../assets/wordbooks/ielts-core-3000.json");

use db::{migration::Migrator, Database};

fn focus_window(window: &tauri::WebviewWindow) {
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

fn show_main_window_internal(app: &tauri::AppHandle) {
    if let Some(card_window) = app.get_webview_window("card") {
        let was_visible = matches!(card_window.is_visible(), Ok(true));
        let _ = card_window.emit("card-window-hidden", ());
        let _ = card_window.hide();
        if was_visible {
            let _ = app.emit("card-hidden", ());
        }
    }

    if let Some(window) = app.get_webview_window("main") {
        focus_window(&window);
    }
}

fn show_card_window_internal(app: &tauri::AppHandle) {
    if let Some(main_window) = app.get_webview_window("main") {
        let _ = main_window.hide();
    }

    if let Some(window) = app.get_webview_window("card") {
        // 根据鼠标位置定位窗口
        if let Ok(Some(monitor)) = window.current_monitor() {
            let screen_size = monitor.size();
            let window_width = 480;
            let window_height = 460;
            let margin = 1000;
            let margin_bottom = 1000;
            let offset = 20;

            // 定义安全区域边界
            let safe_left = margin;
            let safe_right = (screen_size.width as i32) - margin;
            let safe_top = margin;
            let safe_bottom = (screen_size.height as i32) - margin_bottom;

            // 获取鼠标位置
            let (mouse_x, mouse_y) = window.cursor_position()
                .map(|pos| (pos.x as i32, pos.y as i32))
                .unwrap_or_else(|_| {
                    (screen_size.width as i32 / 2, screen_size.height as i32 / 2)
                });

            // 尝试放在鼠标右下方
            let mut x = mouse_x + offset;
            let mut y = mouse_y + offset;

            // 如果右侧超出安全区域，尝试放在鼠标左侧
            if x + window_width > safe_right {
                x = mouse_x - window_width - offset;
            }

            // 如果下方超出安全区域，尝试放在鼠标上方
            if y + window_height > safe_bottom {
                y = mouse_y - window_height - offset;
            }

            // 确保窗口完全在安全区域内
            x = x.max(safe_left).min(safe_right - window_width);
            y = y.max(safe_top).min(safe_bottom - window_height);

            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
        }

        let _ = window.set_always_on_top(true);
        focus_window(&window);
        let _ = window.emit("card-window-shown", ());
    }
}

fn apply_startup_behavior(app: &tauri::AppHandle, config: &commands::AppConfig) {
    if let Some(window) = app.get_webview_window("main") {
        if config.system.start_behavior == "show-main" || !config.system.tray_enabled {
            focus_window(&window);
        } else {
            let _ = window.hide();
        }
    }
}

fn hide_auxiliary_windows(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("card") {
        let _ = window.hide();
    }

    if let Some(window) = app.get_webview_window("stats") {
        let _ = window.hide();
    }
}

fn sync_startup_windows(app: &tauri::AppHandle) {
    if let Some(db) = app.try_state::<Database>() {
        let state_repo = db::StateRepository::new(db.get_connection());
        if let Ok(config) = commands::load_app_config(&state_repo) {
            hide_auxiliary_windows(app);
            apply_startup_behavior(app, &config);
        }
    }
}

#[tauri::command]
fn show_card_window(app: tauri::AppHandle) {
    show_card_window_internal(&app);
}

#[tauri::command]
fn show_main_window(app: tauri::AppHandle) {
    show_main_window_internal(&app);
}

#[tauri::command]
fn hide_card_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("card") {
        let _ = window.emit("card-window-hidden", ());
        let _ = window.hide();
    }
}

#[tauri::command]
fn show_stats_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("stats") {
        focus_window(&window);
    }
}

#[tauri::command]
fn get_idle_seconds() -> Result<f64, String> {
    idle::get_idle_seconds()
}

fn emit_card_shortcut_if_visible(app: &tauri::AppHandle, event_name: &str) {
    if let Some(window) = app.get_webview_window("card") {
        if matches!(window.is_visible(), Ok(true)) {
            let _ = window.emit(event_name, ());
        }
    }
}

fn setup_database(app: &tauri::App<impl Runtime>) -> Result<Database, Box<dyn std::error::Error>> {
    // 获取应用数据目录
    let app_data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data_dir)?;

    let db_path = app_data_dir.join("fragment-vocab.db");
    println!("📁 Database path: {:?}", db_path);

    // 创建数据库连接
    let db = Database::new(db_path)?;

    // 运行 migrations
    Migrator::run_migrations(&db)?;

    // 导入词库（仅在首次运行时）
    let words_repo = db::WordsRepository::new(db.get_connection());
    let word_count = words_repo.count()?;

    if word_count == 0 {
        println!("📚 Importing embedded wordbook...");
        match db::WordbookImporter::import_from_embedded(&db, IELTS_CORE_WORDBOOK, "ielts-core") {
            Ok(count) => println!("✅ Imported {} words", count),
            Err(e) => eprintln!("⚠️  Failed to import wordbook: {}", e),
        }
    } else {
        println!("✅ Database already contains {} words", word_count);
    }

    Ok(db)
}

fn setup_tray(app: &tauri::App<tauri::Wry>) -> tauri::Result<()> {
    // 创建菜单项
    let stats_label = MenuItem::with_id(app, "stats_label", "今日统计", false, None::<&str>)?;
    let show_main_i = MenuItem::with_id(app, "show_main", "打开主页面", true, None::<&str>)?;
    let show_stats_i = MenuItem::with_id(app, "show_stats", "打开统计页", true, None::<&str>)?;
    let pause_i = MenuItem::with_id(app, "pause", "暂停 1 小时", true, None::<&str>)?;
    let no_more_today_i =
        MenuItem::with_id(app, "no_more_today", "今日不再提醒", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &stats_label,
            &show_main_i,
            &show_stats_i,
            &pause_i,
            &no_more_today_i,
            &quit_i,
        ],
    )?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show_main" => {
                show_main_window_internal(app);
            }
            "show_stats" => {
                if let Some(window) = app.get_webview_window("stats") {
                    focus_window(&window);
                }
            }
            "pause" => {
                println!("暂停 1 小时");
                // 调用暂停命令并发送事件通知前端
                if let Some(db) = app.try_state::<Database>() {
                    let _ = commands::pause_scheduler(db.clone(), 60);
                    // 发送事件到主窗口
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.emit("scheduler-paused", 60);
                    }
                }
            }
            "no_more_today" => {
                println!("今日不再提醒");
                // 计算到今天结束的分钟数
                let now = chrono::Local::now();
                let end_of_day = now.date_naive().and_hms_opt(23, 59, 59).unwrap();
                let minutes_until_end = (end_of_day
                    .and_local_timezone(chrono::Local)
                    .unwrap()
                    .timestamp()
                    - now.timestamp())
                    / 60;

                if let Some(db) = app.try_state::<Database>() {
                    let _ = commands::pause_scheduler(db.clone(), minutes_until_end as i64);
                    // 发送事件到主窗口
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.emit("scheduler-paused", minutes_until_end);
                    }
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
                let app_handle = tray.app_handle();
                show_main_window_internal(&app_handle);

                if let Some(db) = app_handle.try_state::<Database>() {
                    if let Ok(stats) = commands::get_today_stats(db.clone()) {
                        let stats_text = format!(
                            "今日: {}次 | 正确率: {:.0}% | 新词: {} | 待复习: {}",
                            stats.total_reviews,
                            stats.accuracy,
                            stats.new_words_today,
                            stats.due_cards_count
                        );

                        println!("📊 {}", stats_text);
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Regular);
                app.set_dock_visibility(true);
            }

            // 初始化数据库
            match setup_database(app) {
                Ok(db) => {
                    println!("✅ Database initialized successfully");
                    let app_config = {
                        let state_repo = db::StateRepository::new(db.get_connection());
                        match commands::load_app_config(&state_repo) {
                            Ok(config) => config,
                            Err(error) => {
                                eprintln!("❌ Failed to load app config: {}", error);
                                return Err(error.into());
                            }
                        }
                    };
                    // 将数据库实例存储到 app state 中供后续使用
                    app.manage(db);
                    if app_config.system.tray_enabled {
                        setup_tray(app)?;
                    }
                    let app_handle = app.handle().clone();
                    hide_auxiliary_windows(&app_handle);
                    apply_startup_behavior(&app_handle, &app_config);

                    let delayed_handle = app_handle.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(450));
                        sync_startup_windows(&delayed_handle);
                    });
                }
                Err(e) => {
                    eprintln!("❌ Failed to initialize database: {}", e);
                    return Err(e.into());
                }
            }

            // 注册全局快捷键（处理注册失败的情况）
            let app_handle = app.handle().clone();
            let _ = app.global_shortcut().on_shortcut(
                "CmdOrCtrl+Shift+1",
                move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        emit_card_shortcut_if_visible(&app_handle, "shortcut-option-1");
                    }
                },
            );

            let app_handle = app.handle().clone();
            let _ = app.global_shortcut().on_shortcut(
                "CmdOrCtrl+Shift+2",
                move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        emit_card_shortcut_if_visible(&app_handle, "shortcut-option-2");
                    }
                },
            );

            let app_handle = app.handle().clone();
            let _ = app.global_shortcut().on_shortcut(
                "CmdOrCtrl+Shift+3",
                move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        emit_card_shortcut_if_visible(&app_handle, "shortcut-option-3");
                    }
                },
            );

            let app_handle = app.handle().clone();
            let _ = app.global_shortcut().on_shortcut(
                "CmdOrCtrl+Shift+4",
                move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        emit_card_shortcut_if_visible(&app_handle, "shortcut-option-4");
                    }
                },
            );

            let app_handle = app.handle().clone();
            let _ = app.global_shortcut().on_shortcut(
                "CmdOrCtrl+Shift+Escape",
                move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        emit_card_shortcut_if_visible(&app_handle, "shortcut-skip");
                    }
                },
            );

            // 注册快捷键（忽略失败）
            let _ = app.global_shortcut().register("CmdOrCtrl+Shift+1");
            let _ = app.global_shortcut().register("CmdOrCtrl+Shift+2");
            let _ = app.global_shortcut().register("CmdOrCtrl+Shift+3");
            let _ = app.global_shortcut().register("CmdOrCtrl+Shift+4");
            let _ = app.global_shortcut().register("CmdOrCtrl+Shift+Escape");

            println!("✅ Global shortcuts registered (Cmd+Shift+1/2/3/4/Esc)");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            show_main_window,
            show_card_window,
            hide_card_window,
            show_stats_window,
            get_idle_seconds,
            commands::get_app_config,
            commands::update_app_config,
            commands::complete_onboarding,
            commands::get_dashboard_state,
            commands::list_team_templates,
            commands::record_feedback,
            commands::get_export_bundle,
            commands::import_custom_wordbook,
            commands::list_wordbooks,
            commands::list_wordbook_words,
            commands::set_wordbook_enabled,
            commands::delete_wordbook,
            commands::get_next_card,
            commands::submit_review,
            commands::get_today_stats,
            commands::pause_scheduler,
            commands::resume_scheduler,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app, event| {
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen {
            has_visible_windows,
            ..
        } = event
        {
            if !has_visible_windows {
                show_main_window_internal(app);
            }
        }
    });
}
