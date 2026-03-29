import type { ThemePreference } from './types';

export function applyThemePreference(theme: ThemePreference | undefined): void {
  const root = document.documentElement;

  if (theme === 'light' || theme === 'dark') {
    root.dataset.theme = theme;
    return;
  }

  delete root.dataset.theme;
}

export function getThemeLabel(theme: ThemePreference | undefined): string {
  switch (theme) {
    case 'light':
      return '浅色';
    case 'dark':
      return '深色';
    default:
      return '跟随系统';
  }
}
