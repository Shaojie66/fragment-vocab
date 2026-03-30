import { invoke } from '@tauri-apps/api/core';
import { triggerScheduler } from '../domain/scheduler/triggerScheduler';
import { applyModePreset } from '../shared/config';
import { applyThemePreference } from '../shared/theme';
import type {
  AppConfig,
  DashboardState,
  ExportBundle,
  FeedbackRecord,
  FeedbackType,
  TeamTemplate,
} from '../shared/types';
import { initializeDashboard, renderDashboard, renderSchedulerControls } from './dashboard';
import { mainElements } from './elements';
import { initializeEvents, initScheduler, isSchedulerStarted, startDashboardRefreshPolling, stopScheduler } from './events';
import { copyToClipboard, downloadTextFile, getErrorMessage } from './helpers';
import { initializeOnboarding, syncOnboardingVisibility } from './onboarding';
import {
  initializeSettings,
  populateForm,
  populateOnboarding,
  readConfigFromForm,
  readOnboardingConfig,
  syncLaunchAtLogin,
} from './settings';
import { mainState } from './state';
import { initializeTabs } from './tabs';
import { checkForUpdate, dismissVersion, openReleaseUrl, type UpdateInfo } from '../shared/update-checker';
import { initializeWordbooks, loadWordbooks } from './wordbooks';

console.log('Fragment Vocab console booting...');

let saveHintTimer: number | null = null;

function setSaveHint(message: string) {
  mainElements.saveHint.textContent = message;

  if (saveHintTimer !== null) {
    window.clearTimeout(saveHintTimer);
  }

  saveHintTimer = window.setTimeout(() => {
    mainElements.saveHint.textContent = '默认使用系统推荐。你可以直接覆盖，之后也能一键恢复。';
  }, 2400);
}

async function refreshDashboard() {
  try {
    mainState.currentDashboard = await invoke<DashboardState>('get_dashboard_state');
    mainState.currentConfig = mainState.currentDashboard.app_config ?? mainState.currentConfig;
    applyThemePreference(mainState.currentConfig.system.theme);
    triggerScheduler.updateConfig(mainState.currentConfig);
    triggerScheduler.syncPauseUntil(mainState.currentDashboard.pause_until);
    populateForm(mainState.currentConfig);
    populateOnboarding(mainState.currentConfig);
    syncOnboardingVisibility(mainState.currentDashboard.needs_onboarding);
    mainState.lastErrorMessage = null;
  } catch (error) {
    mainState.lastErrorMessage = getErrorMessage(error);
  }

  renderDashboard();
}

async function loadTeamTemplates() {
  mainState.currentTemplates = await invoke<TeamTemplate[]>('list_team_templates');
  mainElements.teamTemplateSelect.innerHTML = mainState.currentTemplates
    .map((template) => `<option value="${template.id}">${template.name}</option>`)
    .join('');
  renderDashboard();
}

async function generateExportBundle() {
  mainState.currentExportBundle = await invoke<ExportBundle>('get_export_bundle');
  renderDashboard();
}

async function importConfigFromJson(raw: string) {
  const importedConfig = JSON.parse(raw) as AppConfig;
  const requestedLaunchAtLogin = importedConfig.system?.launch_at_login ?? false;
  const launchAtLogin = await syncLaunchAtLogin(requestedLaunchAtLogin);
  importedConfig.system = {
    ...mainState.currentConfig.system,
    ...importedConfig.system,
    launch_at_login: launchAtLogin,
  };

  mainState.currentConfig = await invoke<AppConfig>('update_app_config', { config: importedConfig });
  mainState.currentExportBundle = null;
  await refreshDashboard();
  setSaveHint('配置已导入，你可以继续在主页面微调。');
}

