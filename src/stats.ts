import { invoke } from '@tauri-apps/api/core';
import type { Achievement, DashboardState, DayStats, FeedbackRecord, StreakStats } from './shared/types';
import { createWordDetailModal } from './shared/word-detail-modal';
import { applyThemePreference, getThemeLabel } from './shared/theme';

interface WrongBookWord {
  card_id: number;
  word_id: number;
  word: string;
  phonetic: string | null;
  part_of_speech: string | null;
  meaning_zh: string;
  lifetime_wrong: number;
  lifetime_correct: number;
  last_result: string | null;
}

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
const metricCurrentStreak = document.querySelector('#metricCurrentStreak') as HTMLElement;
const metricLongestStreak = document.querySelector('#metricLongestStreak') as HTMLElement;

const recommendationText = document.querySelector('#recommendationText') as HTMLElement;
const recommendationMode = document.querySelector('#recommendationMode') as HTMLElement;
const recommendationReason = document.querySelector('#recommendationReason') as HTMLElement;

const pauseSummary = document.querySelector('#pauseSummary') as HTMLElement;
const scheduleSummary = document.querySelector('#scheduleSummary') as HTMLElement;
const learningSummary = document.querySelector('#learningSummary') as HTMLElement;
const systemSummary = document.querySelector('#systemSummary') as HTMLElement;
const feedbackList = document.querySelector('#feedbackList') as HTMLElement;
const historyChartContainer = document.querySelector('#historyChartContainer') as HTMLElement;
const heatmapContainer = document.querySelector('#heatmapContainer') as HTMLElement;
const achievementsContent = document.querySelector('#achievementsContent') as HTMLElement;
const achievementsProgress = document.querySelector('#achievementsProgress') as HTMLElement;
const range7Btn = document.querySelector('#range7Btn') as HTMLButtonElement;
const range30Btn = document.querySelector('#range30Btn') as HTMLButtonElement;

const refreshBtn = document.querySelector('#refreshBtn') as HTMLButtonElement;
const openMainBtn = document.querySelector('#openMainBtn') as HTMLButtonElement;

let selectedHistoryRange = 7;
const HEATMAP_DAYS = 90;
const HEATMAP_COLUMNS = 13;
const HEATMAP_ROWS = 7;
const HEATMAP_DAY_LABELS = ['一', '二', '三', '四', '五', '六', '日'];
const HEATMAP_MONTH_LABELS = ['1月', '2月', '3月', '4月', '5月', '6月', '7月', '8月', '9月', '10月', '11月', '12月'];
const wordDetailModal = createWordDetailModal({
  onWrongBookChange: async () => {
    await loadWrongBook();
  },
  onError: (message) => {
    console.error('单词详情操作失败:', message);
  },
});

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function formatShortDate(value: string): string {
  const date = new Date(`${value}T00:00:00`);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return `${date.getMonth() + 1}/${date.getDate()}`;
}

function getDateKey(date: Date): string {
  const year = date.getFullYear();
  const month = `${date.getMonth() + 1}`.padStart(2, '0');
  const day = `${date.getDate()}`.padStart(2, '0');
  return `${year}-${month}-${day}`;
}

function getRangeDates(days: number): string[] {
  const dates: string[] = [];
  const today = new Date();
  today.setHours(0, 0, 0, 0);

  for (let offset = days - 1; offset >= 0; offset -= 1) {
    const date = new Date(today);
    date.setDate(today.getDate() - offset);
    dates.push(getDateKey(date));
  }

  return dates;
}

function parseDateKey(value: string): Date | null {
  const date = new Date(`${value}T00:00:00`);
  if (Number.isNaN(date.getTime())) {
    return null;
  }
  date.setHours(0, 0, 0, 0);
  return date;
}

function normalizeHistoryStats(stats: DayStats[], days: number): DayStats[] {
  const statsByDate = new Map(stats.map((item) => [item.date, item]));
  return getRangeDates(days).map((date) => {
    const item = statsByDate.get(date);
    return {
      date,
      total_reviews: item?.total_reviews ?? 0,
      correct_count: item?.correct_count ?? 0,
      new_words: item?.new_words ?? 0,
    };
  });
}

function setRangeButtons(days: number) {
  range7Btn.classList.toggle('is-active', days === 7);
  range30Btn.classList.toggle('is-active', days === 30);
}

function getThemeColor(name: string, fallback: string): string {
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return value || fallback;
}

function renderHistoryEmpty(message: string) {
  historyChartContainer.innerHTML = `<div class="chart-empty">${escapeHtml(message)}</div>`;
}

