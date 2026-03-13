import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { disable as disableAutostart, enable as enableAutostart, isEnabled as isAutostartEnabled } from '@tauri-apps/plugin-autostart';
import { triggerScheduler } from './domain/scheduler/triggerScheduler';
import {
  applyModePreset,
  createDefaultAppConfig,
  getBlockReasonLabel,
  getEffectiveReminderConfig,
  getModeLabel,
  getScheduleSegmentLabel,
} from './shared/config';
import type {
  AppConfig,
  DashboardState,
  ExportBundle,
  FeedbackRecord,
  FeedbackType,
  RecommendedReminderMode,
  RecommendationSummary,
  ReminderMode,
  SchedulerSnapshot,
  TeamTemplate,
  TodayStats,
  WordbookImportSummary,
  WordbookListItem,
} from './shared/types';

console.log('Fragment Vocab console booting...');

const modeSelect = document.querySelector('#modeSelect') as HTMLSelectElement;
const idleThresholdInput = document.querySelector('#idleThresholdInput') as HTMLInputElement;
const fallbackEnabledInput = document.querySelector('#fallbackEnabledInput') as HTMLInputElement;
const fallbackIntervalInput = document.querySelector('#fallbackIntervalInput') as HTMLInputElement;
const dailyNewLimitInput = document.querySelector('#dailyNewLimitInput') as HTMLInputElement;
const reviewFirstInput = document.querySelector('#reviewFirstInput') as HTMLInputElement;
const allowNewWhenNoDueInput = document.querySelector('#allowNewWhenNoDueInput') as HTMLInputElement;
const quietStartInput = document.querySelector('#quietStartInput') as HTMLInputElement;
const quietEndInput = document.querySelector('#quietEndInput') as HTMLInputElement;
const weekdayProfileInput = document.querySelector('#weekdayProfileInput') as HTMLSelectElement;
const weekendProfileInput = document.querySelector('#weekendProfileInput') as HTMLSelectElement;
const autoHideInput = document.querySelector('#autoHideInput') as HTMLInputElement;
const revealOrderSelect = document.querySelector('#revealOrderSelect') as HTMLSelectElement;
const showPhoneticInput = document.querySelector('#showPhoneticInput') as HTMLInputElement;
const allowSkipInput = document.querySelector('#allowSkipInput') as HTMLInputElement;
const shortcutsEnabledInput = document.querySelector('#shortcutsEnabledInput') as HTMLInputElement;
const launchAtLoginInput = document.querySelector('#launchAtLoginInput') as HTMLInputElement;
const startBehaviorSelect = document.querySelector('#startBehaviorSelect') as HTMLSelectElement;
const trayEnabledInput = document.querySelector('#trayEnabledInput') as HTMLInputElement;

const pauseOneHourBtn = document.querySelector('#pauseOneHourBtn') as HTMLButtonElement;
const pauseTodayBtn = document.querySelector('#pauseTodayBtn') as HTMLButtonElement;
const resumeBtn = document.querySelector('#resumeBtn') as HTMLButtonElement;
const openStatsBtn = document.querySelector('#openStatsBtn') as HTMLButtonElement;
const saveConfigBtn = document.querySelector('#saveConfigBtn') as HTMLButtonElement;
const restoreRecommendedBtn = document.querySelector('#restoreRecommendedBtn') as HTMLButtonElement;

const heroSummary = document.querySelector('#heroSummary') as HTMLElement;
const statusChip = document.querySelector('#statusChip') as HTMLElement;
const strategyChip = document.querySelector('#strategyChip') as HTMLElement;
const recommendationChip = document.querySelector('#recommendationChip') as HTMLElement;
const modePill = document.querySelector('#modePill') as HTMLElement;
const recommendationText = document.querySelector('#recommendationText') as HTMLElement;
const recommendationReasonList = document.querySelector('#recommendationReasonList') as HTMLElement;
const saveHint = document.querySelector('#saveHint') as HTMLElement;
const stateBanner = document.querySelector('#stateBanner') as HTMLElement;
const stateBannerTitle = document.querySelector('#stateBannerTitle') as HTMLElement;
const stateBannerBody = document.querySelector('#stateBannerBody') as HTMLElement;

const metricTotalReviews = document.querySelector('#metricTotalReviews') as HTMLElement;
const metricAccuracy = document.querySelector('#metricAccuracy') as HTMLElement;
const metricNewWords = document.querySelector('#metricNewWords') as HTMLElement;
const metricDueCards = document.querySelector('#metricDueCards') as HTMLElement;

const diagCurrentStatus = document.querySelector('#diagCurrentStatus') as HTMLElement;
const diagBlockReason = document.querySelector('#diagBlockReason') as HTMLElement;
const diagNextReminder = document.querySelector('#diagNextReminder') as HTMLElement;
const diagLastShow = document.querySelector('#diagLastShow') as HTMLElement;

const teamTemplateSelect = document.querySelector('#teamTemplateSelect') as HTMLSelectElement;
const teamTemplateName = document.querySelector('#teamTemplateName') as HTMLElement;
const teamTemplateDescription = document.querySelector('#teamTemplateDescription') as HTMLElement;
const teamTemplateSummary = document.querySelector('#teamTemplateSummary') as HTMLElement;
const applyTemplateBtn = document.querySelector('#applyTemplateBtn') as HTMLButtonElement;

const feedbackTooManyBtn = document.querySelector('#feedbackTooManyBtn') as HTMLButtonElement;
const feedbackTooFewBtn = document.querySelector('#feedbackTooFewBtn') as HTMLButtonElement;
const feedbackList = document.querySelector('#feedbackList') as HTMLElement;

