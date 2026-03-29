import { triggerScheduler } from '../domain/scheduler/triggerScheduler';
import {
  getBlockReasonLabel,
  getEffectiveReminderConfig,
  getModeLabel,
  getScheduleSegmentLabel,
} from '../shared/config';
import type { AppConfig, FeedbackRecord, RecommendationSummary, SchedulerSnapshot, TodayStats } from '../shared/types';
import { mainElements } from './elements';
import { formatDateTime, parseClockToMinutes } from './helpers';
import { mainState } from './state';

interface DashboardDependencies {
  onApplyTemplate: () => Promise<void>;
  onCopyExportJson: () => Promise<void>;
  onCopyExportSummary: () => Promise<void>;
  onDownloadExportJson: () => Promise<void>;
  onFeedbackTooFew: () => Promise<void>;
  onFeedbackTooMany: () => Promise<void>;
  onGenerateExport: () => Promise<void>;
  onImportConfigFile: (file: File) => Promise<void>;
  onOpenStats: () => Promise<void>;
  onPauseOneHour: () => Promise<void>;
  onPauseToday: () => Promise<void>;
  onResume: () => Promise<void>;
  onStartScheduler: () => Promise<void>;
  onStopScheduler: () => void;
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

function getCurrentStatus(snapshot: SchedulerSnapshot): string {
  if (snapshot.is_card_visible) {
    return '卡片展示中';
  }

  if (snapshot.is_paused) {
    return snapshot.pause_until ? `已暂停至 ${formatDateTime(snapshot.pause_until)}` : '已暂停';
  }

  if (isInQuietHours(mainState.currentConfig)) {
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

  if (isInQuietHours(mainState.currentConfig)) {
    return `静默结束后恢复，预计 ${getQuietHoursResumeText(mainState.currentConfig)}`;
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
      : '已满足兜底条件，空闲达到阈值后会提醒';
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

  mainElements.recommendationReasonList.innerHTML = reasons.map((reason) => `<li>${reason}</li>`).join('');
}

function renderFeedbackHistory(records: FeedbackRecord[] = []) {
  if (!records.length) {
    mainElements.feedbackList.innerHTML = '<li>暂无反馈记录</li>';
    return;
  }

  mainElements.feedbackList.innerHTML = records
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
  const selected = mainState.currentTemplates.find((template) => template.id === mainElements.teamTemplateSelect.value)
    ?? mainState.currentTemplates[0];

  if (!selected) {
    mainElements.teamTemplateName.textContent = '暂无模板';
    mainElements.teamTemplateDescription.textContent = '当前还没有可用的团队模板。';
    mainElements.teamTemplateSummary.textContent = '后续可继续补充不同部门的默认策略。';
    return;
  }

  mainElements.teamTemplateName.textContent = selected.name;
  mainElements.teamTemplateDescription.textContent = selected.description;
  mainElements.teamTemplateSummary.textContent = selected.summary;
}

function renderExportBundle() {
  mainElements.exportSummaryOutput.value = mainState.currentExportBundle?.summary_text ?? '点击“生成摘要”后在这里查看。';
  mainElements.exportJsonOutput.value = mainState.currentExportBundle?.config_json ?? '点击“生成摘要”后在这里查看。';
}

function renderMetrics(stats: TodayStats | undefined) {
  mainElements.metricTotalReviews.textContent = String(stats?.total_reviews ?? 0);
  mainElements.metricAccuracy.textContent = `${(stats?.accuracy ?? 0).toFixed(0)}%`;
  mainElements.metricNewWords.textContent = String(stats?.new_words_today ?? 0);
  mainElements.metricDueCards.textContent = String(stats?.due_cards_count ?? 0);
}

function renderStateBanner(snapshot: SchedulerSnapshot) {
  const notice = (() => {
    if (mainState.lastErrorMessage) {
      return {
        title: '主页面状态读取失败',
        body: `${mainState.lastErrorMessage} 你仍然可以调整本地表单，但建议先刷新或重启应用确认状态同步。`,
      };
    }

    if (mainState.currentDashboard?.needs_onboarding) {
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

    if (isInQuietHours(mainState.currentConfig)) {
      return {
        title: '当前处于静默时段',
        body: `静默结束后会恢复提醒，预计 ${getQuietHoursResumeText(mainState.currentConfig)} 重新开始。`,
      };
    }

    return null;
  })();

  mainElements.stateBanner.classList.toggle('hidden', notice === null);

  if (!notice) {
    return;
  }

  mainElements.stateBannerTitle.textContent = notice.title;
  mainElements.stateBannerBody.textContent = notice.body;
}

export function syncOnboardingVisibility(visible: boolean) {
  mainElements.onboardingBackdrop.classList.toggle('hidden', !visible);
  document.body.classList.toggle('modal-open', visible);
}

export function renderSchedulerControls(isStarted: boolean) {
  mainElements.startSchedulerBtn.style.display = isStarted ? 'none' : 'inline-block';
  mainElements.stopSchedulerBtn.style.display = isStarted ? 'inline-block' : 'none';
}

export function renderDashboard() {
  const snapshot = triggerScheduler.getSnapshot();
  const stats = mainState.currentDashboard?.today_stats;
  const effectiveReminder = getEffectiveReminderConfig(mainState.currentConfig);
  const modeLabel = getModeLabel(effectiveReminder.mode);
  const scheduleSegmentLabel = getScheduleSegmentLabel();
  const isRecommended = mainState.currentConfig.reminder.using_recommended && mainState.currentConfig.reminder.mode !== 'custom';
  const recommendation = mainState.currentDashboard?.recommendation;

  renderMetrics(stats);

  mainElements.statusChip.textContent = getCurrentStatus(snapshot);
  mainElements.strategyChip.textContent = isRecommended ? `今日${scheduleSegmentLabel}：${modeLabel}` : `当前模式：${modeLabel}`;
  mainElements.recommendationChip.textContent = isRecommended ? `系统推荐 · ${scheduleSegmentLabel}` : '用户自定义';
  mainElements.modePill.textContent = modeLabel;
  mainElements.recommendationText.textContent = getRecommendationText(mainState.currentConfig, recommendation);
  renderRecommendationReasons(recommendation);

  mainElements.heroSummary.textContent = isRecommended
    ? `今天是${scheduleSegmentLabel}，当前按${modeLabel}策略运行；系统先给推荐值，你仍然可以随时覆盖这些参数。`
    : `当前使用自定义策略，空闲 ${effectiveReminder.idle_threshold_sec} 秒后提醒；系统仍会给出建议，但不会自动覆盖你的设置。`;

  mainElements.diagCurrentStatus.textContent = getCurrentStatus(snapshot);
  mainElements.diagBlockReason.textContent = getBlockReasonLabel(snapshot.last_block_reason);
  mainElements.diagNextReminder.textContent = getNextReminderHint(snapshot);
  mainElements.diagLastShow.textContent = formatDateTime(snapshot.last_show_time);
  renderFeedbackHistory(mainState.currentDashboard?.recent_feedback);
  renderTemplateSummary();
  renderExportBundle();
  renderStateBanner(snapshot);
}

export function initializeDashboard(dependencies: DashboardDependencies) {
  mainElements.teamTemplateSelect.addEventListener('change', () => {
    renderTemplateSummary();
  });

  mainElements.openStatsBtn.addEventListener('click', () => {
    void dependencies.onOpenStats();
  });
  mainElements.pauseOneHourBtn.addEventListener('click', () => {
    void dependencies.onPauseOneHour();
  });
  mainElements.pauseTodayBtn.addEventListener('click', () => {
    void dependencies.onPauseToday();
  });
  mainElements.resumeBtn.addEventListener('click', () => {
    void dependencies.onResume();
  });
  mainElements.startSchedulerBtn.addEventListener('click', () => {
    void dependencies.onStartScheduler();
  });
  mainElements.stopSchedulerBtn.addEventListener('click', () => {
    dependencies.onStopScheduler();
  });
  mainElements.applyTemplateBtn.addEventListener('click', () => {
    void dependencies.onApplyTemplate();
  });
  mainElements.feedbackTooManyBtn.addEventListener('click', () => {
    void dependencies.onFeedbackTooMany();
  });
  mainElements.feedbackTooFewBtn.addEventListener('click', () => {
    void dependencies.onFeedbackTooFew();
  });
  mainElements.generateExportBtn.addEventListener('click', () => {
    void dependencies.onGenerateExport();
  });
  mainElements.copyExportSummaryBtn.addEventListener('click', () => {
    void dependencies.onCopyExportSummary();
  });
  mainElements.copyExportJsonBtn.addEventListener('click', () => {
    void dependencies.onCopyExportJson();
  });
  mainElements.importConfigBtn.addEventListener('click', () => {
    mainElements.importConfigFileInput.click();
  });
  mainElements.importConfigFileInput.addEventListener('change', () => {
    void (async () => {
      const [file] = Array.from(mainElements.importConfigFileInput.files ?? []);

      if (!file) {
        return;
      }

      try {
        await dependencies.onImportConfigFile(file);
      } finally {
        mainElements.importConfigFileInput.value = '';
      }
    })();
  });
  mainElements.downloadExportJsonBtn.addEventListener('click', () => {
    void dependencies.onDownloadExportJson();
  });
}
