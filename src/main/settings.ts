import { invoke } from '@tauri-apps/api/core';
import { disable as disableAutostart, enable as enableAutostart, isEnabled as isAutostartEnabled } from '@tauri-apps/plugin-autostart';
import { applyModePreset, getModeLabel, isRecommendedReminderMode } from '../shared/config';
import { applyThemePreference } from '../shared/theme';
import type { AppConfig, ImportSummary, RecommendedReminderMode, ReminderMode, ThemePreference } from '../shared/types';
import { mainElements } from './elements';
import { cloneConfig, downloadTextFile, getErrorMessage, readNumberInput } from './helpers';
import { mainState } from './state';

interface SettingsDependencies {
  renderDashboard: () => void;
  setSaveHint: (message: string) => void;
  onRefreshDashboard: () => Promise<void>;
  onSaveConfig: () => Promise<void>;
  onRestoreRecommended: () => Promise<void>;
}

function buildImportSummaryMessage(summary: ImportSummary): string {
  return `学习数据已导入：单词 ${summary.words}，卡片 ${summary.srs_cards}，复习记录 ${summary.review_logs}，状态 ${summary.app_state}，宠物 ${summary.pets}，成就 ${summary.achievements}。`;
}

async function exportLearningData(dependencies: SettingsDependencies): Promise<void> {
  try {
    const json = await invoke<string>('export_all_data');
    const stamp = new Date().toISOString().slice(0, 10);
    downloadTextFile(json, `fragment-vocab-learning-data-${stamp}.json`, 'application/json');
    dependencies.setSaveHint('学习数据已导出到 JSON 文件。');
  } catch (error) {
    console.error('导出学习数据失败:', error);
    dependencies.setSaveHint(`导出学习数据失败：${getErrorMessage(error)}`);
  }
}

async function importLearningData(file: File, dependencies: SettingsDependencies): Promise<void> {
  try {
    const jsonData = await file.text();
    const summary = await invoke<ImportSummary>('import_all_data', { jsonData });
    await dependencies.onRefreshDashboard();
    dependencies.setSaveHint(buildImportSummaryMessage(summary));
  } catch (error) {
    console.error('导入学习数据失败:', error);
    dependencies.setSaveHint(`导入学习数据失败：${getErrorMessage(error)}`);
  }
}

function syncReminderFieldStates(mode: ReminderMode) {
  const isCustomMode = mode === 'custom';
  mainElements.weekdayProfileInput.disabled = isCustomMode;
  mainElements.weekendProfileInput.disabled = isCustomMode;
}

function syncSystemFieldStates() {
  const trayDisabled = !mainElements.trayEnabledInput.checked;
  mainElements.startBehaviorSelect.disabled = trayDisabled;

  if (trayDisabled && mainElements.startBehaviorSelect.value === 'minimize-to-tray') {
    mainElements.startBehaviorSelect.value = 'show-main';
  }
}

export function populateForm(config: AppConfig) {
  mainElements.modeSelect.value = config.reminder.mode;
  mainElements.idleThresholdInput.value = String(config.reminder.idle_threshold_sec);
  mainElements.fallbackEnabledInput.checked = config.reminder.fallback_enabled;
  mainElements.fallbackIntervalInput.value = String(config.reminder.fallback_interval_min);

  mainElements.dailyNewLimitInput.value = String(config.learning.daily_new_limit);
  mainElements.reviewFirstInput.checked = config.learning.review_first;
  mainElements.allowNewWhenNoDueInput.checked = config.learning.allow_new_when_no_due;

  mainElements.quietStartInput.value = config.schedule.quiet_hours_start;
  mainElements.quietEndInput.value = config.schedule.quiet_hours_end;
  mainElements.weekdayProfileInput.value = config.schedule.weekday_profile ?? 'gentle';
  mainElements.weekendProfileInput.value = config.schedule.weekend_profile ?? 'balanced';

  mainElements.autoHideInput.value = String(config.card.auto_hide_sec);
  mainElements.revealOrderSelect.value = config.card.reveal_order;
  mainElements.showPhoneticInput.checked = config.card.show_phonetic;
  mainElements.allowSkipInput.checked = config.card.allow_skip;
  mainElements.shortcutsEnabledInput.checked = config.card.shortcuts_enabled;
  mainElements.animationsEnabledInput.checked = config.card.animations_enabled;
  mainElements.autoPronounceInput.checked = config.card.auto_pronounce;

  mainElements.launchAtLoginInput.checked = config.system.launch_at_login;
  mainElements.startBehaviorSelect.value = config.system.start_behavior;
  mainElements.trayEnabledInput.checked = config.system.tray_enabled;
  mainElements.themeSelect.value = config.system.theme;

  syncReminderFieldStates(config.reminder.mode);
  syncSystemFieldStates();
}