const generateExportBtn = document.querySelector('#generateExportBtn') as HTMLButtonElement;
const copyExportSummaryBtn = document.querySelector('#copyExportSummaryBtn') as HTMLButtonElement;
const copyExportJsonBtn = document.querySelector('#copyExportJsonBtn') as HTMLButtonElement;
const importConfigBtn = document.querySelector('#importConfigBtn') as HTMLButtonElement;
const importConfigFileInput = document.querySelector('#importConfigFileInput') as HTMLInputElement;
const uploadWordbookBtn = document.querySelector('#uploadWordbookBtn') as HTMLButtonElement;
const uploadWordbookFileInput = document.querySelector('#uploadWordbookFileInput') as HTMLInputElement;
const wordbookUploadHint = document.querySelector('#wordbookUploadHint') as HTMLElement;
const wordbookList = document.querySelector('#wordbookList') as HTMLElement;
const downloadExportJsonBtn = document.querySelector('#downloadExportJsonBtn') as HTMLButtonElement;
const exportSummaryOutput = document.querySelector('#exportSummaryOutput') as HTMLTextAreaElement;
const exportJsonOutput = document.querySelector('#exportJsonOutput') as HTMLTextAreaElement;

const onboardingBackdrop = document.querySelector('#onboardingBackdrop') as HTMLElement;
const onboardingDailyNewInput = document.querySelector('#onboardingDailyNewInput') as HTMLInputElement;
const onboardingModeSelect = document.querySelector('#onboardingModeSelect') as HTMLSelectElement;
const onboardingQuietStartInput = document.querySelector('#onboardingQuietStartInput') as HTMLInputElement;
const onboardingQuietEndInput = document.querySelector('#onboardingQuietEndInput') as HTMLInputElement;
const onboardingLaunchAtLoginInput = document.querySelector('#onboardingLaunchAtLoginInput') as HTMLInputElement;
const completeOnboardingBtn = document.querySelector('#completeOnboardingBtn') as HTMLButtonElement;

let currentConfig: AppConfig = createDefaultAppConfig();
let currentDashboard: DashboardState | null = null;
let currentTemplates: TeamTemplate[] = [];
let currentWordbooks: WordbookListItem[] = [];
let currentExportBundle: ExportBundle | null = null;
let schedulerStarted = false;
let saveHintTimer: number | null = null;
let lastErrorMessage: string | null = null;

function cloneConfig(config: AppConfig): AppConfig {
  return JSON.parse(JSON.stringify(config)) as AppConfig;
}

function setSaveHint(message: string) {
  saveHint.textContent = message;

  if (saveHintTimer !== null) {
    window.clearTimeout(saveHintTimer);
  }

  saveHintTimer = window.setTimeout(() => {
    saveHint.textContent = '默认使用系统推荐。你可以直接覆盖，之后也能一键恢复。';
  }, 2400);
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === 'string') {
    return error;
  }

  return '出现了未预期的错误。';
}

async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch (_error) {
    const helper = document.createElement('textarea');
    helper.value = text;
    helper.setAttribute('readonly', 'true');
    helper.style.position = 'fixed';
    helper.style.opacity = '0';
    document.body.appendChild(helper);
    helper.select();
    const copied = document.execCommand('copy');
    document.body.removeChild(helper);
    return copied;
  }
}

async function fileToBase64(file: File): Promise<string> {
  const bytes = new Uint8Array(await file.arrayBuffer());
  let binary = '';
  const chunkSize = 0x8000;

  for (let offset = 0; offset < bytes.length; offset += chunkSize) {
    const chunk = bytes.subarray(offset, offset + chunkSize);
    binary += String.fromCharCode(...chunk);
  }

  return btoa(binary);
}

function downloadTextFile(content: string, filename: string, mimeType: string) {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  URL.revokeObjectURL(url);
}

function parseClockToMinutes(value: string): number {
  const [hourRaw = '0', minuteRaw = '0'] = value.split(':');
  const hour = Number(hourRaw);
  const minute = Number(minuteRaw);
  return hour * 60 + minute;
}

function readNumberInput(input: HTMLInputElement, fallback: number): number {
  const value = input.valueAsNumber;
  return Number.isFinite(value) ? value : fallback;
}

function isInQuietHours(config: AppConfig, now: Date = new Date()): boolean {
  const currentMinutes = now.getHours() * 60 + now.getMinutes();
  const start = parseClockToMinutes(config.schedule.quiet_hours_start);
  const end = parseClockToMinutes(config.schedule.quiet_hours_end);

  if (start === end) {
    return false;
  }

  if (start < end) {
    return currentMinutes >= start && currentMinutes < end;
  }

  return currentMinutes >= start || currentMinutes < end;
}

