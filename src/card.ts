import { invoke } from '@tauri-apps/api/core';
import { emit, listen } from '@tauri-apps/api/event';
import type { AppConfig, WordCardData } from './shared/types';

let currentCard: WordCardData | null = null;
let currentConfig: AppConfig | null = null;
let autoHideTimer: number | null = null;
let closeDelayTimer: number | null = null;
let isLoadingCard = false;
let isSubmitting = false;
let hasAnswered = false;

const modeBadgeEl = document.getElementById('modeBadge') as HTMLElement;
const promptEl = document.getElementById('prompt') as HTMLElement;
const promptHintEl = document.getElementById('promptHint') as HTMLElement;
const optionsEl = document.getElementById('options') as HTMLElement;
const answerPanelEl = document.getElementById('answerPanel') as HTMLElement;
const answerTitleEl = document.getElementById('answerTitle') as HTMLElement;
const answerDetailEl = document.getElementById('answerDetail') as HTMLElement;
const feedbackStatusEl = document.getElementById('feedbackStatus') as HTMLElement;

const btnSkip = document.getElementById('btnSkip') as HTMLButtonElement;
const btnNotInterested = document.getElementById('btnNotInterested') as HTMLButtonElement;
const btnContinue = document.getElementById('btnContinue') as HTMLButtonElement;
const btnSettings = document.getElementById('btnSettings') as HTMLButtonElement;

function clearTimer(timer: number | null) {
  if (timer !== null) {
    window.clearTimeout(timer);
  }
}

function clearAllTimers() {
  clearTimer(autoHideTimer);
  clearTimer(closeDelayTimer);
  autoHideTimer = null;
  closeDelayTimer = null;
}

function setFeedbackStatus(message: string) {
  feedbackStatusEl.textContent = message;
}

function isSkipAllowed(): boolean {
  return currentConfig?.card.allow_skip ?? true;
}

function getOptionButtons(): HTMLButtonElement[] {
  return Array.from(optionsEl.querySelectorAll<HTMLButtonElement>('.option-btn'));
}

function setOptionButtonsDisabled(disabled: boolean) {
  getOptionButtons().forEach((button) => {
    button.disabled = disabled;
  });
}

function syncActionButtons() {
  btnSkip.hidden = !isSkipAllowed();
  btnSkip.disabled = isSubmitting || hasAnswered || !isSkipAllowed();
  btnNotInterested.disabled = isSubmitting || hasAnswered;
  btnContinue.disabled = isSubmitting;
  btnSettings.disabled = isSubmitting;
}

function getModeLabel(card: WordCardData): string {
  return card.quiz_mode === 'zh_to_en_choice' ? '根据中文选英文' : '根据英文选中文';
}

function resetCardState() {
  clearAllTimers();
  currentCard = null;
  isSubmitting = false;
  hasAnswered = false;

  modeBadgeEl.textContent = '等待提醒';
  promptEl.textContent = '';
  promptHintEl.textContent = '';
  promptHintEl.hidden = true;
  optionsEl.innerHTML = '';
  answerPanelEl.hidden = true;
  answerTitleEl.textContent = '';
  answerDetailEl.textContent = '';
  btnContinue.hidden = true;
  setFeedbackStatus('');
  syncActionButtons();
}

async function delay(ms: number) {
  await new Promise((resolve) => window.setTimeout(resolve, ms));
}

async function hideCardWindow() {
  clearAllTimers();
  resetCardState();
  await invoke('hide_card_window');
  await emit('card-hidden');
}

async function showMainWindowFromCard() {
  clearAllTimers();
  resetCardState();
  await invoke('show_main_window');
}

function showDailyGoalComplete(currentLimit: number) {
  clearAllTimers();
  modeBadgeEl.textContent = '今日目标达成';
  promptEl.textContent = `🎉 恭喜完成今日任务！`;
  promptHintEl.textContent = `已学习 ${currentLimit} 个新词`;
  promptHintEl.hidden = false;
  optionsEl.innerHTML = '';
  answerPanelEl.hidden = true;
  btnContinue.hidden = true;
  btnSkip.hidden = true;
  btnNotInterested.hidden = true;
  setFeedbackStatus('');

  const increaseBtn = document.createElement('button');
  increaseBtn.className = 'option-btn';
  increaseBtn.textContent = '调高目标继续挑战';
  increaseBtn.onclick = async () => {
    const newLimit = currentLimit + 10;
    const config = await invoke<AppConfig>('get_app_config');
    config.learning.daily_new_limit = newLimit;
    await invoke('update_app_config', { config });
    await hideCardWindow();
  };

  const finishBtn = document.createElement('button');
  finishBtn.className = 'option-btn';
  finishBtn.textContent = '今天就到这里';
  finishBtn.onclick = () => void hideCardWindow();

  optionsEl.appendChild(increaseBtn);
  optionsEl.appendChild(finishBtn);
}