export function populateOnboarding(config: AppConfig) {
  const candidateMode = config.schedule.weekday_profile ?? config.reminder.mode;
  const mode = isRecommendedReminderMode(candidateMode) ? candidateMode : 'gentle';

  mainElements.onboardingFrequencyInputs.forEach((input) => {
    input.checked = input.value === mode;
  });
}

export function readConfigFromForm(): AppConfig {
  const mode = mainElements.modeSelect.value as ReminderMode;
  let config = cloneConfig(mainState.currentConfig);

  const userIdleThreshold = readNumberInput(mainElements.idleThresholdInput, config.reminder.idle_threshold_sec);
  const userFallbackEnabled = mainElements.fallbackEnabledInput.checked;
  const userFallbackInterval = readNumberInput(mainElements.fallbackIntervalInput, config.reminder.fallback_interval_min);

  console.log('📝 readConfigFromForm - mode:', mode);
  console.log('📝 readConfigFromForm - userIdleThreshold:', userIdleThreshold);
  console.log('📝 readConfigFromForm - userFallbackInterval:', userFallbackInterval);

  if (mode === 'custom') {
    config.reminder.mode = 'custom';
    config.reminder.using_recommended = false;
    config.reminder.idle_threshold_sec = userIdleThreshold;
    config.reminder.fallback_enabled = userFallbackEnabled;
    config.reminder.fallback_interval_min = userFallbackInterval;
    console.log('✅ Applied custom mode:', config.reminder);
  } else {
    config = applyModePreset(config, mode);
    console.log('✅ Applied preset mode:', mode, config.reminder);
  }

  config.learning.daily_new_limit = readNumberInput(mainElements.dailyNewLimitInput, config.learning.daily_new_limit);
  config.learning.review_first = mainElements.reviewFirstInput.checked;
  config.learning.allow_new_when_no_due = mainElements.allowNewWhenNoDueInput.checked;

  config.schedule.quiet_hours_start = mainElements.quietStartInput.value || config.schedule.quiet_hours_start;
  config.schedule.quiet_hours_end = mainElements.quietEndInput.value || config.schedule.quiet_hours_end;
  config.schedule.weekday_profile = mainElements.weekdayProfileInput.value as RecommendedReminderMode;
  config.schedule.weekend_profile = mainElements.weekendProfileInput.value as RecommendedReminderMode;

  config.card.auto_hide_sec = readNumberInput(mainElements.autoHideInput, config.card.auto_hide_sec);
  config.card.reveal_order = mainElements.revealOrderSelect.value as AppConfig['card']['reveal_order'];
  config.card.show_phonetic = mainElements.showPhoneticInput.checked;
  config.card.allow_skip = mainElements.allowSkipInput.checked;
  config.card.shortcuts_enabled = mainElements.shortcutsEnabledInput.checked;
  config.card.animations_enabled = mainElements.animationsEnabledInput.checked;
  config.card.auto_pronounce = mainElements.autoPronounceInput.checked;

  config.system.launch_at_login = mainElements.launchAtLoginInput.checked;
  config.system.start_behavior = mainElements.startBehaviorSelect.value as AppConfig['system']['start_behavior'];
  config.system.tray_enabled = mainElements.trayEnabledInput.checked;
  config.system.theme = mainElements.themeSelect.value as ThemePreference;

  if (!config.system.tray_enabled && config.system.start_behavior === 'minimize-to-tray') {
    config.system.start_behavior = 'show-main';
  }

  return config;
}