function renderHeatmapEmpty(message: string) {
  heatmapContainer.innerHTML = `<div class="chart-empty">${escapeHtml(message)}</div>`;
}

function renderHistoryChart(stats: DayStats[]) {
  if (!stats.some((item) => item.total_reviews > 0)) {
    renderHistoryEmpty('所选时间范围内还没有复习记录，开始一次学习后这里会出现趋势变化。');
    return;
  }

  const width = 920;
  const height = 320;
  const paddingTop = 24;
  const paddingRight = 28;
  const paddingBottom = 54;
  const paddingLeft = 40;
  const plotWidth = width - paddingLeft - paddingRight;
  const plotHeight = height - paddingTop - paddingBottom;
  const maxReviews = Math.max(...stats.map((item) => item.total_reviews), 1);
  const stepX = stats.length > 1 ? plotWidth / (stats.length - 1) : plotWidth;
  const barSlotWidth = plotWidth / stats.length;
  const barWidth = Math.max(10, Math.min(26, barSlotWidth * 0.56));

  const gridLines = 4;
  const ratePoints: string[] = [];
  const bars: string[] = [];
  const labels: string[] = [];
  const dots: string[] = [];
  const grid: string[] = [];
  const gridColor = getThemeColor('--chart-grid', 'rgba(38, 65, 53, 0.12)');
  const axisColor = getThemeColor('--chart-axis', 'rgba(38, 65, 53, 0.22)');
  const labelColor = getThemeColor('--chart-label', '#667169');
  const captionColor = getThemeColor('--chart-caption', '#94674c');
  const chartLineColor = getThemeColor('--chart-line', '#c46d2d');
  const chartBarStart = getThemeColor('--chart-bar-start', '#2f5d4a');
  const chartBarEnd = getThemeColor('--chart-bar-end', '#81b997');

  for (let index = 0; index <= gridLines; index += 1) {
    const ratio = index / gridLines;
    const y = paddingTop + plotHeight - ratio * plotHeight;
    const value = Math.round(maxReviews * ratio);
    grid.push(
      `<line x1="${paddingLeft}" y1="${y}" x2="${width - paddingRight}" y2="${y}" stroke="${gridColor}" stroke-width="1" />`,
      `<text x="${paddingLeft - 8}" y="${y + 4}" text-anchor="end" font-size="11" fill="${labelColor}">${value}</text>`,
    );
  }

  stats.forEach((item, index) => {
    const centerX = paddingLeft + barSlotWidth * index + barSlotWidth / 2;
    const x = centerX - barWidth / 2;
    const barHeight = (item.total_reviews / maxReviews) * plotHeight;
    const y = paddingTop + plotHeight - barHeight;
    const rate = item.total_reviews > 0 ? item.correct_count / item.total_reviews : 0;
    const pointX = stats.length > 1 ? paddingLeft + stepX * index : paddingLeft + plotWidth / 2;
    const pointY = paddingTop + plotHeight - rate * plotHeight;
    const tooltip = `${item.date} 复习 ${item.total_reviews} 次，正确率 ${(rate * 100).toFixed(0)}%，新词 ${item.new_words}`;

    bars.push(
      `<rect x="${x}" y="${y}" width="${barWidth}" height="${Math.max(barHeight, 2)}" rx="8" fill="url(#reviewBarGradient)">
        <title>${escapeHtml(tooltip)}</title>
      </rect>`,
    );

    ratePoints.push(`${pointX},${pointY}`);
    dots.push(
      `<circle cx="${pointX}" cy="${pointY}" r="3.5" fill="${chartLineColor}">
        <title>${escapeHtml(tooltip)}</title>
      </circle>`,
    );

    labels.push(
      `<text x="${centerX}" y="${height - 18}" text-anchor="middle" font-size="11" fill="${labelColor}">${formatShortDate(item.date)}</text>`,
    );
  });

  const linePath = ratePoints.join(' ');

  historyChartContainer.innerHTML = `
    <svg viewBox="0 0 ${width} ${height}" role="img" aria-label="每日复习次数柱状图和正确率折线图">
      <defs>
        <linearGradient id="reviewBarGradient" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stop-color="${chartBarStart}" />
          <stop offset="100%" stop-color="${chartBarEnd}" />
        </linearGradient>
      </defs>
      ${grid.join('')}
      <line x1="${paddingLeft}" y1="${paddingTop + plotHeight}" x2="${width - paddingRight}" y2="${paddingTop + plotHeight}" stroke="${axisColor}" stroke-width="1" />
      ${bars.join('')}
      <polyline fill="none" stroke="${chartLineColor}" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" points="${linePath}" />
      ${dots.join('')}
      ${labels.join('')}
      <text x="${width - paddingRight}" y="${paddingTop - 4}" text-anchor="end" font-size="11" fill="${captionColor}">正确率</text>
    </svg>
  `;
}