async function submitFeedback(
  feedbackType: FeedbackType,
  source: 'console' | 'card',
  extra: Partial<Pick<FeedbackRecord, 'card_id' | 'word'>> = {},
) {
  try {
    mainState.currentDashboard = mainState.currentDashboard
      ? {
          ...mainState.currentDashboard,
          recent_feedback: await invoke<FeedbackRecord[]>('record_feedback', {
            feedbackType,
            source,
            cardId: extra.card_id,
            word: extra.word,
          }),
        }
      : mainState.currentDashboard;
    await refreshDashboard();
    setSaveHint('反馈已记录，后续推荐会参考这条信号。');
  } catch (error) {
    mainState.lastErrorMessage = getErrorMessage(error);
    renderDashboard();
    setSaveHint('记录反馈失败，请稍后再试。');
  }
}

async function applySelectedTemplate() {
  try {
    const selected = mainState.currentTemplates.find((template) => template.id === mainElements.teamTemplateSelect.value);

    if (!selected) {
      setSaveHint('当前没有可应用的模板。');
      return;
    }

    const requestedLaunchAtLogin = selected.config.system.launch_at_login;
    const launchAtLogin = await syncLaunchAtLogin(requestedLaunchAtLogin);
    const nextConfig = JSON.parse(JSON.stringify(selected.config)) as AppConfig;
    nextConfig.system.launch_at_login = launchAtLogin;
    mainState.currentConfig = await invoke<AppConfig>('update_app_config', { config: nextConfig });
    mainState.currentExportBundle = null;
    await refreshDashboard();
    setSaveHint(`${selected.name}模板已应用，你可以继续在主页面微调。`);
  } catch (error) {
    mainState.lastErrorMessage = getErrorMessage(error);
    renderDashboard();
    setSaveHint('应用模板失败，请稍后重试。');
  }
}

async function saveConfig(): Promise<boolean> {
  try {
    const previousConfig = JSON.parse(JSON.stringify(mainState.currentConfig)) as AppConfig;
    const nextConfig = readConfigFromForm();

    const requestedLaunchAtLogin = nextConfig.system.launch_at_login;
    const launchAtLogin = await syncLaunchAtLogin(requestedLaunchAtLogin);
    nextConfig.system.launch_at_login = launchAtLogin;

    mainState.currentConfig = await invoke<AppConfig>('update_app_config', { config: nextConfig });
    applyThemePreference(mainState.currentConfig.system.theme);

    mainState.currentExportBundle = null;
    triggerScheduler.updateConfig(mainState.currentConfig);
    populateForm(mainState.currentConfig);
    renderDashboard();

    const requiresRestartNotice =
      previousConfig.system.start_behavior !== mainState.currentConfig.system.start_behavior
      || previousConfig.system.tray_enabled !== mainState.currentConfig.system.tray_enabled;

    if (launchAtLogin !== requestedLaunchAtLogin) {
      setSaveHint('设置已保存，但开机启动未能按预期更新。');
      return true;
    }

    setSaveHint(
      requiresRestartNotice
        ? '设置已保存。启动行为和托盘开关会在下次启动时完全生效。'
        : '设置已保存，调度器会按新配置继续运行。',
    );
    return true;
  } catch (error) {
    mainState.lastErrorMessage = getErrorMessage(error);
    renderDashboard();
    setSaveHint('保存失败，请检查当前配置后重试。');
    return false;
  }
}

async function restoreRecommended() {
  try {
    mainState.currentConfig = applyModePreset(mainState.currentConfig, 'gentle');
    mainState.currentConfig.reminder.using_recommended = true;
    mainState.currentConfig.reminder.mode = 'gentle';
    mainState.currentConfig.schedule.weekday_profile = 'gentle';
    mainState.currentConfig.schedule.weekend_profile = 'balanced';
    mainState.currentConfig = await invoke<AppConfig>('update_app_config', { config: mainState.currentConfig });
    applyThemePreference(mainState.currentConfig.system.theme);
    mainState.currentExportBundle = null;
    triggerScheduler.updateConfig(mainState.currentConfig);
    populateForm(mainState.currentConfig);
    renderDashboard();
    setSaveHint('已恢复为默认推荐值。');
  } catch (error) {
    mainState.lastErrorMessage = getErrorMessage(error);
    renderDashboard();
    setSaveHint('恢复推荐值失败，请稍后再试。');
  }
}

