import { invoke } from '@tauri-apps/api/core';
import type { WordDetail } from './types';

interface WordDetailModalOptions {
  onWrongBookChange?: (detail: WordDetail) => Promise<void> | void;
  onError?: (error: string) => void;
}

interface WordDetailModalController {
  close: () => void;
  open: (wordId: number) => Promise<void>;
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function formatDateTime(value?: string): string {
  if (!value) {
    return '暂无';
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: 'numeric',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function getStatusLabel(status: WordDetail['srs_status']): string {
  if (status === 'learning') {
    return '学习中';
  }

  if (status === 'mastered') {
    return '已掌握';
  }

  return '新词';
}

function getStageLabel(stage: number): string {
  return stage >= 0 ? `Stage ${stage}` : '待开始';
}

function buildDetailMarkup(detail: WordDetail): string {
  const auxText = [detail.phonetic, detail.part_of_speech].filter(Boolean).join(' · ') || '未提供音标或词性';
  const actionLabel = detail.in_wrong_book ? '移出错题本' : '加入错题本';

  return `
    <div class="word-detail-modal-header">
      <div class="word-detail-modal-copy">
        <p class="word-detail-modal-kicker">Word Detail</p>
        <h2>${escapeHtml(detail.word)}</h2>
        <p class="word-detail-modal-aux">${escapeHtml(auxText)}</p>
      </div>
      <button class="word-detail-close" type="button" aria-label="关闭">×</button>
    </div>
    <div class="word-detail-modal-actions">
      <button class="ghost-btn word-detail-pronounce" type="button">朗读发音</button>
      <span class="word-detail-status-badge status-${escapeHtml(detail.srs_status)}">${escapeHtml(getStatusLabel(detail.srs_status))} · ${escapeHtml(getStageLabel(detail.srs_stage))}</span>
      <button class="primary-btn word-detail-wrong-book-toggle" type="button">${escapeHtml(actionLabel)}</button>
    </div>
    <section class="word-detail-panel">
      <p class="word-detail-section-label">中文释义</p>
      <p class="word-detail-meaning">${escapeHtml(detail.meaning_zh)}</p>
    </section>
    <section class="word-detail-panel">
      <p class="word-detail-section-label">例句</p>
      <p class="word-detail-example">${escapeHtml(detail.example_sentence || '暂无例句')}</p>
    </section>
    <section class="word-detail-grid">
      <article class="word-detail-metric">
        <span>来源</span>
        <strong>${escapeHtml(detail.source)}</strong>
      </article>
      <article class="word-detail-metric">
        <span>难度</span>
        <strong>${detail.difficulty}</strong>
      </article>
      <article class="word-detail-metric">
        <span>下次到期</span>
        <strong>${escapeHtml(formatDateTime(detail.due_at))}</strong>
      </article>
      <article class="word-detail-metric">
        <span>当前连对</span>
        <strong>${detail.correct_streak}</strong>
      </article>
      <article class="word-detail-metric">
        <span>累计答对</span>
        <strong>${detail.lifetime_correct}</strong>
      </article>
      <article class="word-detail-metric">
        <span>累计答错</span>
        <strong>${detail.lifetime_wrong}</strong>
      </article>
    </section>
  `;
}

function ensureStyles() {
  if (document.getElementById('word-detail-modal-styles')) {
    return;
  }

  const style = document.createElement('style');
  style.id = 'word-detail-modal-styles';
  style.textContent = `
    body.modal-open {
      overflow: hidden;
    }

    .word-detail-modal-overlay {
      position: fixed;
      inset: 0;
      display: grid;
      place-items: center;
      padding: 24px;
      background: var(--bg-overlay, rgba(16, 22, 18, 0.4));
      backdrop-filter: blur(16px);
      z-index: 1000;
    }

    .word-detail-modal-overlay.hidden {
      display: none;
    }

    .wordbook-preview-item[role="button"],
    .wrong-book-item[role="button"] {
      cursor: pointer;
    }

    .wordbook-preview-item[role="button"]:focus-visible,
    .wrong-book-item[role="button"]:focus-visible,
    .word-detail-close:focus-visible {
      outline: 2px solid var(--accent, #355f4c);
      outline-offset: 3px;
    }

    .word-detail-modal-card {
      width: min(720px, calc(100vw - 32px));
      max-height: min(82vh, 920px);
      overflow: auto;
      padding: 28px;
      border-radius: 28px;
      border: 1px solid var(--border-color, rgba(38, 65, 53, 0.12));
      background:
        linear-gradient(135deg, var(--bg-hero-start, rgba(255, 249, 239, 0.92)), var(--bg-hero-end, rgba(241, 248, 243, 0.86))),
        var(--bg-card-strong, rgba(255, 255, 255, 0.84));
      box-shadow: 0 30px 80px var(--shadow-modal, rgba(47, 61, 52, 0.22));
      backdrop-filter: blur(20px);
      color: var(--text-primary, #1e2a26);
    }

    .word-detail-modal-header,
    .word-detail-modal-actions {
      display: flex;
      align-items: flex-start;
      justify-content: space-between;
      gap: 14px;
    }

    .word-detail-modal-copy h2 {
      margin: 6px 0 0;
      font-size: clamp(30px, 4vw, 42px);
      line-height: 1.05;
      color: var(--text-strong, #243930);
    }

    .word-detail-modal-kicker,
    .word-detail-section-label,
    .word-detail-metric span {
      margin: 0;
      font-size: 12px;
      letter-spacing: 0.12em;
      text-transform: uppercase;
      color: var(--text-accent, #7a6b4c);
    }

    .word-detail-modal-aux {
      margin: 12px 0 0;
      color: var(--text-secondary, #5a665f);
    }

    .word-detail-close {
      width: 44px;
      min-width: 44px;
      padding: 0;
      border: 1px solid var(--border-color, rgba(38, 65, 53, 0.12));
      border-radius: 999px;
      background: var(--bg-card, rgba(255, 255, 255, 0.72));
      color: var(--text-primary, #1e2a26);
      font-size: 24px;
      line-height: 1;
      cursor: pointer;
    }

    .word-detail-modal-actions {
      flex-wrap: wrap;
      margin-top: 20px;
    }

    .word-detail-status-badge {
      display: inline-flex;
      align-items: center;
      min-height: 44px;
      padding: 0 16px;
      border-radius: 999px;
      border: 1px solid var(--border-color, rgba(38, 65, 53, 0.12));
      background: var(--bg-card, rgba(255, 255, 255, 0.72));
      color: var(--text-pill, #254337);
      font-weight: 600;
    }

    .word-detail-status-badge.status-learning {
      background: var(--accent-soft, rgba(53, 95, 76, 0.12));
      color: var(--accent-soft-text, #2f5d4b);
    }

    .word-detail-status-badge.status-mastered {
      background: var(--accent-gold-soft, rgba(124, 109, 90, 0.12));
      color: var(--accent-gold-text, #6a5f52);
    }

    .word-detail-panel,
    .word-detail-metric {
      border-radius: 20px;
      border: 1px solid var(--border-soft, rgba(38, 65, 53, 0.08));
      background: var(--bg-card-soft, var(--bg-card-muted, rgba(247, 243, 235, 0.88)));
      box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.12);
    }

    .word-detail-panel {
      margin-top: 16px;
      padding: 18px 20px;
    }

    .word-detail-meaning,
    .word-detail-example {
      margin: 10px 0 0;
      line-height: 1.7;
      color: var(--text-strong, #243930);
    }

    .word-detail-grid {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 14px;
      margin-top: 16px;
    }

    .word-detail-metric {
      padding: 18px;
    }

    .word-detail-metric strong {
      display: block;
      margin-top: 10px;
      font-size: 18px;
      line-height: 1.4;
      color: var(--text-strong, #243930);
      word-break: break-word;
    }

    .word-detail-loading,
    .word-detail-error {
      margin: 0;
      color: var(--text-secondary, #5a665f);
      line-height: 1.6;
    }

    .word-detail-error {
      color: var(--accent-orange-text, #8b4f13);
    }

    @media (max-width: 760px) {
      .word-detail-modal-card {
        padding: 22px;
      }

      .word-detail-modal-header {
        align-items: center;
      }

      .word-detail-grid {
        grid-template-columns: 1fr;
      }
    }
  `;

  document.head.appendChild(style);
}

function ensureOverlay(): HTMLDivElement {
  const existing = document.getElementById('word-detail-modal-overlay');
  if (existing instanceof HTMLDivElement) {
    return existing;
  }

  const overlay = document.createElement('div');
  overlay.id = 'word-detail-modal-overlay';
  overlay.className = 'word-detail-modal-overlay hidden';
  overlay.innerHTML = '<div class="word-detail-modal-card" role="dialog" aria-modal="true" aria-label="单词详情"></div>';
  document.body.appendChild(overlay);
  return overlay;
}

export function createWordDetailModal(options: WordDetailModalOptions = {}): WordDetailModalController {
  ensureStyles();

  const overlay = ensureOverlay();
  const card = overlay.querySelector('.word-detail-modal-card');

  if (!(card instanceof HTMLDivElement)) {
    throw new Error('Missing word detail modal card');
  }

  let activeWordId: number | null = null;
  let currentDetail: WordDetail | null = null;
  let requestId = 0;

  const close = () => {
    activeWordId = null;
    overlay.classList.add('hidden');
    document.body.classList.remove('modal-open');
  };

  const show = () => {
    overlay.classList.remove('hidden');
    document.body.classList.add('modal-open');
  };

  const renderLoading = () => {
    card.innerHTML = '<p class="word-detail-loading">正在读取单词详情...</p>';
  };

  const renderError = (message: string) => {
    card.innerHTML = `
      <div class="word-detail-modal-header">
        <div class="word-detail-modal-copy">
          <p class="word-detail-modal-kicker">Word Detail</p>
          <h2>读取失败</h2>
          <p class="word-detail-error">${escapeHtml(message)}</p>
        </div>
        <button class="word-detail-close" type="button" aria-label="关闭">×</button>
      </div>
    `;
  };

  const renderDetail = (detail: WordDetail) => {
    card.innerHTML = buildDetailMarkup(detail);
  };

  overlay.addEventListener('click', (event) => {
    if (event.target === overlay) {
      close();
    }
  });

  card.addEventListener('click', (event) => {
    const target = event.target as HTMLElement | null;

    if (target?.closest('.word-detail-close')) {
      close();
      return;
    }

    if (target?.closest('.word-detail-pronounce') && currentDetail) {
      void invoke('speak_word', { text: currentDetail.word }).catch((error) => {
        options.onError?.(error instanceof Error ? error.message : String(error));
      });
      return;
    }

    if (target?.closest('.word-detail-wrong-book-toggle') && activeWordId !== null && currentDetail) {
      const detailSnapshot = currentDetail;
      const nextState = !detailSnapshot.in_wrong_book;
      const toggleButton = card.querySelector('.word-detail-wrong-book-toggle');
      if (toggleButton instanceof HTMLButtonElement) {
        toggleButton.disabled = true;
      }

      void invoke('set_wrong_book_state', {
        wordId: activeWordId,
        inWrongBook: nextState,
      })
        .then(async () => {
          currentDetail = { ...detailSnapshot, in_wrong_book: nextState };
          renderDetail(currentDetail);
          await options.onWrongBookChange?.(currentDetail);
        })
        .catch((error) => {
          options.onError?.(error instanceof Error ? error.message : String(error));
          renderDetail(detailSnapshot);
        });
    }
  });

  document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape' && !overlay.classList.contains('hidden')) {
      close();
    }
  });

  return {
    close,
    async open(wordId: number) {
      activeWordId = wordId;
      currentDetail = null;
      requestId += 1;
      const currentRequest = requestId;

      show();
      renderLoading();

      try {
        const detail = await invoke<WordDetail>('get_word_detail', { wordId });
        if (currentRequest !== requestId || activeWordId !== wordId) {
          return;
        }

        currentDetail = detail;
        renderDetail(detail);
      } catch (error) {
        if (currentRequest !== requestId || activeWordId !== wordId) {
          return;
        }

        const message = error instanceof Error ? error.message : String(error);
        renderError(message);
        options.onError?.(message);
      }
    },
  };
}