function getHeatmapLevel(totalReviews: number): 0 | 1 | 2 | 3 {
  if (totalReviews <= 0) {
    return 0;
  }
  if (totalReviews <= 5) {
    return 1;
  }
  if (totalReviews <= 15) {
    return 2;
  }
  return 3;
}

function getHeatmapColor(totalReviews: number): string {
  const level = getHeatmapLevel(totalReviews);
  if (level === 0) {
    return getThemeColor('--heatmap-empty', 'rgba(53, 95, 76, 0.1)');
  }
  if (level === 1) {
    return getThemeColor('--heatmap-light', '#d6eadc');
  }
  if (level === 2) {
    return getThemeColor('--heatmap-medium', '#86b79a');
  }
  return getThemeColor('--heatmap-dark', '#355f4c');
}

function getMondayIndex(date: Date): number {
  const weekDay = date.getDay();
  return weekDay === 0 ? 6 : weekDay - 1;
}

function renderLearningHeatmap(stats: DayStats[]) {
  const normalizedStats = normalizeHistoryStats(stats, HEATMAP_DAYS);
  const statsByDate = new Map(normalizedStats.map((item) => [item.date, item]));
  const startDate = parseDateKey(normalizedStats[0]?.date ?? '');
  const endDate = parseDateKey(normalizedStats[normalizedStats.length - 1]?.date ?? '');

  if (!startDate || !endDate) {
    renderHeatmapEmpty('过去 90 天的学习记录读取失败，请稍后刷新重试。');
    return;
  }

  const firstGridDate = new Date(endDate);
  firstGridDate.setDate(endDate.getDate() - HEATMAP_COLUMNS * HEATMAP_ROWS + 1);
  firstGridDate.setHours(0, 0, 0, 0);

  const cellSize = 18;
  const cellGap = 5;
  const labelWidth = 32;
  const monthLabelHeight = 24;
  const topPadding = 16;
  const rightPadding = 12;
  const bottomPadding = 12;
  const gridWidth = HEATMAP_COLUMNS * cellSize + (HEATMAP_COLUMNS - 1) * cellGap;
  const gridHeight = HEATMAP_ROWS * cellSize + (HEATMAP_ROWS - 1) * cellGap;
  const width = labelWidth + gridWidth + rightPadding;
  const height = topPadding + monthLabelHeight + gridHeight + bottomPadding;
  const labelColor = getThemeColor('--heatmap-label', '#667169');
  const borderColor = getThemeColor('--border-soft', 'rgba(38, 65, 53, 0.08)');
  const monthLabels: string[] = [];
  const dayLabels: string[] = [];
  const cells: string[] = [];

  for (let row = 0; row < HEATMAP_ROWS; row += 1) {
    const y = topPadding + monthLabelHeight + row * (cellSize + cellGap) + cellSize / 2 + 4;
    dayLabels.push(
      `<text x="${labelWidth - 8}" y="${y}" text-anchor="end" font-size="12" fill="${labelColor}">${HEATMAP_DAY_LABELS[row]}</text>`,
    );
  }

  for (let column = 0; column < HEATMAP_COLUMNS; column += 1) {
    const columnDate = new Date(firstGridDate);
    columnDate.setDate(firstGridDate.getDate() + column * HEATMAP_ROWS);

    const previousColumnDate = new Date(firstGridDate);
    previousColumnDate.setDate(firstGridDate.getDate() + (column - 1) * HEATMAP_ROWS);

    if (column === 0 || columnDate.getMonth() !== previousColumnDate.getMonth()) {
      const x = labelWidth + column * (cellSize + cellGap);
      monthLabels.push(
        `<text x="${x}" y="${topPadding + 12}" font-size="12" fill="${labelColor}">${HEATMAP_MONTH_LABELS[columnDate.getMonth()]}</text>`,
      );
    }
  }

  for (let index = 0; index < HEATMAP_COLUMNS * HEATMAP_ROWS; index += 1) {
    const date = new Date(firstGridDate);
    date.setDate(firstGridDate.getDate() + index);

    const column = Math.floor(index / HEATMAP_ROWS);
    const row = getMondayIndex(date);
    const dateKey = getDateKey(date);

    if (date < startDate || date > endDate) {
      continue;
    }

    const item = statsByDate.get(dateKey) ?? {
      date: dateKey,
      total_reviews: 0,
      correct_count: 0,
      new_words: 0,
    };
    const x = labelWidth + column * (cellSize + cellGap);
    const y = topPadding + monthLabelHeight + row * (cellSize + cellGap);

    cells.push(
      `<rect x="${x}" y="${y}" width="${cellSize}" height="${cellSize}" rx="5" fill="${getHeatmapColor(item.total_reviews)}" stroke="${borderColor}" stroke-width="1">
        <title>${escapeHtml(`${item.date} · ${item.total_reviews} 次复习`)}</title>
      </rect>`,
    );
  }

  heatmapContainer.innerHTML = `
    <svg viewBox="0 0 ${width} ${height}" role="img" aria-label="过去 90 天学习活跃热力图">
      ${monthLabels.join('')}
      ${dayLabels.join('')}
      ${cells.join('')}
    </svg>
  `;
}