async function completeOnboarding() {
  mainElements.completeOnboardingBtn.disabled = true;

  try {
    const nextConfig = readOnboardingConfig();
    const requestedLaunchAtLogin = nextConfig.system.launch_at_login;
    const launchAtLogin = await syncLaunchAtLogin(requestedLaunchAtLogin);
    nextConfig.system.launch_at_login = launchAtLogin;

    mainState.currentConfig = await invoke<AppConfig>('complete_onboarding', { config: nextConfig });
    mainState.currentExportBundle = null;
    syncOnboardingVisibility(false);
    await refreshDashboard();

    if (launchAtLogin !== requestedLaunchAtLogin) {
      setSaveHint('首次设置已完成，但开机启动未能按预期更新。');
      return;
    }

    setSaveHint('首次设置已完成，系统会按你的偏好开始提醒。');
  } catch (error) {
    mainState.lastErrorMessage = getErrorMessage(error);
    renderDashboard();
    setSaveHint('首次设置保存失败，请再试一次。');
  } finally {
    mainElements.completeOnboardingBtn.disabled = false;
  }
}

async function pauseScheduler(minutes: number) {
  try {
    await invoke('pause_scheduler', { minutes });
    triggerScheduler.pause(minutes);
    await refreshDashboard();
  } catch (error) {
    mainState.lastErrorMessage = getErrorMessage(error);
    renderDashboard();
    setSaveHint('暂停失败，请稍后再试。');
  }
}

async function pauseUntilEndOfDay() {
  const now = new Date();
  const endOfDay = new Date(now);
  endOfDay.setHours(23, 59, 59, 999);
  const minutes = Math.max(1, Math.ceil((endOfDay.getTime() - now.getTime()) / (60 * 1000)));
  await pauseScheduler(minutes);
}

async function resumeScheduler() {
  try {
    await invoke('resume_scheduler');
    triggerScheduler.resume();
    await refreshDashboard();
  } catch (error) {
    mainState.lastErrorMessage = getErrorMessage(error);
    renderDashboard();
    setSaveHint('恢复失败，请稍后再试。');
  }
}

async function openStatsWindow() {
  await invoke('show_stats_window');
}

async function handleGenerateExport() {
  try {
    await generateExportBundle();
    setSaveHint('已生成当前配置摘要，可直接复制给团队讨论。');
  } catch (error) {
    mainState.lastErrorMessage = getErrorMessage(error);
    renderDashboard();
    setSaveHint('生成导出摘要失败，请稍后重试。');
  }
}

async function handleCopyExportSummary() {
  if (!mainState.currentExportBundle) {
    await generateExportBundle();
  }

  const copied = await copyToClipboard(mainState.currentExportBundle?.summary_text ?? '');
  setSaveHint(copied ? '配置摘要已复制。' : '复制失败，请检查系统剪贴板权限。');
}

async function handleCopyExportJson() {
  if (!mainState.currentExportBundle) {
    await generateExportBundle();
  }

  const copied = await copyToClipboard(mainState.currentExportBundle?.config_json ?? '');
  setSaveHint(copied ? '配置 JSON 已复制。' : '复制失败，请检查系统剪贴板权限。');
}

async function handleImportConfigFile(file: File) {
  try {
    const raw = await file.text();
    await importConfigFromJson(raw);
  } catch (error) {
    mainState.lastErrorMessage = getErrorMessage(error);
    renderDashboard();
    setSaveHint('导入失败，请确认 JSON 来自有效的配置导出。');
  }
}

async function handleDownloadExportJson() {
  if (!mainState.currentExportBundle) {
    await generateExportBundle();
  }

  if (mainState.currentExportBundle) {
    downloadTextFile(
      mainState.currentExportBundle.config_json,
      `${mainState.currentExportBundle.file_name_hint}.json`,
      'application/json',
    );
    setSaveHint('配置 JSON 已下载。');
  }
}

