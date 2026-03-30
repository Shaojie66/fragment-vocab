import { mainElements } from './elements';

interface OnboardingDependencies {
  onComplete: () => Promise<void>;
}

const LAST_STEP_INDEX = 3;
let currentStepIndex = 0;

function clampStep(stepIndex: number): number {
  return Math.min(Math.max(stepIndex, 0), LAST_STEP_INDEX);
}

function renderOnboardingStep() {
  const isFirstStep = currentStepIndex === 0;
  const isLastStep = currentStepIndex === LAST_STEP_INDEX;

  mainElements.onboardingWizard.dataset.step = String(currentStepIndex + 1);

  mainElements.onboardingStepPanels.forEach((panel, index) => {
    const isActive = index === currentStepIndex;
    panel.classList.toggle('is-active', isActive);
    panel.setAttribute('aria-hidden', String(!isActive));
  });

  mainElements.onboardingStepDots.forEach((dot, index) => {
    const isActive = index === currentStepIndex;
    dot.classList.toggle('is-active', isActive);
    dot.setAttribute('aria-current', isActive ? 'step' : 'false');
  });

  mainElements.onboardingPrevBtn.disabled = isFirstStep;
  mainElements.onboardingNextBtn.classList.toggle('hidden', isLastStep);
  mainElements.completeOnboardingBtn.classList.toggle('hidden', !isLastStep);
}

function setStep(stepIndex: number) {
  currentStepIndex = clampStep(stepIndex);
  renderOnboardingStep();
}

export function syncOnboardingVisibility(visible: boolean) {
  const wasVisible = !mainElements.onboardingWizard.classList.contains('hidden');
  mainElements.onboardingWizard.classList.toggle('hidden', !visible);
  mainElements.onboardingWizard.setAttribute('aria-hidden', String(!visible));
  document.body.classList.toggle('modal-open', visible);

  if (visible && !wasVisible) {
    setStep(0);
  }
}

export function initializeOnboarding(dependencies: OnboardingDependencies) {
  mainElements.onboardingPrevBtn.addEventListener('click', () => {
    setStep(currentStepIndex - 1);
  });

  mainElements.onboardingNextBtn.addEventListener('click', () => {
    setStep(currentStepIndex + 1);
  });

  mainElements.onboardingStepDots.forEach((dot, index) => {
    dot.addEventListener('click', () => {
      setStep(index);
    });
  });

  mainElements.completeOnboardingBtn.addEventListener('click', () => {
    void dependencies.onComplete();
  });

  renderOnboardingStep();
}