async function loadHistoryStats(days: number) {
  selectedHistoryRange = days;
  setRangeButtons(days);

  try {
    const history = await invoke<DayStats[]>('get_history_stats', { days });
    renderHistoryChart(normalizeHistoryStats(history, days));
  } catch (error) {
    console.error('加载历史趋势失败:', error);
    renderHistoryEmpty('学习历史趋势读取失败，请稍后刷新重试。');
  }
}
const wrongBookContent = document.querySelector('#wrongBookContent') as HTMLElement;
const wrongBookCount = document.querySelector('#wrongBookCount') as HTMLElement;

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

function renderAchievements(achievements: Achievement[]) {
  const unlockedCount = achievements.filter((achievement) => achievement.unlocked).length;
  achievementsProgress.textContent = `${unlockedCount} / ${achievements.length} 已解锁`;

  achievementsContent.innerHTML = `<div class="achievements-grid">${achievements
    .map((achievement) => {
      const unlockLabel = achievement.unlocked
        ? `解锁于 ${formatDateTime(achievement.unlocked_at)}`
        : '尚未解锁';

      return `
        <article class="achievement-card${achievement.unlocked ? ' is-unlocked' : ''}">
          <div class="achievement-icon" aria-hidden="true">${achievement.unlocked ? '★' : '○'}</div>
          <div>
            <h3>${escapeHtml(achievement.title)}</h3>
            <p>${escapeHtml(achievement.description)}</p>
            <span class="achievement-meta">${escapeHtml(unlockLabel)}</span>
          </div>
        </article>
      `;
    })
    .join('')}</div>`;
}

function renderDashboard(state: DashboardState) {
  const { today_stats: stats, app_config: config, recommendation, pause_until } = state;
  applyThemePreference(config.system.theme);

  metricTotalReviews.textContent = String(stats.total_reviews);
  metricAccuracy.textContent = `${stats.accuracy.toFixed(0)}%`;
  metricNewWords.textContent = String(stats.new_words_today);
  metricDueCards.textContent = String(stats.due_cards_count);
  metricKnowCount.textContent = String(stats.know_count);
  metricDontKnowCount.textContent = String(stats.dont_know_count);
  metricSkipCount.textContent = String(stats.skip_count);
  metricMasteredCount.textContent = String(stats.mastered_count);
  metricCurrentStreak.textContent = String(state.current_streak);

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
  systemSummary.textContent = `${config.system.start_behavior === 'show-main' ? '启动显示主页面' : '启动最小化到托盘'}，托盘${config.system.tray_enabled ? '开启' : '关闭'}，开机启动${config.system.launch_at_login ? '开启' : '关闭'}，主题${getThemeLabel(config.system.theme)}`;

  renderFeedback(state.recent_feedback);
}

function renderStreakStats(stats: StreakStats) {
  metricCurrentStreak.textContent = String(stats.current_streak);
  metricLongestStreak.textContent = String(stats.longest_streak);
}

