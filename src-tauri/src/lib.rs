use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime, Emitter,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

mod db;
mod idle;
mod commands;

// 编译期内嵌词库
const IELTS_CORE_WORDBOOK: &str = include_str!("../../assets/wordbooks/ielts-core-3000.json");

use db::{Database, migration::Migrator};

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

#[tauri::command]
fn get_idle_seconds() -> Result<f64, String> {
    idle::get_idle_seconds()
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

fn setup_tray<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    // 创建菜单项
    let stats_label = MenuItem::with_id(app, "stats_label", "今日统计", false, None::<&str>)?;
    let show_stats_i = MenuItem::with_id(app, "show_stats", "打开统计页", true, None::<&str>)?;
    let pause_i = MenuItem::with_id(app, "pause", "暂停 1 小时", true, None::<&str>)?;
    let no_more_today_i = MenuItem::with_id(app, "no_more_today", "今日不再提醒", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    
    let menu = Menu::with_items(
        app,
        &[&stats_label, &show_stats_i, &pause_i, &no_more_today_i, &quit_i]
    )?;

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
                // 调用暂停命令
                if let Some(db) = app.try_state::<Database>() {
                    let _ = commands::pause_scheduler(db.clone(), 60);
                }
            }
            "no_more_today" => {
                println!("今日不再提醒");
                // 计算到今天结束的分钟数
                let now = chrono::Local::now();
                let end_of_day = now.date_naive().and_hms_opt(23, 59, 59).unwrap();
                let minutes_until_end = (end_of_day.and_local_timezone(chrono::Local).unwrap().timestamp() - now.timestamp()) / 60;
                
                if let Some(db) = app.try_state::<Database>() {
                    let _ = commands::pause_scheduler(db.clone(), minutes_until_end as i64);
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
                // 更新菜单统计信息
                let app_handle = tray.app_handle();
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
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // 初始化数据库
            match setup_database(app) {
                Ok(db) => {
                    println!("✅ Database initialized successfully");
                    // 将数据库实例存储到 app state 中供后续使用
                    app.manage(db);
                }
                Err(e) => {
                    eprintln!("❌ Failed to initialize database: {}", e);
                    return Err(e.into());
                }
            }
            
            setup_tray(app)?;
            
            // 注册全局快捷键（处理注册失败的情况）
            let app_handle = app.handle().clone();
            let _ = app.global_shortcut().on_shortcut("CmdOrCtrl+Shift+K", move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    if let Some(window) = app_handle.get_webview_window("card") {
                        let _ = window.emit("shortcut-know", ());
                    }
                }
            });
            
            let app_handle = app.handle().clone();
            let _ = app.global_shortcut().on_shortcut("CmdOrCtrl+Shift+J", move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    if let Some(window) = app_handle.get_webview_window("card") {
                        let _ = window.emit("shortcut-dont-know", ());
                    }
                }
            });
            
            let app_handle = app.handle().clone();
            let _ = app.global_shortcut().on_shortcut("CmdOrCtrl+Shift+Escape", move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    if let Some(window) = app_handle.get_webview_window("card") {
                        let _ = window.emit("shortcut-skip", ());
                    }
                }
            });
            
            // 注册快捷键（忽略失败）
            let _ = app.global_shortcut().register("CmdOrCtrl+Shift+K");
            let _ = app.global_shortcut().register("CmdOrCtrl+Shift+J");
            let _ = app.global_shortcut().register("CmdOrCtrl+Shift+Escape");
            
            println!("✅ Global shortcuts registered (Cmd+Shift+K/J/Esc)");
            
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            show_card_window,
            hide_card_window,
            show_stats_window,
            get_idle_seconds,
            commands::get_next_card,
            commands::submit_review,
            commands::get_today_stats,
            commands::pause_scheduler,
            commands::resume_scheduler,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
