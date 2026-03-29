import { mainElements } from './elements';

function applyActiveTab(nextTabId: string) {
  mainElements.tabButtons.forEach((button) => {
    const isActive = button.dataset.mainTab === nextTabId;
    button.classList.toggle('active', isActive);
    button.setAttribute('aria-selected', String(isActive));
  });

  mainElements.tabPanels.forEach((panel) => {
    const isActive = panel.dataset.mainPanel === nextTabId;
    panel.classList.toggle('hidden', !isActive);
  });
}

export function initializeTabs() {
  if (!mainElements.tabButtons.length || !mainElements.tabPanels.length) {
    return;
  }

  const initialTabId = mainElements.tabButtons.find((button) => button.getAttribute('aria-selected') === 'true')?.dataset.mainTab
    ?? mainElements.tabButtons[0]?.dataset.mainTab;

  if (!initialTabId) {
    return;
  }

  applyActiveTab(initialTabId);
  mainElements.tabButtons.forEach((button) => {
    button.addEventListener('click', () => {
      const nextTabId = button.dataset.mainTab;

      if (!nextTabId) {
        return;
      }

      applyActiveTab(nextTabId);
    });
  });
}