async function loadStats() {
  try {
    const [dashboard, history, heatmapHistory, streak, achievements] = await Promise.all([
      invoke<DashboardState>('get_dashboard_state'),
      invoke<DayStats[]>('get_history_stats', { days: selectedHistoryRange }),
      invoke<DayStats[]>('get_history_stats', { days: HEATMAP_DAYS }),
      invoke<StreakStats>('get_streak_stats'),
      invoke<Achievement[]>('get_achievements'),
    ]);
    renderDashboard(dashboard);
    renderStreakStats(streak);
    renderHistoryChart(normalizeHistoryStats(history, selectedHistoryRange));
    renderLearningHeatmap(heatmapHistory);
    renderAchievements(achievements);
  } catch (error) {
    console.error('加载统计页失败:', error);
    heroSummary.textContent = '统计页读取失败，请稍后重试或返回主页面查看当前状态。';
    statusChip.textContent = '读取失败';
    recommendationChip.textContent = '请重试';
    metricCurrentStreak.textContent = '0';
    metricLongestStreak.textContent = '0';
    achievementsProgress.textContent = '读取失败';
    achievementsContent.innerHTML = '<p class="panel-body">成就读取失败，请稍后刷新重试。</p>';
    renderHistoryEmpty('学习历史趋势读取失败，请稍后刷新重试。');
    renderHeatmapEmpty('过去 90 天的学习记录读取失败，请稍后刷新重试。');
  }
}

window.addEventListener('DOMContentLoaded', () => {
  void loadStats();
  void loadWrongBook();
  window.setInterval(() => {
    void loadStats();
  }, 30000);
});

refreshBtn.addEventListener('click', () => {
  void loadStats();
});

range7Btn.addEventListener('click', () => {
  void loadHistoryStats(7);
});

range30Btn.addEventListener('click', () => {
  void loadHistoryStats(30);
});

openMainBtn.addEventListener('click', async () => {
  await invoke('show_main_window');
});

// Wrong Book
async function loadWrongBook() {
  try {
    const words = await invoke<WrongBookWord[]>('get_wrong_book_words');
    wrongBookCount.textContent = `${words.length} 词`;
    renderWrongBook(words);
  } catch (error) {
    console.error('加载错题本失败:', error);
    wrongBookContent.innerHTML = '<p class="wrong-book-empty">加载错题本失败</p>';
  }
}

function renderWrongBook(words: WrongBookWord[]) {
  if (!words.length) {
    wrongBookContent.innerHTML = '<p class="wrong-book-empty">还没有错题记录，继续学习吧！</p>';
    return;
  }

  wrongBookContent.innerHTML = `<div class="wrong-book-list">${words
    .map(
      (w) => `
    <div class="wrong-book-item" data-card-id="${w.card_id}" data-word-id="${w.word_id}" role="button" tabindex="0" aria-label="查看 ${escapeHtml(w.word)} 详情">
      <div>
        <div class="wrong-book-word">${escapeHtml(w.word)}${w.phonetic ? ` <span class="wrong-book-phonetic">${escapeHtml(w.phonetic)}</span>` : ''}</div>
        <div class="wrong-book-meaning">${w.part_of_speech ? `${escapeHtml(w.part_of_speech)} ` : ''}${escapeHtml(w.meaning_zh)}</div>
        <div class="wrong-book-meta">错 ${w.lifetime_wrong} 次 · 对 ${w.lifetime_correct} 次</div>
      </div>
      <button class="wrong-book-remove" type="button" data-remove-card-id="${w.card_id}">已掌握</button>
    </div>`
    )
    .join('')}</div>`;
}

wrongBookContent.addEventListener('click', (event) => {
  const target = event.target as HTMLElement | null;
  const removeButton = target?.closest<HTMLButtonElement>('button[data-remove-card-id]');

  if (removeButton) {
    const cardId = Number(removeButton.dataset.removeCardId);
    if (!Number.isFinite(cardId)) {
      return;
    }

    void (async () => {
      try {
        await invoke('remove_from_wrong_book', { cardId });
        await loadWrongBook();
      } catch (error) {
        console.error('移除错题失败:', error);
      }
    })();
    return;
  }

  const item = target?.closest<HTMLElement>('.wrong-book-item[data-word-id]');
  if (!item) {
    return;
  }

  const wordId = Number(item.dataset.wordId);
  if (!Number.isFinite(wordId)) {
    return;
  }

  void wordDetailModal.open(wordId);
});

wrongBookContent.addEventListener('keydown', (event) => {
  if (event.key !== 'Enter' && event.key !== ' ') {
    return;
  }

  const target = event.target as HTMLElement | null;
  const item = target?.closest<HTMLElement>('.wrong-book-item[data-word-id]');
  if (!item || target?.closest('button[data-remove-card-id]')) {
    return;
  }

  event.preventDefault();
  const wordId = Number(item.dataset.wordId);
  if (!Number.isFinite(wordId)) {
    return;
  }

  void wordDetailModal.open(wordId);
});
