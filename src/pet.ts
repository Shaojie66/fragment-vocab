import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface PetState {
  id: number;
  stage: number;
  health: number;
  experience: number;
  current_streak: number;
  vitality_multiplier: number;
  last_study_at: string | null;
  last_review_at: string | null;
  created_at: string;
  updated_at: string;
}

// Visual state mapping from health
function getVisualState(health: number): { saturate: number; opacity: number } {
  if (health >= 1.0) return { saturate: 1, opacity: 1 };
  if (health >= 0.75) return { saturate: 0.8, opacity: 0.9 };
  if (health >= 0.5) return { saturate: 0.6, opacity: 0.7 };
  if (health >= 0.25) return { saturate: 0.4, opacity: 0.5 };
  return { saturate: 0.2, opacity: 0.3 };
}

// Update the pet visual based on state
function updatePetVisual(pet: PetState) {
  const slime = document.getElementById('slime');
  if (!slime) return;

  // Update stage class
  slime.className = `slime stage-${pet.stage}`;

  // Update visual state (saturation and opacity)
  const visual = getVisualState(pet.health);
  document.documentElement.style.setProperty('--saturate', String(visual.saturate));
  document.documentElement.style.setProperty('--opacity', String(visual.opacity));

  // Update hunger state
  if (pet.health < 0.25) {
    slime.classList.add('very-hungry');
    slime.classList.remove('hungry');
  } else if (pet.health < 0.5) {
    slime.classList.add('hungry');
    slime.classList.remove('very-hungry');
  } else {
    slime.classList.remove('hungry', 'very-hungry');
  }

  // Add evolved class for stages 3+
  if (pet.stage >= 3) {
    slime.classList.add('evolved');
  }

  console.log(`Pet updated: stage=${pet.stage}, health=${pet.health.toFixed(2)}, exp=${pet.experience}`);
}

// Trigger evolution animation
function triggerEvolution() {
  const slime = document.getElementById('slime');
  if (!slime) return;

  slime.classList.add('evolving');
  setTimeout(() => {
    slime.classList.remove('evolving');
  }, 1000);
}

// Trigger celebration animation
function triggerCelebration() {
  const slime = document.getElementById('slime');
  if (!slime) return;

  slime.classList.add('celebrating');
  setTimeout(() => {
    slime.classList.remove('celebrating');
  }, 1500);
}

// Load and display pet state
async function loadPetState() {
  try {
    const pet = await invoke<PetState>('get_pet_state');
    updatePetVisual(pet);
  } catch (error) {
    console.error('Failed to load pet state:', error);
  }
}

// Handle pet state update events from backend
async function setupEventListeners() {
  // Listen for pet state updates
  await listen('pet-state-updated', (event) => {
    const pet = event.payload as PetState;
    const slime = document.getElementById('slime');
    const currentStage = slime?.className.match(/stage-(\d)/)?.[1];

    // Check if stage changed (evolution)
    if (currentStage && parseInt(currentStage) < pet.stage) {
      triggerEvolution();
    }

    updatePetVisual(pet);
  });

  // Listen for study completed events
  await listen('study-completed', () => {
    // Small celebration on each study
    triggerCelebration();
  });
}

// Make window draggable
function setupWindowDrag() {
  // Window dragging is handled by Tauri's decorations: false
  // and -webkit-app-region: drag in CSS
}

// Initialize
async function init() {
  console.log('Pixel Pet window initialized');

  // Load initial state
  await loadPetState();

  // Setup event listeners
  await setupEventListeners();

  // Setup window dragging
  setupWindowDrag();
}

// Start the app
init();
