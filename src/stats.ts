import { invoke } from '@tauri-apps/api/core';
import type { DashboardState, FeedbackRecord } from './shared/types';

const heroSummary = document.querySelector('#heroSummary') as HTMLElement;
const statusChip = document.querySelector('#statusChip') as HTMLElement;
const recommendationChip = document.querySelector('#recommendationChip') as HTMLElement;

const metricTotalReviews = document.querySelector('#metricTotalReviews') as HTMLElement;
const metricAccuracy = document.querySelector('#metricAccuracy') as HTMLElement;
const metricNewWords = document.querySelector('#metricNewWords') as HTMLElement;
const metricDueCards = document.querySelector('#metricDueCards') as HTMLElement;
const metricKnowCount = document.querySelector('#metricKnowCount') as HTMLElement;
const metricDontKnowCount = document.querySelector('#metricDontKnowCount') as HTMLElement;
const metricSkipCount = document.querySelector('#metricSkipCount') as HTMLElement;
const metricMasteredCount = document.querySelector('#metricMasteredCount') as HTMLElement;

const recommendationText = document.querySelector('#recommendationText') as HTMLElement;
const recommendationMode = document.querySelector('#recommendationMode') as HTMLElement;
const recommendationReason = document.querySelector('#recommendationReason') as HTMLElement;

const pauseSummary = document.querySelector('#pauseSummary') as HTMLElement;
const scheduleSummary = document.querySelector('#scheduleSummary') as HTMLElement;
const learningSummary = document.querySelector('#learningSummary') as HTMLElement;
const systemSummary = document.querySelector('#systemSummary') as HTMLElement;
const feedbackList = document.querySelector('#feedbackList') as HTMLElement;

const refreshBtn = document.querySelector('#refreshBtn') as HTMLButtonElement;
const openMainBtn = document.querySelector('#openMainBtn') as HTMLButtonElement;

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

function getFeedbackLabel(record: FeedbackRecord): string {
  if (record.feedback_type === 'too_many_reminders') {
    return '提醒太多';
  }

  if (record.feedback_type === 'too_few_reminders') {
    return '提醒太少';
  }

  return `这张词先别再推${record.word ? ` · ${record.word}` : ''}`;
}

function renderFeedback(records: FeedbackRecord[] = []) {
  if (!records.length) {
    feedbackList.innerHTML = '<li>最近还没有反馈记录。</li>';
    return;
  }

  feedbackList.innerHTML = records
    .map((record) => {
      const source = record.source === 'card' ? '浮卡' : '主页面';
      return `<li>${getFeedbackLabel(record)} · ${formatDateTime(record.created_at)} · 来自${source}</li>`;
    })
    .join('');
}

function renderDashboard(state: DashboardState) {
  const { today_stats: stats, app_config: config, recommendation, pause_until } = state;

  metricTotalReviews.textContent = String(stats.total_reviews);
  metricAccuracy.textContent = `${stats.accuracy.toFixed(0)}%`;
  metricNewWords.textContent = String(stats.new_words_today);
  metricDueCards.textContent = String(stats.due_cards_count);
  metricKnowCount.textContent = String(stats.know_count);
  metricDontKnowCount.textContent = String(stats.dont_know_count);
  metricSkipCount.textContent = String(stats.skip_count);
  metricMasteredCount.textContent = String(stats.mastered_count);

  statusChip.textContent = pause_until ? `已暂停至 ${formatDateTime(pause_until)}` : '运行中';
  recommendationChip.textContent = recommendation.source === 'adaptive' ? '动态推荐' : '默认推荐';
  heroSummary.textContent = pause_until
    ? '当前处于暂停状态，统计仍会刷新，但不会触发新的浮卡提醒。'
    : '这个页面用于快速查看今日学习表现、当前推荐和最近的使用信号。';

  recommendationText.textContent = recommendation.explanation;
  recommendationMode.textContent = `建议模式：${recommendation.suggested_mode}`;
  recommendationReason.textContent = recommendation.reasons[0] ?? '系统会在这里解释当前建议。';

  pauseSummary.textContent = pause_until ? `暂停到 ${formatDateTime(pause_until)}` : '当前未暂停';
  scheduleSummary.textContent = `静默时间 ${config.schedule.quiet_hours_start} - ${config.schedule.quiet_hours_end}，工作日 ${config.schedule.weekday_profile ?? 'gentle'} / 周末 ${config.schedule.weekend_profile ?? 'balanced'}`;
  learningSummary.textContent = `每日新词 ${config.learning.daily_new_limit}，${config.learning.review_first ? '优先复习词' : '允许新词优先'}`;
  systemSummary.textContent = `${config.system.start_behavior === 'show-main' ? '启动显示主页面' : '启动最小化到托盘'}，托盘${config.system.tray_enabled ? '开启' : '关闭'}，开机启动${config.system.launch_at_login ? '开启' : '关闭'}`;

  renderFeedback(state.recent_feedback);
}

async function loadStats() {
  try {
    const dashboard = await invoke<DashboardState>('get_dashboard_state');
    renderDashboard(dashboard);
  } catch (error) {
    console.error('加载统计页失败:', error);
    heroSummary.textContent = '统计页读取失败，请稍后重试或返回主页面查看当前状态。';
    statusChip.textContent = '读取失败';
    recommendationChip.textContent = '请重试';
  }
}

window.addEventListener('DOMContentLoaded', () => {
  void loadStats();
  window.setInterval(() => {
    void loadStats();
  }, 30000);
});

refreshBtn.addEventListener('click', () => {
  void loadStats();
});

openMainBtn.addEventListener('click', async () => {
  await invoke('show_main_window');
});