function markSelectedOptions(selectedOptionId: string, correctOptionId: string) {
  getOptionButtons().forEach((button) => {
    const isSelected = button.dataset.optionId === selectedOptionId;
    const isCorrect = button.dataset.optionId === correctOptionId;

    button.classList.toggle('selected', isSelected);
    button.classList.toggle('correct', isCorrect);
    button.classList.toggle('wrong', isSelected && !isCorrect);
  });
}

function renderAnswerPanel(card: WordCardData) {
  answerTitleEl.textContent = card.explanation_title;
  answerDetailEl.textContent = card.explanation_detail;
}

async function persistReview(result: 'know' | 'dont_know' | 'skip'): Promise<boolean> {
  if (!currentCard) {
    setFeedbackStatus('卡片还在加载，请稍后再试。');
    return false;
  }

  if (isSubmitting) {
    return false;
  }

  try {
    isSubmitting = true;
    syncActionButtons();
    setOptionButtonsDisabled(true);
    await invoke('submit_review', {
      cardId: currentCard.card_id,
      result,
    });
    isSubmitting = false;
    syncActionButtons();
    return true;
  } catch (error) {
    console.error('提交复习结果失败:', error);
    isSubmitting = false;
    setOptionButtonsDisabled(false);
    syncActionButtons();
    setFeedbackStatus('提交失败，请稍后再试。');
    return false;
  }
}

function applyCardPreferences(config: AppConfig) {
  currentConfig = config;
  syncActionButtons();
}

function renderOptions(card: WordCardData) {
  optionsEl.innerHTML = '';

  card.options.forEach((option, index) => {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'option-btn';
    button.dataset.optionId = option.id;

    const indexEl = document.createElement('span');
    indexEl.className = 'option-index';
    indexEl.textContent = String(index + 1);
    button.appendChild(indexEl);

    const content = document.createElement('span');
    content.className = 'option-content';

    const label = document.createElement('span');
    label.className = 'option-label';
    label.textContent = option.label;
    content.appendChild(label);

    if (option.detail) {
      const detail = document.createElement('span');
      detail.className = 'option-detail';
      detail.textContent = option.detail;
      content.appendChild(detail);
    }

    button.appendChild(content);
    button.addEventListener('click', () => {
      void handleOptionSelect(option.id);
    });
    optionsEl.appendChild(button);
  });
}

function scheduleAutoHide(config: AppConfig) {
  const autoHideDelay = (config.card.auto_hide_sec * 1000) + Math.random() * 2000;
  autoHideTimer = window.setTimeout(() => {
    void handleSkip();
  }, autoHideDelay);
}

async function loadAndShowCard() {
  if (isLoadingCard) {
    return;
  }

  isLoadingCard = true;
  resetCardState();
  setOptionButtonsDisabled(true);
  syncActionButtons();

  try {
    const config = await invoke<AppConfig>('get_app_config');
    applyCardPreferences(config);

    const card = await invoke<WordCardData | null>('get_next_card');
    if (!card) {
      try {
        const stats = await invoke<any>('get_today_stats');
        if (stats.new_words_today >= config.learning.daily_new_limit && stats.due_cards_count === 0) {
          showDailyGoalComplete(config.learning.daily_new_limit);
          isLoadingCard = false;
          return;
        }
      } catch (error) {
        console.error('Failed to check daily stats:', error);
      }

      setFeedbackStatus('当前没有可学习卡片');
      await delay(700);
      await hideCardWindow();
      return;
    }

    currentCard = card;
    hasAnswered = false;
    modeBadgeEl.textContent = getModeLabel(card);
    promptEl.textContent = card.prompt;
    promptHintEl.textContent = card.prompt_hint ?? '';
    promptHintEl.hidden = !card.prompt_hint;
    answerPanelEl.hidden = true;
    btnContinue.hidden = true;
    renderOptions(card);
    renderAnswerPanel(card);
    setFeedbackStatus('选对才会计入掌握进度，选错会加入错题集。');
    setOptionButtonsDisabled(false);
    syncActionButtons();
    scheduleAutoHide(config);
  } catch (error) {
    console.error('加载卡片失败:', error);
    resetCardState();
    setFeedbackStatus('加载失败，稍后会再试');
    await delay(700);
    await hideCardWindow();
  } finally {
    isLoadingCard = false;
  }
}