function getQuietHoursResumeText(config: AppConfig, now: Date = new Date()): string {
  const [endHourRaw = '7', endMinuteRaw = '0'] = config.schedule.quiet_hours_end.split(':');
  const endHour = Number(endHourRaw);
  const endMinute = Number(endMinuteRaw);
  const resumeAt = new Date(now);

  if (isInQuietHours(config, now)) {
    const currentMinutes = now.getHours() * 60 + now.getMinutes();
    if (currentMinutes >= parseClockToMinutes(config.schedule.quiet_hours_start)) {
      resumeAt.setDate(resumeAt.getDate() + 1);
    }
  }

  resumeAt.setHours(endHour, endMinute, 0, 0);
  return resumeAt.toLocaleString('zh-CN', {
    month: 'numeric',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function formatDateTime(value?: string): string {
  if (!value) {
    return '暂无';
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return '暂无';
  }

  return date.toLocaleString('zh-CN', {
    month: 'numeric',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function getCurrentStatus(snapshot: SchedulerSnapshot): string {
  if (snapshot.is_card_visible) {
    return '卡片展示中';
  }

  if (snapshot.is_paused) {
    return snapshot.pause_until ? `已暂停至 ${formatDateTime(snapshot.pause_until)}` : '已暂停';
  }

  if (isInQuietHours(currentConfig)) {
    return '夜间静默中';
  }

  if (snapshot.last_block_reason === 'main_window_active') {
    return '主页面使用中';
  }

  if (snapshot.last_block_reason === 'no_card') {
    return '当前无可学习卡片';
  }

  return '运行中';
}

function getNextReminderHint(snapshot: SchedulerSnapshot): string {
  if (snapshot.is_paused && snapshot.pause_until) {
    return `暂停结束后恢复，预计 ${formatDateTime(snapshot.pause_until)}`;
  }

  if (isInQuietHours(currentConfig)) {
    return `静默结束后恢复，预计 ${getQuietHoursResumeText(currentConfig)}`;
  }

  if (snapshot.last_block_reason === 'main_window_active') {
    return '当前主页面正在使用中，离开设置控制台后才会恢复自动提醒';
  }

  if (snapshot.last_block_reason === 'no_card') {
    return '当前没有可学习卡片，词库或复习到期后会恢复提醒';
  }

  if (snapshot.is_card_visible) {
    return '当前已有浮卡展示中，完成后再评估下一次提醒';
  }

  if (snapshot.last_show_time && snapshot.fallback_enabled) {
    const lastShow = new Date(snapshot.last_show_time);
    const nextFallback = new Date(lastShow.getTime() + snapshot.fallback_interval_min * 60 * 1000);
    const diffMinutes = Math.max(0, Math.ceil((nextFallback.getTime() - Date.now()) / (60 * 1000)));
    return diffMinutes > 0
      ? `若持续空闲不足，最晚 ${diffMinutes} 分钟后兜底提醒`
      : `已满足兜底条件，空闲达到阈值后会提醒`;
  }

  return `满足空闲 ${snapshot.idle_threshold_sec} 秒后提醒`;
}

function getRecommendationText(config: AppConfig, recommendation?: RecommendationSummary): string {
  const effectiveReminder = getEffectiveReminderConfig(config);

  if (!recommendation) {
    if (!config.reminder.using_recommended || config.reminder.mode === 'custom') {
      return `当前使用自定义策略：空闲 ${effectiveReminder.idle_threshold_sec} 秒触发，兜底 ${effectiveReminder.fallback_interval_min} 分钟。`;
    }

    const scheduleLabel = getScheduleSegmentLabel();
    const weekdayMode = getModeLabel(config.schedule.weekday_profile ?? config.reminder.mode);
    const weekendMode = getModeLabel(config.schedule.weekend_profile ?? config.reminder.mode);
    const activeModeLabel = getModeLabel(effectiveReminder.mode);

    return `系统今天按${scheduleLabel}${activeModeLabel}模式运行：空闲 ${effectiveReminder.idle_threshold_sec} 秒触发，兜底 ${effectiveReminder.fallback_interval_min} 分钟。工作日 ${weekdayMode}，周末 ${weekendMode}。`;
  }

  if (!config.reminder.using_recommended || config.reminder.mode === 'custom') {
    return `当前使用自定义策略：空闲 ${effectiveReminder.idle_threshold_sec} 秒触发，兜底 ${effectiveReminder.fallback_interval_min} 分钟。${recommendation.explanation}`;
  }

  return recommendation.explanation;
}

function renderRecommendationReasons(recommendation?: RecommendationSummary) {
  const reasons = recommendation?.reasons?.length
    ? recommendation.reasons
    : ['系统会根据最近的跳过率、暂停状态和反馈信号解释当前建议。'];
  recommendationReasonList.innerHTML = reasons.map((reason) => `<li>${reason}</li>`).join('');
}

function renderFeedbackHistory(records: FeedbackRecord[] = []) {
  if (!records.length) {
    feedbackList.innerHTML = '<li>暂无反馈记录</li>';
    return;
  }

  feedbackList.innerHTML = records
    .map((record) => {
      const label = record.feedback_type === 'too_many_reminders'
        ? '提醒太多'
        : record.feedback_type === 'too_few_reminders'
          ? '提醒太少'
          : `这张词先别再推${record.word ? ` · ${record.word}` : ''}`;
      return `<li>${label} · ${formatDateTime(record.created_at)} · 来自主${record.source === 'card' ? '卡片' : '页面'}</li>`;
    })
    .join('');
}

function renderTemplateSummary() {
  const selected = currentTemplates.find((template) => template.id === teamTemplateSelect.value) ?? currentTemplates[0];

  if (!selected) {
    teamTemplateName.textContent = '暂无模板';
    teamTemplateDescription.textContent = '当前还没有可用的团队模板。';
    teamTemplateSummary.textContent = '后续可继续补充不同部门的默认策略。';
    return;
  }

  teamTemplateName.textContent = selected.name;
  teamTemplateDescription.textContent = selected.description;
  teamTemplateSummary.textContent = selected.summary;
}

function renderExportBundle() {
  exportSummaryOutput.value = currentExportBundle?.summary_text ?? '点击“生成摘要”后在这里查看。';
  exportJsonOutput.value = currentExportBundle?.config_json ?? '点击“生成摘要”后在这里查看。';
}

function renderWordbooks() {
  if (!currentWordbooks.length) {
    wordbookList.innerHTML = '<div class="wordbook-empty">当前还没有可用词库。</div>';
    return;
  }

  wordbookList.innerHTML = '';

  currentWordbooks.forEach((item) => {
    const row = document.createElement('article');
    row.className = 'wordbook-item';

    const copy = document.createElement('div');
    const title = document.createElement('strong');
    title.textContent = item.display_name;
    copy.appendChild(title);

    const badgeWrap = document.createElement('div');
    badgeWrap.className = 'wordbook-item-meta';
    badgeWrap.textContent = `${item.total_words} 个单词 · ${item.enabled ? '启用中' : '已停用'}${item.last_created_at ? ` · 最近导入 ${formatDateTime(item.last_created_at)}` : ''}`;
    copy.appendChild(badgeWrap);

    const actions = document.createElement('div');
    actions.className = 'wordbook-item-actions';

    const sourceBadge = document.createElement('span');
    sourceBadge.className = `wordbook-badge${item.built_in ? '' : ' muted'}`;
    sourceBadge.textContent = item.built_in ? '内置词库' : item.source;
    actions.appendChild(sourceBadge);

    const toggleButton = document.createElement('button');
    toggleButton.className = 'ghost-btn';
    toggleButton.type = 'button';
    toggleButton.dataset.source = item.source;
    toggleButton.dataset.action = 'toggle';
    toggleButton.textContent = item.enabled ? '停用' : '启用';
    actions.appendChild(toggleButton);

    const deleteButton = document.createElement('button');
    deleteButton.className = 'ghost-btn';
    deleteButton.type = 'button';
    deleteButton.dataset.source = item.source;
    deleteButton.dataset.action = 'delete';
    deleteButton.textContent = '删除';
    actions.appendChild(deleteButton);

    row.append(copy, actions);
    wordbookList.appendChild(row);
  });
}

function renderStateBanner(snapshot: SchedulerSnapshot) {
  const notice = (() => {
    if (lastErrorMessage) {
      return {
        title: '主页面状态读取失败',
        body: `${lastErrorMessage} 你仍然可以调整本地表单，但建议先刷新或重启应用确认状态同步。`,
      };
    }

    if (currentDashboard?.needs_onboarding) {
      return {
        title: '先完成首次设置',
        body: '应用会在你确认新词目标、静默时间和提醒强度后再开始调度。',
      };
    }

    if (snapshot.is_paused) {
      return {
        title: '提醒已暂停',
        body: snapshot.pause_until
          ? `当前暂停会持续到 ${formatDateTime(snapshot.pause_until)}，你也可以随时手动恢复。`
          : '当前处于暂停状态，恢复后才会继续弹卡。',
      };
    }

    if (snapshot.last_block_reason === 'no_card') {
      return {
        title: '当前无卡可学',
        body: '调度器已检查到暂时没有可展示的词卡。词库补充或复习到期后会自动恢复。',
      };
    }

    if (isInQuietHours(currentConfig)) {
      return {
        title: '当前处于静默时段',
        body: `静默结束后会恢复提醒，预计 ${getQuietHoursResumeText(currentConfig)} 重新开始。`,
      };
    }

    return null;
  })();

  stateBanner.classList.toggle('hidden', notice === null);
  if (!notice) {
    return;
  }

  stateBannerTitle.textContent = notice.title;
  stateBannerBody.textContent = notice.body;
}

function syncReminderFieldStates(mode: ReminderMode) {
  const isCustomMode = mode === 'custom';
  idleThresholdInput.disabled = !isCustomMode;
  fallbackEnabledInput.disabled = !isCustomMode;
  fallbackIntervalInput.disabled = !isCustomMode;
  weekdayProfileInput.disabled = isCustomMode;
  weekendProfileInput.disabled = isCustomMode;
}

function syncSystemFieldStates() {
  const trayDisabled = !trayEnabledInput.checked;
  startBehaviorSelect.disabled = trayDisabled;

  if (trayDisabled && startBehaviorSelect.value === 'minimize-to-tray') {
    startBehaviorSelect.value = 'show-main';
  }
}

function populateForm(config: AppConfig) {
  const effectiveReminder = getEffectiveReminderConfig(config);

  modeSelect.value = config.reminder.mode;
  idleThresholdInput.value = String(effectiveReminder.idle_threshold_sec);
  fallbackEnabledInput.checked = effectiveReminder.fallback_enabled;
  fallbackIntervalInput.value = String(effectiveReminder.fallback_interval_min);

  dailyNewLimitInput.value = String(config.learning.daily_new_limit);
  reviewFirstInput.checked = config.learning.review_first;
  allowNewWhenNoDueInput.checked = config.learning.allow_new_when_no_due;

  quietStartInput.value = config.schedule.quiet_hours_start;
  quietEndInput.value = config.schedule.quiet_hours_end;
  weekdayProfileInput.value = config.schedule.weekday_profile ?? 'gentle';
  weekendProfileInput.value = config.schedule.weekend_profile ?? 'balanced';

  autoHideInput.value = String(config.card.auto_hide_sec);
  revealOrderSelect.value = config.card.reveal_order;
  showPhoneticInput.checked = config.card.show_phonetic;
  allowSkipInput.checked = config.card.allow_skip;
  shortcutsEnabledInput.checked = config.card.shortcuts_enabled;

  launchAtLoginInput.checked = config.system.launch_at_login;
  startBehaviorSelect.value = config.system.start_behavior;
  trayEnabledInput.checked = config.system.tray_enabled;

  syncReminderFieldStates(config.reminder.mode);
  syncSystemFieldStates();
}

function populateOnboarding(config: AppConfig) {
  onboardingDailyNewInput.value = String(config.learning.daily_new_limit);
  onboardingModeSelect.value = (config.schedule.weekday_profile ?? config.reminder.mode) as RecommendedReminderMode;
  onboardingQuietStartInput.value = config.schedule.quiet_hours_start;
  onboardingQuietEndInput.value = config.schedule.quiet_hours_end;
  onboardingLaunchAtLoginInput.checked = config.system.launch_at_login;
}

function readConfigFromForm(): AppConfig {
  const mode = modeSelect.value as ReminderMode;
  let config = cloneConfig(currentConfig);

  if (mode !== 'custom') {
    config = applyModePreset(config, mode);
  } else {
    config.reminder.idle_threshold_sec = readNumberInput(idleThresholdInput, config.reminder.idle_threshold_sec);
    config.reminder.fallback_enabled = fallbackEnabledInput.checked;
    config.reminder.fallback_interval_min = readNumberInput(
      fallbackIntervalInput,
      config.reminder.fallback_interval_min,
    );
    config.reminder.mode = 'custom';
    config.reminder.using_recommended = false;
  }

  config.learning.daily_new_limit = readNumberInput(dailyNewLimitInput, config.learning.daily_new_limit);
  config.learning.review_first = reviewFirstInput.checked;
  config.learning.allow_new_when_no_due = allowNewWhenNoDueInput.checked;

  config.schedule.quiet_hours_start = quietStartInput.value || config.schedule.quiet_hours_start;
  config.schedule.quiet_hours_end = quietEndInput.value || config.schedule.quiet_hours_end;
  config.schedule.weekday_profile = weekdayProfileInput.value as RecommendedReminderMode;
  config.schedule.weekend_profile = weekendProfileInput.value as RecommendedReminderMode;

  config.card.auto_hide_sec = readNumberInput(autoHideInput, config.card.auto_hide_sec);
  config.card.reveal_order = revealOrderSelect.value as AppConfig['card']['reveal_order'];
  config.card.show_phonetic = showPhoneticInput.checked;
  config.card.allow_skip = allowSkipInput.checked;
  config.card.shortcuts_enabled = shortcutsEnabledInput.checked;

  config.system.launch_at_login = launchAtLoginInput.checked;
  config.system.start_behavior = startBehaviorSelect.value as AppConfig['system']['start_behavior'];
  config.system.tray_enabled = trayEnabledInput.checked;

  if (!config.system.tray_enabled && config.system.start_behavior === 'minimize-to-tray') {
    config.system.start_behavior = 'show-main';
  }

  return config;
}

function readOnboardingConfig(): AppConfig {
  const mode = onboardingModeSelect.value as RecommendedReminderMode;
  let config = applyModePreset(createDefaultAppConfig(), mode);

  config.schedule.weekday_profile = mode;
  config.schedule.weekend_profile = mode;
  config.schedule.quiet_hours_start = onboardingQuietStartInput.value || config.schedule.quiet_hours_start;
  config.schedule.quiet_hours_end = onboardingQuietEndInput.value || config.schedule.quiet_hours_end;
  config.learning.daily_new_limit = readNumberInput(onboardingDailyNewInput, config.learning.daily_new_limit);
  config.system.launch_at_login = onboardingLaunchAtLoginInput.checked;

  return config;
}

async function syncLaunchAtLogin(enabled: boolean): Promise<boolean> {
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

function renderMetrics(stats: TodayStats | undefined) {
  metricTotalReviews.textContent = String(stats?.total_reviews ?? 0);
  metricAccuracy.textContent = `${(stats?.accuracy ?? 0).toFixed(0)}%`;
  metricNewWords.textContent = String(stats?.new_words_today ?? 0);
  metricDueCards.textContent = String(stats?.due_cards_count ?? 0);
}

function renderConsole() {
  const snapshot = triggerScheduler.getSnapshot();
  const stats = currentDashboard?.today_stats;
  const effectiveReminder = getEffectiveReminderConfig(currentConfig);
  const modeLabel = getModeLabel(effectiveReminder.mode);
  const scheduleSegmentLabel = getScheduleSegmentLabel();
  const isRecommended = currentConfig.reminder.using_recommended && currentConfig.reminder.mode !== 'custom';
  const recommendation = currentDashboard?.recommendation;

  renderMetrics(stats);

  statusChip.textContent = getCurrentStatus(snapshot);
  strategyChip.textContent = isRecommended ? `今日${scheduleSegmentLabel}：${modeLabel}` : `当前模式：${modeLabel}`;
  recommendationChip.textContent = isRecommended ? `系统推荐 · ${scheduleSegmentLabel}` : '用户自定义';
  modePill.textContent = modeLabel;
  recommendationText.textContent = getRecommendationText(currentConfig, recommendation);
  renderRecommendationReasons(recommendation);

  heroSummary.textContent = isRecommended
    ? `今天是${scheduleSegmentLabel}，当前按${modeLabel}策略运行；系统先给推荐值，你仍然可以随时覆盖这些参数。`
    : `当前使用自定义策略，空闲 ${effectiveReminder.idle_threshold_sec} 秒后提醒；系统仍会给出建议，但不会自动覆盖你的设置。`;

  diagCurrentStatus.textContent = getCurrentStatus(snapshot);
  diagBlockReason.textContent = getBlockReasonLabel(snapshot.last_block_reason);
  diagNextReminder.textContent = getNextReminderHint(snapshot);
  diagLastShow.textContent = formatDateTime(snapshot.last_show_time);
  renderFeedbackHistory(currentDashboard?.recent_feedback);
  renderTemplateSummary();
  renderExportBundle();
  renderStateBanner(snapshot);
}

function syncOnboardingVisibility(visible: boolean) {
  onboardingBackdrop.classList.toggle('hidden', !visible);
  document.body.classList.toggle('modal-open', visible);
}

async function loadTeamTemplates() {
  currentTemplates = await invoke<TeamTemplate[]>('list_team_templates');
  teamTemplateSelect.innerHTML = currentTemplates
    .map((template) => `<option value="${template.id}">${template.name}</option>`)
    .join('');
  renderTemplateSummary();
}

async function loadWordbooks() {
  currentWordbooks = await invoke<WordbookListItem[]>('list_wordbooks');
  renderWordbooks();
}

async function generateExportBundle() {
  currentExportBundle = await invoke<ExportBundle>('get_export_bundle');
  renderExportBundle();
}

async function importConfigFromJson(raw: string) {
  const importedConfig = JSON.parse(raw) as AppConfig;
  const requestedLaunchAtLogin = importedConfig.system?.launch_at_login ?? false;
  const launchAtLogin = await syncLaunchAtLogin(requestedLaunchAtLogin);
  importedConfig.system = {
    ...currentConfig.system,
    ...importedConfig.system,
    launch_at_login: launchAtLogin,
  };

  currentConfig = await invoke<AppConfig>('update_app_config', { config: importedConfig });
  currentExportBundle = null;
  await refreshDashboard();
  setSaveHint('配置已导入，你可以继续在主页面微调。');
}

async function importCustomWordbook(file: File) {
  const contentBase64 = await fileToBase64(file);
  const summary = await invoke<WordbookImportSummary>('import_custom_wordbook', {
    fileName: file.name,
    contentBase64,
  });

  await loadWordbooks();
  await refreshDashboard();
  currentExportBundle = null;
  wordbookUploadHint.textContent = `已导入 ${summary.imported_count} 个单词，跳过 ${summary.skipped_count} 个重复或无效条目。格式：${summary.format.toUpperCase()} · 来源：${file.name}`;
  setSaveHint(`词库导入完成，新增 ${summary.imported_count} 个单词。`);
}

async function toggleWordbook(source: string, enabled: boolean) {
  try {
    currentWordbooks = await invoke<WordbookListItem[]>('set_wordbook_enabled', {
      source,
      enabled,
    });
    renderWordbooks();
    await refreshDashboard();
    setSaveHint(enabled ? '词库已重新启用。' : '词库已停用，之后不会继续出题。');
  } catch (error) {
    lastErrorMessage = getErrorMessage(error);
    renderConsole();
    setSaveHint('更新词库状态失败，请稍后重试。');
  }
}

async function deleteWordbookBySource(source: string) {
  try {
    currentWordbooks = await invoke<WordbookListItem[]>('delete_wordbook', { source });
    renderWordbooks();
    await refreshDashboard();
    setSaveHint('词库已删除，对应单词和学习记录也已移除。');
  } catch (error) {
    lastErrorMessage = getErrorMessage(error);
    renderConsole();
    setSaveHint('删除词库失败，请稍后重试。');
  }
}

async function submitFeedback(
  feedbackType: FeedbackType,
  source: 'console' | 'card',
  extra: Partial<Pick<FeedbackRecord, 'card_id' | 'word'>> = {},
) {
  try {
    currentDashboard = currentDashboard
      ? {
          ...currentDashboard,
          recent_feedback: await invoke<FeedbackRecord[]>('record_feedback', {
            feedbackType,
            source,
            cardId: extra.card_id,
            word: extra.word,
          }),
        }
      : currentDashboard;
    await refreshDashboard();
    setSaveHint('反馈已记录，后续推荐会参考这条信号。');
  } catch (error) {
    lastErrorMessage = getErrorMessage(error);
    renderConsole();
    setSaveHint('记录反馈失败，请稍后再试。');
  }
}

async function applySelectedTemplate() {
  try {
    const selected = currentTemplates.find((template) => template.id === teamTemplateSelect.value);

    if (!selected) {
      setSaveHint('当前没有可应用的模板。');
      return;
    }

    const requestedLaunchAtLogin = selected.config.system.launch_at_login;
    const launchAtLogin = await syncLaunchAtLogin(requestedLaunchAtLogin);
    const nextConfig = cloneConfig(selected.config);
    nextConfig.system.launch_at_login = launchAtLogin;
    currentConfig = await invoke<AppConfig>('update_app_config', { config: nextConfig });
    currentExportBundle = null;
    await refreshDashboard();
    setSaveHint(`${selected.name}模板已应用，你可以继续在主页面微调。`);
  } catch (error) {
    lastErrorMessage = getErrorMessage(error);
    renderConsole();
    setSaveHint('应用模板失败，请稍后重试。');
  }
}

async function refreshDashboard() {
  try {
    currentDashboard = await invoke<DashboardState>('get_dashboard_state');
    currentConfig = currentDashboard.app_config ?? currentConfig;
    triggerScheduler.updateConfig(currentConfig);
    triggerScheduler.syncPauseUntil(currentDashboard.pause_until);
    populateForm(currentConfig);
    populateOnboarding(currentConfig);
    syncOnboardingVisibility(currentDashboard.needs_onboarding);
    lastErrorMessage = null;
  } catch (error) {
    lastErrorMessage = getErrorMessage(error);
  }

  renderConsole();
}

async function saveConfig() {
  try {
    const previousConfig = cloneConfig(currentConfig);
    const nextConfig = readConfigFromForm();
    const requestedLaunchAtLogin = nextConfig.system.launch_at_login;
    const launchAtLogin = await syncLaunchAtLogin(requestedLaunchAtLogin);
    nextConfig.system.launch_at_login = launchAtLogin;

    currentConfig = await invoke<AppConfig>('update_app_config', { config: nextConfig });
    currentExportBundle = null;
    triggerScheduler.updateConfig(currentConfig);
    populateForm(currentConfig);
    renderConsole();

    const requiresRestartNotice =
      previousConfig.system.start_behavior !== currentConfig.system.start_behavior ||
      previousConfig.system.tray_enabled !== currentConfig.system.tray_enabled;

    if (launchAtLogin !== requestedLaunchAtLogin) {
      setSaveHint('设置已保存，但开机启动未能按预期更新。');
      return;
    }

    setSaveHint(
      requiresRestartNotice
        ? '设置已保存。启动行为和托盘开关会在下次启动时完全生效。'
        : '设置已保存，调度器会按新配置继续运行。',
    );
  } catch (error) {
    lastErrorMessage = getErrorMessage(error);
    renderConsole();
    setSaveHint('保存失败，请检查当前配置后重试。');
  }
}

async function restoreRecommended() {
  try {
    currentConfig = applyModePreset(currentConfig, 'gentle');
    currentConfig.reminder.using_recommended = true;
    currentConfig.reminder.mode = 'gentle';
    currentConfig.schedule.weekday_profile = 'gentle';
    currentConfig.schedule.weekend_profile = 'balanced';
    currentConfig = await invoke<AppConfig>('update_app_config', { config: currentConfig });
    currentExportBundle = null;
    triggerScheduler.updateConfig(currentConfig);
    populateForm(currentConfig);
    renderConsole();
    setSaveHint('已恢复为默认推荐值。');
  } catch (error) {
    lastErrorMessage = getErrorMessage(error);
    renderConsole();
    setSaveHint('恢复推荐值失败，请稍后再试。');
  }
}

async function completeOnboarding() {
  completeOnboardingBtn.disabled = true;

  try {
    const nextConfig = readOnboardingConfig();
    const requestedLaunchAtLogin = nextConfig.system.launch_at_login;
    const launchAtLogin = await syncLaunchAtLogin(requestedLaunchAtLogin);
    nextConfig.system.launch_at_login = launchAtLogin;

    currentConfig = await invoke<AppConfig>('complete_onboarding', { config: nextConfig });
    currentExportBundle = null;
    await refreshDashboard();

    if (!schedulerStarted) {
      initScheduler();
    }

    if (launchAtLogin !== requestedLaunchAtLogin) {
      setSaveHint('首次设置已完成，但开机启动未能按预期更新。');
      return;
    }

    setSaveHint('首次设置已完成，系统会按你的偏好开始提醒。');
  } catch (error) {
    lastErrorMessage = getErrorMessage(error);
    renderConsole();
    setSaveHint('首次设置保存失败，请再试一次。');
  } finally {
    completeOnboardingBtn.disabled = false;
  }
}

async function pauseScheduler(minutes: number) {
  try {
    await invoke('pause_scheduler', { minutes });
    triggerScheduler.pause(minutes);
    await refreshDashboard();
  } catch (error) {
    lastErrorMessage = getErrorMessage(error);
    renderConsole();
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
    lastErrorMessage = getErrorMessage(error);
    renderConsole();
    setSaveHint('恢复失败，请稍后再试。');
  }
}

function initScheduler() {
  if (schedulerStarted) {
    return;
  }

  triggerScheduler.updateConfig(currentConfig);
  triggerScheduler.syncPauseUntil(currentDashboard?.pause_until);
  triggerScheduler.start(async () => {
    await invoke('show_card_window');
    triggerScheduler.markCardShown();
    renderConsole();
  });

  void listen('card-hidden', async () => {
    triggerScheduler.markCardHidden();
    await refreshDashboard();
  });

  void listen('scheduler-paused', async (event) => {
    const minutes = Number(event.payload);
    if (!Number.isNaN(minutes) && minutes > 0) {
      triggerScheduler.pause(minutes);
      await refreshDashboard();
    }
  });

  schedulerStarted = true;
}

function wireControls() {
  modeSelect.addEventListener('change', () => {
    const mode = modeSelect.value as ReminderMode;
    if (mode !== 'custom') {
      currentConfig = applyModePreset(currentConfig, mode);
      currentConfig.schedule.weekday_profile ??= mode;
      currentConfig.schedule.weekend_profile ??= mode;
      populateForm(currentConfig);
      renderConsole();
      setSaveHint(`已切换到${getModeLabel(mode)}模式，保存后正式生效。`);
      return;
    }

    currentConfig.reminder.idle_threshold_sec = readNumberInput(idleThresholdInput, currentConfig.reminder.idle_threshold_sec);
    currentConfig.reminder.fallback_enabled = fallbackEnabledInput.checked;
    currentConfig.reminder.fallback_interval_min = readNumberInput(
      fallbackIntervalInput,
      currentConfig.reminder.fallback_interval_min,
    );
    currentConfig.reminder.mode = 'custom';
    currentConfig.reminder.using_recommended = false;
    syncReminderFieldStates('custom');
    renderConsole();
  });

  trayEnabledInput.addEventListener('change', () => {
    syncSystemFieldStates();
    if (!trayEnabledInput.checked) {
      setSaveHint('关闭托盘后，启动行为会自动回退为显示主页面。');
    }
  });

  teamTemplateSelect.addEventListener('change', () => {
    renderTemplateSummary();
  });

  openStatsBtn.addEventListener('click', async () => {
    await invoke('show_stats_window');
  });
  saveConfigBtn.addEventListener('click', () => {
    void saveConfig();
  });
  restoreRecommendedBtn.addEventListener('click', () => {
    void restoreRecommended();
  });
  pauseOneHourBtn.addEventListener('click', () => {
    void pauseScheduler(60);
  });
  pauseTodayBtn.addEventListener('click', () => {
    void pauseUntilEndOfDay();
  });
  resumeBtn.addEventListener('click', () => {
    void resumeScheduler();
  });
  applyTemplateBtn.addEventListener('click', () => {
    void applySelectedTemplate();
  });
  feedbackTooManyBtn.addEventListener('click', () => {
    void submitFeedback('too_many_reminders', 'console');
  });
  feedbackTooFewBtn.addEventListener('click', () => {
    void submitFeedback('too_few_reminders', 'console');
  });
  generateExportBtn.addEventListener('click', () => {
    void (async () => {
      try {
        await generateExportBundle();
        setSaveHint('已生成当前配置摘要，可直接复制给团队讨论。');
      } catch (error) {
        lastErrorMessage = getErrorMessage(error);
        renderConsole();
        setSaveHint('生成导出摘要失败，请稍后重试。');
      }
    })();
  });
  copyExportSummaryBtn.addEventListener('click', () => {
    void (async () => {
      if (!currentExportBundle) {
        await generateExportBundle();
      }
      const copied = await copyToClipboard(currentExportBundle?.summary_text ?? '');
      setSaveHint(copied ? '配置摘要已复制。' : '复制失败，请检查系统剪贴板权限。');
    })();
  });
  copyExportJsonBtn.addEventListener('click', () => {
    void (async () => {
      if (!currentExportBundle) {
        await generateExportBundle();
      }
      const copied = await copyToClipboard(currentExportBundle?.config_json ?? '');
      setSaveHint(copied ? '配置 JSON 已复制。' : '复制失败，请检查系统剪贴板权限。');
    })();
  });
  importConfigBtn.addEventListener('click', () => {
    importConfigFileInput.click();
  });
  uploadWordbookBtn.addEventListener('click', () => {
    uploadWordbookFileInput.click();
  });
  wordbookList.addEventListener('click', (event) => {
    const target = event.target as HTMLElement | null;
    const button = target?.closest<HTMLButtonElement>('button[data-action][data-source]');
    if (!button) {
      return;
    }

    const source = button.dataset.source;
    const action = button.dataset.action;

    if (!source || !action) {
      return;
    }

    if (action === 'toggle') {
      const wordbook = currentWordbooks.find((item) => item.source === source);
      if (!wordbook) {
        return;
      }
      void toggleWordbook(source, !wordbook.enabled);
      return;
    }

    if (action === 'delete') {
      const confirmed = window.confirm(`确定删除词库“${source}”吗？删除后对应单词和学习记录会一起移除。`);
      if (!confirmed) {
        return;
      }
      void deleteWordbookBySource(source);
    }
  });
  importConfigFileInput.addEventListener('change', () => {
    void (async () => {
      const [file] = Array.from(importConfigFileInput.files ?? []);
      if (!file) {
        return;
      }

      try {
        const raw = await file.text();
        await importConfigFromJson(raw);
      } catch (error) {
        lastErrorMessage = getErrorMessage(error);
        renderConsole();
        setSaveHint('导入失败，请确认 JSON 来自有效的配置导出。');
      } finally {
        importConfigFileInput.value = '';
      }
    })();
  });
  uploadWordbookFileInput.addEventListener('change', () => {
    void (async () => {
      const [file] = Array.from(uploadWordbookFileInput.files ?? []);
      if (!file) {
        return;
      }

      try {
        wordbookUploadHint.textContent = `正在导入 ${file.name}...`;
        await importCustomWordbook(file);
      } catch (error) {
        lastErrorMessage = getErrorMessage(error);
        renderConsole();
        wordbookUploadHint.textContent = '导入失败。请使用 JSON / CSV / TXT / XLSX，并确保包含 word / meaning_zh。';
        setSaveHint('自定义词库导入失败，请检查文件格式后重试。');
      } finally {
        uploadWordbookFileInput.value = '';
      }
    })();
  });
  downloadExportJsonBtn.addEventListener('click', () => {
    void (async () => {
      if (!currentExportBundle) {
        await generateExportBundle();
      }
      if (currentExportBundle) {
        downloadTextFile(
          currentExportBundle.config_json,
          `${currentExportBundle.file_name_hint}.json`,
          'application/json',
        );
        setSaveHint('配置 JSON 已下载。');
      }
    })();
  });
  completeOnboardingBtn.addEventListener('click', () => {
    void completeOnboarding();
  });
}

async function bootstrap() {
  await refreshDashboard();
  try {
    await loadTeamTemplates();
    await loadWordbooks();
  } catch (error) {
    lastErrorMessage = getErrorMessage(error);
  }
  currentConfig.system.launch_at_login = await syncLaunchAtLogin(currentConfig.system.launch_at_login);
  populateForm(currentConfig);
  populateOnboarding(currentConfig);
  renderConsole();
  wireControls();
  if (!currentDashboard?.needs_onboarding) {
    initScheduler();
  }
  window.setInterval(() => {
    void refreshDashboard();
  }, 30000);
}

window.addEventListener('DOMContentLoaded', () => {
  void bootstrap();
});

window.addEventListener('beforeunload', () => {
  triggerScheduler.stop();
});
