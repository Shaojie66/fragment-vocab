use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};
use std::sync::{Arc, Mutex};

use crate::db::pet_model::PetState;

pub struct PetsRepository {
    conn: Arc<Mutex<Connection>>,
}

impl PetsRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Get the pet state (singleton, always id=1)
    pub fn get(&self) -> Result<Option<PetState>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, stage, health, experience, current_streak, vitality_multiplier,
                    last_study_at, last_review_at, created_at, updated_at
             FROM pets WHERE id = 1",
        )?;

        let pet = stmt
            .query_row([], |row| {
                Ok(PetState {
                    id: row.get(0)?,
                    stage: row.get::<_, i64>(1)? as u8,
                    health: row.get(2)?,
                    experience: row.get::<_, i64>(3)? as u32,
                    current_streak: row.get::<_, i64>(4)? as u32,
                    vitality_multiplier: row.get(5)?,
                    last_study_at: row.get(6)?,
                    last_review_at: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })
            .optional()?;

        Ok(pet)
    }

    /// Create a new pet with default state
    pub fn create(&self) -> Result<PetState> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().to_rfc3339();

        conn.execute(
            "INSERT INTO pets (id, stage, health, experience, current_streak, vitality_multiplier, created_at, updated_at)
             VALUES (1, 0, 1.0, 0, 0, 1.0, ?1, ?1)",
            [&now],
        )
        .context("Failed to create pet")?;

        Ok(PetState::default())
    }

    /// Update the pet state
    pub fn update(&self, pet: &PetState) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().to_rfc3339();

        conn.execute(
            "UPDATE pets SET
                stage = ?1,
                health = ?2,
                experience = ?3,
                current_streak = ?4,
                vitality_multiplier = ?5,
                last_study_at = ?6,
                last_review_at = ?7,
                updated_at = ?8
             WHERE id = 1",
            (
                pet.stage as i64,
                pet.health,
                pet.experience as i64,
                pet.current_streak as i64,
                pet.vitality_multiplier,
                &pet.last_study_at,
                &pet.last_review_at,
                &now,
            ),
        )
        .context("Failed to update pet")?;

        Ok(())
    }

    /// Get or create the pet
    pub fn get_or_create(&self) -> Result<PetState> {
        match self.get() {
            Ok(Some(pet)) => Ok(pet),
            Ok(None) => self.create(),
            Err(e) => Err(e),
        }
    }
}