export function readOnboardingConfig(): AppConfig {
  const selectedMode = mainElements.onboardingFrequencyInputs.find((input) => input.checked)?.value;
  const mode: RecommendedReminderMode = isRecommendedReminderMode(selectedMode) ? selectedMode : 'gentle';
  const config = applyModePreset(cloneConfig(mainState.currentConfig), mode);

  config.schedule.weekday_profile = mode;
  config.schedule.weekend_profile = mode;

  return config;
}

export async function syncLaunchAtLogin(enabled: boolean): Promise<boolean> {
  if (typeof window !== 'undefined' && !(window as Window & { __TAURI__?: unknown }).__TAURI__) {
    console.warn('非 Tauri 环境，跳过自动启动同步');
    return enabled;
  }

  try {
    const current = await isAutostartEnabled();

    if (enabled && !current) {
      await enableAutostart();
    } else if (!enabled && current) {
      await disableAutostart();
    }

    return await isAutostartEnabled();
  } catch (error) {
    console.error('同步开机启动失败:', error);
    return enabled;
  }
}

export function initializeSettings(dependencies: SettingsDependencies) {
  mainElements.modeSelect.addEventListener('change', () => {
    const mode = mainElements.modeSelect.value as ReminderMode;

    if (mode !== 'custom') {
      mainState.currentConfig = applyModePreset(mainState.currentConfig, mode);
      mainState.currentConfig.schedule.weekday_profile ??= mode;
      mainState.currentConfig.schedule.weekend_profile ??= mode;
      populateForm(mainState.currentConfig);
      dependencies.renderDashboard();
      dependencies.setSaveHint(`已切换到${getModeLabel(mode)}模式，保存后正式生效。`);
      return;
    }

    mainState.currentConfig.reminder.idle_threshold_sec = readNumberInput(
      mainElements.idleThresholdInput,
      mainState.currentConfig.reminder.idle_threshold_sec,
    );
    mainState.currentConfig.reminder.fallback_enabled = mainElements.fallbackEnabledInput.checked;
    mainState.currentConfig.reminder.fallback_interval_min = readNumberInput(
      mainElements.fallbackIntervalInput,
      mainState.currentConfig.reminder.fallback_interval_min,
    );
    mainState.currentConfig.reminder.mode = 'custom';
    mainState.currentConfig.reminder.using_recommended = false;
    syncReminderFieldStates('custom');
    dependencies.renderDashboard();
  });

  [
    mainElements.idleThresholdInput,
    mainElements.fallbackEnabledInput,
    mainElements.fallbackIntervalInput,
  ].forEach((input) => {
    input.addEventListener('change', () => {
      if (mainElements.modeSelect.value !== 'custom') {
        mainElements.modeSelect.value = 'custom';
        mainState.currentConfig.reminder.mode = 'custom';
        mainState.currentConfig.reminder.using_recommended = false;
        syncReminderFieldStates('custom');
        dependencies.setSaveHint('已切换到自定义模式，保存后生效。');
      }
    });
  });

  mainElements.trayEnabledInput.addEventListener('change', () => {
    syncSystemFieldStates();

    if (!mainElements.trayEnabledInput.checked) {
      dependencies.setSaveHint('关闭托盘后，启动行为会自动回退为显示主页面。');
    }
  });

  mainElements.themeSelect.addEventListener('change', () => {
    applyThemePreference(mainElements.themeSelect.value as ThemePreference);
    dependencies.setSaveHint('主题预览已切换，保存后会同步到其他窗口。');
  });

  mainElements.exportLearningDataBtn.addEventListener('click', () => {
    void exportLearningData(dependencies);
  });
  mainElements.importLearningDataBtn.addEventListener('click', () => {
    mainElements.importLearningDataFileInput.click();
  });
  mainElements.importLearningDataFileInput.addEventListener('change', () => {
    void (async () => {
      const [file] = Array.from(mainElements.importLearningDataFileInput.files ?? []);

      if (!file) {
        return;
      }

      try {
        await importLearningData(file, dependencies);
      } finally {
        mainElements.importLearningDataFileInput.value = '';
      }
    })();
  });

  mainElements.saveConfigBtn.addEventListener('click', () => {
    void dependencies.onSaveConfig();
  });
  mainElements.restoreRecommendedBtn.addEventListener('click', () => {
    void dependencies.onRestoreRecommended();
  });
}