async function handleOptionSelect(optionId: string) {
  if (!currentCard || hasAnswered || isSubmitting) {
    return;
  }

  clearAllTimers();
  hasAnswered = true;
  markSelectedOptions(optionId, currentCard.correct_option_id);
  setOptionButtonsDisabled(true);
  syncActionButtons();

  const isCorrect = optionId === currentCard.correct_option_id;

  if (isCorrect) {
    setFeedbackStatus('回答正确，已计入掌握进度。');
    const saved = await persistReview('know');
    if (!saved) {
      hasAnswered = false;
      setOptionButtonsDisabled(false);
      syncActionButtons();
      return;
    }

    closeDelayTimer = window.setTimeout(() => {
      void hideCardWindow();
    }, 800);
    return;
  }

  renderAnswerPanel(currentCard);
  answerPanelEl.hidden = false;
  setFeedbackStatus('回答错误，已加入错题集，之后会再次出现。');
  const saved = await persistReview('dont_know');
  if (!saved) {
    hasAnswered = false;
    answerPanelEl.hidden = true;
    setOptionButtonsDisabled(false);
    syncActionButtons();
    return;
  }

  btnContinue.hidden = false;
  syncActionButtons();

  closeDelayTimer = window.setTimeout(() => {
    void hideCardWindow();
  }, 5000);
}

async function handleSkip() {
  if (!isSkipAllowed()) {
    return;
  }

  const saved = await persistReview('skip');
  if (saved) {
    await hideCardWindow();
  }
}

async function handleNotInterested() {
  if (!currentCard || isSubmitting || hasAnswered) {
    return;
  }

  try {
    isSubmitting = true;
    syncActionButtons();
    setFeedbackStatus('已记录偏好，这张词会先冷却一段时间。');
    await invoke('record_feedback', {
      feedbackType: 'not_interested_word',
      source: 'card',
      cardId: currentCard.card_id,
      word: currentCard.word,
    });
    isSubmitting = false;
    syncActionButtons();
    await handleSkip();
  } catch (error) {
    console.error('记录卡片反馈失败:', error);
    isSubmitting = false;
    syncActionButtons();
    setFeedbackStatus('记录失败，请稍后再试。');
  }
}

function isCardWindowVisible(): boolean {
  return document.visibilityState === 'visible';
}

function ensureCardLoadedIfVisible() {
  if (!isCardWindowVisible()) {
    return;
  }

  if (currentCard || isLoadingCard || isSubmitting) {
    return;
  }

  void loadAndShowCard();
}

function chooseOptionByIndex(index: number) {
  const button = getOptionButtons()[index];
  if (button) {
    void handleOptionSelect(button.dataset.optionId || '');
  }
}

btnSkip.addEventListener('click', () => {
  void handleSkip();
});

btnNotInterested.addEventListener('click', () => {
  void handleNotInterested();
});

btnContinue.addEventListener('click', () => {
  void hideCardWindow();
});

btnSettings.addEventListener('click', () => {
  void showMainWindowFromCard();
});

document.addEventListener('keydown', (event) => {
  if (currentConfig && !currentConfig.card.shortcuts_enabled) {
    return;
  }

  if (event.key >= '1' && event.key <= '4') {
    event.preventDefault();
    chooseOptionByIndex(Number(event.key) - 1);
    return;
  }

  if (event.key === 'Escape') {
    if (!isSkipAllowed()) {
      return;
    }
    event.preventDefault();
    void handleSkip();
  }
});

listen('shortcut-option-1', () => {
  if (currentConfig && !currentConfig.card.shortcuts_enabled) {
    return;
  }
  chooseOptionByIndex(0);
});

listen('shortcut-option-2', () => {
  if (currentConfig && !currentConfig.card.shortcuts_enabled) {
    return;
  }
  chooseOptionByIndex(1);
});

listen('shortcut-option-3', () => {
  if (currentConfig && !currentConfig.card.shortcuts_enabled) {
    return;
  }
  chooseOptionByIndex(2);
});

listen('shortcut-option-4', () => {
  if (currentConfig && !currentConfig.card.shortcuts_enabled) {
    return;
  }
  chooseOptionByIndex(3);
});

listen('shortcut-skip', () => {
  if (currentConfig && !currentConfig.card.shortcuts_enabled) {
    return;
  }
  if (!isSkipAllowed()) {
    return;
  }
  void handleSkip();
});

listen('card-window-shown', () => {
  void loadAndShowCard();
});

listen('card-window-hidden', () => {
  resetCardState();
});

document.addEventListener('visibilitychange', () => {
  ensureCardLoadedIfVisible();
});

window.addEventListener('focus', () => {
  ensureCardLoadedIfVisible();
});

window.addEventListener('DOMContentLoaded', () => {
  resetCardState();
  ensureCardLoadedIfVisible();
});