async function handleStartScheduler() {
  const saved = await saveConfig();

  if (!saved) {
    return;
  }

  if (isSchedulerStarted()) {
    stopScheduler();
  }

  triggerScheduler.updateConfig(mainState.currentConfig);
  await initScheduler();
  renderDashboard();
}

function handleStopScheduler() {
  stopScheduler();
  renderDashboard();
}

function initializeModules() {
  initializeOnboarding({
    onComplete: completeOnboarding,
  });

  initializeSettings({
    renderDashboard,
    setSaveHint,
    onRefreshDashboard: refreshDashboard,
    onSaveConfig: async () => {
      await saveConfig();
    },
    onRestoreRecommended: restoreRecommended,
  });

  initializeDashboard({
    onApplyTemplate: applySelectedTemplate,
    onCopyExportJson: handleCopyExportJson,
    onCopyExportSummary: handleCopyExportSummary,
    onDownloadExportJson: handleDownloadExportJson,
    onFeedbackTooFew: async () => {
      await submitFeedback('too_few_reminders', 'console');
    },
    onFeedbackTooMany: async () => {
      await submitFeedback('too_many_reminders', 'console');
    },
    onGenerateExport: handleGenerateExport,
    onImportConfigFile: handleImportConfigFile,
    onOpenStats: openStatsWindow,
    onPauseOneHour: async () => {
      await pauseScheduler(60);
    },
    onPauseToday: pauseUntilEndOfDay,
    onResume: resumeScheduler,
    onStartScheduler: handleStartScheduler,
    onStopScheduler: handleStopScheduler,
  });

  initializeWordbooks({
    refreshDashboard,
    renderDashboard,
    setSaveHint,
  });

  initializeTabs();
  initializeEvents({
    onSchedulerStateChange: renderSchedulerControls,
    refreshDashboard,
    renderDashboard,
  });
}

function renderUpdateBanner(info: UpdateInfo) {
  const existing = document.getElementById('updateBanner');
  if (existing) {
    existing.remove();
  }

  if (!info.available || !info.version || !info.url) {
    return;
  }

  const banner = document.createElement('section');
  banner.id = 'updateBanner';
  banner.className = 'update-banner';
  banner.innerHTML = `
    <div class="update-banner-copy">
      <strong>有新版本可用：${info.name ?? `v${info.version}`}</strong>
      <span>当前版本 v0.1.0</span>
    </div>
    <div class="update-banner-actions">
      <button class="primary-btn" id="updateDownloadBtn" type="button">前往下载</button>
      <button class="ghost-btn" id="updateDismissBtn" type="button">忽略此版本</button>
    </div>
  `;

  const topline = document.querySelector('.topline');
  if (topline) {
    topline.insertAdjacentElement('afterend', banner);
  } else {
    document.querySelector('.app-shell')?.prepend(banner);
  }

  document.getElementById('updateDownloadBtn')?.addEventListener('click', () => {
    openReleaseUrl(info.url!);
  });

  document.getElementById('updateDismissBtn')?.addEventListener('click', () => {
    dismissVersion(info.version!);
    banner.remove();
  });
}

async function bootstrap() {
  initializeModules();
  renderSchedulerControls(false);
  await refreshDashboard();

  try {
    await loadTeamTemplates();
    await loadWordbooks();
  } catch (error) {
    mainState.lastErrorMessage = getErrorMessage(error);
  }

  mainState.currentConfig.system.launch_at_login = await syncLaunchAtLogin(mainState.currentConfig.system.launch_at_login);
  populateForm(mainState.currentConfig);
  populateOnboarding(mainState.currentConfig);
  renderDashboard();
  startDashboardRefreshPolling();

  // Check for updates (non-blocking)
  checkForUpdate().then(renderUpdateBanner).catch(() => {});
}

window.addEventListener('DOMContentLoaded', () => {
  void bootstrap();
});
