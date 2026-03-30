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

/// Internal function to update pet after a review action.
/// Both reviews and study actions update the pet (experience, streak, timestamps).
/// Health decay is handled by init_pet_on_startup.
pub fn update_pet_after_review(db: &Database) -> Result<PetState, String> {
    let conn = db.get_connection();
    let pets_repo = PetsRepository::new(conn);

    let mut pet = pets_repo
        .get_or_create()
        .map_err(|e| format!("Failed to get pet: {}", e))?;

    // Process study effects (experience, streak, vitality, timestamps)
    PetEngine::process_study_action(&mut pet);

    // Update last_review_at timestamp
    pet.last_review_at = Some(chrono::Local::now().to_rfc3339());

    pets_repo
        .update(&pet)
        .map_err(|e| format!("Failed to update pet: {}", e))?;

    Ok(pet)
}

/// Internal function to update pet after a study action.
/// Applies study effects (experience, streak, timestamps) without re-running
/// health decay — health decay is handled by init_pet_on_startup on app launch.
pub fn update_pet_on_study(db: &Database) -> Result<PetState, String> {
    let conn = db.get_connection();
    let pets_repo = PetsRepository::new(conn);

    let mut pet = pets_repo
        .get_or_create()
        .map_err(|e| format!("Failed to get pet: {}", e))?;

    // Process the study action (streak, vitality, experience, timestamps).
    // Health decay is handled by init_pet_on_startup so it's only applied once per session.
    PetEngine::process_study_action(&mut pet);

    // Also update last_review_at since studying a new word counts as a review
    pet.last_review_at = Some(chrono::Local::now().to_rfc3339());

    // Save updated pet
    pets_repo
        .update(&pet)
        .map_err(|e| format!("Failed to update pet: {}", e))?;

    Ok(pet)
}

/// Initialize pet on app startup — apply daily health decay and save.
/// Health decay should only run once per app session (here), not again on study actions.
pub fn init_pet_on_startup(db: &Database) -> Result<(), String> {
    let conn = db.get_connection();
    let pets_repo = PetsRepository::new(conn);

    let mut pet = pets_repo
        .get_or_create()
        .map_err(|e| format!("Failed to get pet: {}", e))?;

    // Apply daily health decay and streak check
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
