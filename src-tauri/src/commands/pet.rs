use tauri::{Manager, State};

use crate::db::pet_model::PetState;
use crate::db::{Database, PetsRepository};
use crate::pet::PetEngine;

const PET_WINDOW_X: i32 = 20;
const PET_WINDOW_Y: i32 = 30;

/// Get the current pet state
#[tauri::command]
pub fn get_pet_state(db: State<Database>) -> Result<PetState, String> {
    let conn = db.get_connection();
    let pets_repo = PetsRepository::new(conn);

    pets_repo
        .get_or_create()
        .map_err(|e| format!("Failed to get pet state: {}", e))
}

/// Internal function to update pet after a study action
pub fn update_pet_on_study(db: &Database) -> Result<PetState, String> {
    let conn = db.get_connection();
    let pets_repo = PetsRepository::new(conn);

    let mut pet = pets_repo
        .get_or_create()
        .map_err(|e| format!("Failed to get pet: {}", e))?;

    // Process the study action
    PetEngine::process_study_action(&mut pet);

    // Save updated pet
    pets_repo
        .update(&pet)
        .map_err(|e| format!("Failed to update pet: {}", e))?;

    Ok(pet)
}

/// Initialize pet on app startup - process daily health check
pub fn init_pet_on_startup(db: &Database) -> Result<(), String> {
    let conn = db.get_connection();
    let pets_repo = PetsRepository::new(conn);

    let mut pet = pets_repo
        .get_or_create()
        .map_err(|e| format!("Failed to get pet: {}", e))?;

    // Process daily health check (decay health, check streak)
    PetEngine::process_daily_health_check(&mut pet);

    // Save updated pet
    pets_repo
        .update(&pet)
        .map_err(|e| format!("Failed to update pet: {}", e))?;

    Ok(())
}

/// Show the pet window
#[tauri::command]
pub fn show_pet_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("pet") {
        // Position in top-left corner, below menu bar
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: PET_WINDOW_X,
            y: PET_WINDOW_Y,
        }));
        let _ = window.show();
    }
}
