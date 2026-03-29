import { invoke } from '@tauri-apps/api/core';
import type { SearchResult, WordbookImportSummary, WordbookListItem, WordbookWordItem } from '../shared/types';
import { mainElements } from './elements';
import { fileToBase64, formatDateTime, getErrorMessage } from './helpers';
import { mainState } from './state';

const WORDBOOK_PREVIEW_PAGE_SIZE = 12;
const WORDBOOK_SEARCH_LIMIT = 20;
const WORDBOOK_SEARCH_DEBOUNCE_MS = 300;

interface WordbookDependencies {
  refreshDashboard: () => Promise<void>;
  renderDashboard: () => void;
  setSaveHint: (message: string) => void;
}

let dependencies: WordbookDependencies | null = null;
let searchDebounceTimer: number | null = null;
let activeSearchRequestId = 0;

function getStatusLabel(status: SearchResult['status']) {
  if (status === 'learning') {
    return '学习中';
  }

  if (status === 'mastered') {
    return '已掌握';
  }

  return '新词';
}

function getStatusBadgeClass(status: SearchResult['status']) {
  return `wordbook-badge status-${status}`;
}

function renderWordSearch() {
  const query = mainState.currentWordbookSearchQuery.trim();
  mainElements.wordbookSearchInput.value = mainState.currentWordbookSearchQuery;

  if (!query) {
    mainElements.wordbookSearchMeta.textContent = '输入关键词后，会在全部词库中搜索英文或中文释义。';
    mainElements.wordbookSearchResults.innerHTML = '<div class="wordbook-empty">开始输入后，会在这里显示匹配的词条。</div>';
    return;
  }

  if (mainState.isWordbookSearchLoading) {
    mainElements.wordbookSearchMeta.textContent = `正在搜索“${query}”...`;
    mainElements.wordbookSearchResults.innerHTML = '<div class="wordbook-empty">正在搜索词条...</div>';
    return;
  }

  if (!mainState.currentWordbookSearchResults.length) {
    mainElements.wordbookSearchMeta.textContent = `没有找到与“${query}”相关的词条。`;
    mainElements.wordbookSearchResults.innerHTML = '<div class="wordbook-empty">没有匹配结果，请尝试更换关键词。</div>';
    return;
  }

  mainElements.wordbookSearchMeta.textContent = `找到 ${mainState.currentWordbookSearchResults.length} 条与“${query}”相关的结果。`;
  mainElements.wordbookSearchResults.innerHTML = '';

  mainState.currentWordbookSearchResults.forEach((item) => {
    const card = document.createElement('article');
    card.className = 'wordbook-search-item';

    const topline = document.createElement('div');
    topline.className = 'wordbook-search-topline';

    const titleWrap = document.createElement('div');
    const title = document.createElement('strong');
    title.textContent = item.word;
    titleWrap.appendChild(title);

    const aux = document.createElement('p');
    aux.className = 'wordbook-preview-aux';
    aux.textContent = [item.phonetic, item.part_of_speech].filter(Boolean).join(' · ') || '未提供音标或词性';
    titleWrap.appendChild(aux);

    const statusBadge = document.createElement('span');
    statusBadge.className = getStatusBadgeClass(item.status);
    statusBadge.textContent = getStatusLabel(item.status);

    topline.append(titleWrap, statusBadge);

    const meaning = document.createElement('p');
    meaning.className = 'wordbook-search-meaning';
    meaning.textContent = item.meaning_zh;

    const meta = document.createElement('div');
    meta.className = 'wordbook-search-meta';

    const sourceBadge = document.createElement('span');
    sourceBadge.className = 'wordbook-badge muted';
    sourceBadge.textContent = item.source;
    meta.appendChild(sourceBadge);

    card.append(topline, meaning, meta);
    mainElements.wordbookSearchResults.appendChild(card);
  });
}

async function runWordSearch(query: string) {
  const normalizedQuery = query.trim();
  const requestId = activeSearchRequestId;
  mainState.currentWordbookSearchQuery = query;

  if (!normalizedQuery) {
    mainState.isWordbookSearchLoading = false;
    mainState.currentWordbookSearchResults = [];
    renderWordSearch();
    return;
  }

  mainState.isWordbookSearchLoading = true;
  renderWordSearch();

  try {
    const results = await invoke<SearchResult[]>('search_words', {
      query: normalizedQuery,
      limit: WORDBOOK_SEARCH_LIMIT,
    });

    if (requestId !== activeSearchRequestId) {
      return;
    }

    mainState.currentWordbookSearchResults = results;
    mainState.lastErrorMessage = null;
  } catch (error) {
    if (requestId !== activeSearchRequestId) {
      return;
    }

    mainState.isWordbookSearchLoading = false;
    mainState.currentWordbookSearchResults = [];
    mainState.lastErrorMessage = getErrorMessage(error);
    dependencies?.renderDashboard();
    mainElements.wordbookSearchMeta.textContent = '搜索失败';
    mainElements.wordbookSearchResults.innerHTML = `<div class="wordbook-empty">${mainState.lastErrorMessage}</div>`;
    dependencies?.setSaveHint('词条搜索失败，请稍后重试。');
    return;
  }

  if (requestId !== activeSearchRequestId) {
    return;
  }

  mainState.isWordbookSearchLoading = false;
  renderWordSearch();
}

function scheduleWordSearch(query: string) {
  activeSearchRequestId += 1;
  mainState.currentWordbookSearchQuery = query;

  if (searchDebounceTimer !== null) {
    window.clearTimeout(searchDebounceTimer);
  }

  searchDebounceTimer = window.setTimeout(() => {
    searchDebounceTimer = null;
    void runWordSearch(query);
  }, WORDBOOK_SEARCH_DEBOUNCE_MS);

  renderWordSearch();
}

export function renderWordbooks() {
  if (!mainState.currentWordbooks.length) {
    mainElements.wordbookList.innerHTML = '<div class="wordbook-empty">当前还没有可用词库。</div>';
    return;
  }

  mainElements.wordbookList.innerHTML = '';

  mainState.currentWordbooks.forEach((item) => {
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

    const previewButton = document.createElement('button');
    previewButton.className = 'ghost-btn';
    previewButton.type = 'button';
    previewButton.dataset.source = item.source;
    previewButton.dataset.action = 'preview';
    previewButton.textContent = mainState.currentWordbookPreviewSource === item.source ? '刷新预览' : '查看单词';
    actions.appendChild(previewButton);

    const toggleButton = document.createElement('button');
    toggleButton.className = 'ghost-btn';
    toggleButton.type = 'button';
    toggleButton.dataset.source = item.source;
    toggleButton.dataset.action = 'toggle';
    toggleButton.textContent = item.enabled ? '停用' : '启用';
    actions.appendChild(toggleButton);

    if (!item.built_in) {
      const deleteButton = document.createElement('button');
      deleteButton.className = 'ghost-btn';
      deleteButton.type = 'button';
      deleteButton.dataset.source = item.source;
      deleteButton.dataset.action = 'delete';
      deleteButton.textContent = '删除';
      actions.appendChild(deleteButton);
    }

    row.append(copy, actions);
    mainElements.wordbookList.appendChild(row);
  });
}

export function closeWordbookPreview() {
  mainState.currentWordbookPreviewSource = null;
  mainState.currentWordbookPreviewWords = [];
  mainState.currentWordbookPreviewOffset = 0;
  mainState.isWordbookPreviewLoading = false;
  mainElements.wordbookPreview.classList.add('hidden');
  renderWordbooks();
}

export function renderWordbookPreview() {
  if (!mainState.currentWordbookPreviewSource) {
    mainElements.wordbookPreview.classList.add('hidden');
    return;
  }

  const selectedWordbook = mainState.currentWordbooks.find((item) => item.source === mainState.currentWordbookPreviewSource);

  if (!selectedWordbook) {
    closeWordbookPreview();
    return;
  }

  mainElements.wordbookPreview.classList.remove('hidden');
  mainElements.wordbookPreviewTitle.textContent = `${selectedWordbook.display_name} · 词条预览`;

  if (mainState.isWordbookPreviewLoading) {
    mainElements.wordbookPreviewMeta.textContent = '正在读取词条...';
    mainElements.wordbookPreviewList.innerHTML = '<div class="wordbook-empty">正在读取词条...</div>';
  } else if (!mainState.currentWordbookPreviewWords.length) {
    mainElements.wordbookPreviewMeta.textContent = `${selectedWordbook.total_words} 个单词 · 当前词库${selectedWordbook.enabled ? '已启用' : '已停用'}`;
    mainElements.wordbookPreviewList.innerHTML = '<div class="wordbook-empty">这个词库当前没有可展示的词条。</div>';
  } else {
    const start = mainState.currentWordbookPreviewOffset + 1;
    const end = mainState.currentWordbookPreviewOffset + mainState.currentWordbookPreviewWords.length;
    mainElements.wordbookPreviewMeta.textContent = `第 ${start}-${end} / ${selectedWordbook.total_words} 个单词 · 当前词库${selectedWordbook.enabled ? '已启用' : '已停用'}`;
    mainElements.wordbookPreviewList.innerHTML = '';

    mainState.currentWordbookPreviewWords.forEach((item) => {
      const card = document.createElement('article');
      card.className = 'wordbook-preview-item';

      const topline = document.createElement('div');
      topline.className = 'wordbook-preview-topline';

      const title = document.createElement('strong');
      title.textContent = item.word;
      topline.appendChild(title);

      const difficultyBadge = document.createElement('span');
      difficultyBadge.className = 'wordbook-badge muted';
      difficultyBadge.textContent = `难度 ${item.difficulty}`;
      topline.appendChild(difficultyBadge);

      const aux = document.createElement('p');
      aux.className = 'wordbook-preview-aux';
      aux.textContent = [item.phonetic, item.part_of_speech].filter(Boolean).join(' · ') || '未提供音标或词性';

      const meaning = document.createElement('p');
      meaning.className = 'wordbook-preview-meaning';
      meaning.textContent = item.meaning_zh;

      const bottomline = document.createElement('div');
      bottomline.className = 'wordbook-preview-bottomline';

      const idBadge = document.createElement('span');
      idBadge.className = 'wordbook-badge';
      idBadge.textContent = `#${item.id}`;

      const createdText = document.createElement('span');
      createdText.className = 'wordbook-preview-aux';
      createdText.textContent = `导入于 ${formatDateTime(item.created_at)}`;

      bottomline.append(idBadge, createdText);
      card.append(topline, aux, meaning, bottomline);
      mainElements.wordbookPreviewList.appendChild(card);
    });
  }

  const hasPrev = mainState.currentWordbookPreviewOffset > 0;
  const hasNext = mainState.currentWordbookPreviewOffset + mainState.currentWordbookPreviewWords.length < selectedWordbook.total_words;
  mainElements.wordbookPreviewPrevBtn.disabled = mainState.isWordbookPreviewLoading || !hasPrev;
  mainElements.wordbookPreviewNextBtn.disabled = mainState.isWordbookPreviewLoading || !hasNext;
}

async function openWordbookPreview(source: string, offset = 0) {
  const selectedWordbook = mainState.currentWordbooks.find((item) => item.source === source);

  if (!selectedWordbook) {
    return;
  }

  mainState.currentWordbookPreviewSource = source;
  mainState.currentWordbookPreviewOffset = Math.max(0, offset);
  mainState.isWordbookPreviewLoading = true;
  renderWordbooks();
  renderWordbookPreview();

  try {
    mainState.currentWordbookPreviewWords = await invoke<WordbookWordItem[]>('list_wordbook_words', {
      source,
      limit: WORDBOOK_PREVIEW_PAGE_SIZE,
      offset: mainState.currentWordbookPreviewOffset,
    });
    renderWordbookPreview();
  } catch (error) {
    mainState.isWordbookPreviewLoading = false;
    mainState.currentWordbookPreviewWords = [];
    mainState.lastErrorMessage = getErrorMessage(error);
    dependencies?.renderDashboard();
    mainElements.wordbookPreview.classList.remove('hidden');
    mainElements.wordbookPreviewTitle.textContent = `${selectedWordbook.display_name} · 词条预览`;
    mainElements.wordbookPreviewMeta.textContent = '词条读取失败';
    mainElements.wordbookPreviewList.innerHTML = `<div class="wordbook-empty">${mainState.lastErrorMessage}</div>`;
    mainElements.wordbookPreviewPrevBtn.disabled = true;
    mainElements.wordbookPreviewNextBtn.disabled = true;
    dependencies?.setSaveHint('读取词库预览失败，请稍后重试。');
    return;
  }

  mainState.isWordbookPreviewLoading = false;
  renderWordbooks();
  renderWordbookPreview();
}

export async function loadWordbooks() {
  mainState.currentWordbooks = await invoke<WordbookListItem[]>('list_wordbooks');
  renderWordSearch();
  renderWordbooks();

  if (!mainState.currentWordbookPreviewSource) {
    return;
  }

  const selectedWordbook = mainState.currentWordbooks.find((item) => item.source === mainState.currentWordbookPreviewSource);

  if (!selectedWordbook) {
    closeWordbookPreview();
    return;
  }

  renderWordbookPreview();
}

async function importCustomWordbook(file: File) {
  const contentBase64 = await fileToBase64(file);
  const summary = await invoke<WordbookImportSummary>('import_custom_wordbook', {
    fileName: file.name,
    contentBase64,
  });

  await loadWordbooks();

  if (mainState.currentWordbookPreviewSource === summary.source) {
    await openWordbookPreview(summary.source, 0);
  }

  await dependencies?.refreshDashboard();
  mainState.currentExportBundle = null;
  mainElements.wordbookUploadHint.textContent = `已导入 ${summary.imported_count} 个单词，跳过 ${summary.skipped_count} 个重复或无效条目。格式：${summary.format.toUpperCase()} · 来源：${file.name}`;
  dependencies?.setSaveHint(`词库导入完成，新增 ${summary.imported_count} 个单词。`);
}

async function toggleWordbook(source: string, enabled: boolean) {
  try {
    mainState.currentWordbooks = await invoke<WordbookListItem[]>('set_wordbook_enabled', {
      source,
      enabled,
    });
    renderWordbooks();
    await dependencies?.refreshDashboard();

    if (mainState.currentWordbookPreviewSource === source) {
      renderWordbookPreview();
    }

    dependencies?.setSaveHint(enabled ? '词库已重新启用。' : '词库已停用，之后不会继续出题。');
  } catch (error) {
    mainState.lastErrorMessage = getErrorMessage(error);
    dependencies?.renderDashboard();
    dependencies?.setSaveHint('更新词库状态失败，请稍后重试。');
  }
}

async function deleteWordbookBySource(source: string) {
  try {
    mainState.currentWordbooks = await invoke<WordbookListItem[]>('delete_wordbook', { source });

    if (mainState.currentWordbookPreviewSource === source) {
      closeWordbookPreview();
    } else {
      renderWordbooks();
    }

    await dependencies?.refreshDashboard();
    dependencies?.setSaveHint('词库已删除，对应单词和学习记录也已移除。');
  } catch (error) {
    mainState.lastErrorMessage = getErrorMessage(error);
    dependencies?.renderDashboard();
    dependencies?.setSaveHint('删除词库失败，请稍后重试。');
  }
}

export function initializeWordbooks(nextDependencies: WordbookDependencies) {
  dependencies = nextDependencies;
  renderWordSearch();

  mainElements.uploadWordbookBtn.addEventListener('click', () => {
    mainElements.uploadWordbookFileInput.click();
  });
  mainElements.wordbookSearchInput.addEventListener('input', () => {
    scheduleWordSearch(mainElements.wordbookSearchInput.value);
  });
  mainElements.closeWordbookPreviewBtn.addEventListener('click', () => {
    closeWordbookPreview();
  });
  mainElements.wordbookPreviewPrevBtn.addEventListener('click', () => {
    if (!mainState.currentWordbookPreviewSource || mainState.currentWordbookPreviewOffset <= 0) {
      return;
    }

    void openWordbookPreview(
      mainState.currentWordbookPreviewSource,
      mainState.currentWordbookPreviewOffset - WORDBOOK_PREVIEW_PAGE_SIZE,
    );
  });
  mainElements.wordbookPreviewNextBtn.addEventListener('click', () => {
    if (!mainState.currentWordbookPreviewSource) {
      return;
    }

    void openWordbookPreview(
      mainState.currentWordbookPreviewSource,
      mainState.currentWordbookPreviewOffset + WORDBOOK_PREVIEW_PAGE_SIZE,
    );
  });
  mainElements.wordbookList.addEventListener('click', (event) => {
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

    if (action === 'preview') {
      void openWordbookPreview(source);
      return;
    }

    if (action === 'toggle') {
      const wordbook = mainState.currentWordbooks.find((item) => item.source === source);

      if (!wordbook) {
        return;
      }

      void toggleWordbook(source, !wordbook.enabled);
      return;
    }

    if (action === 'delete') {
      const wordbook = mainState.currentWordbooks.find((item) => item.source === source);

      if (!wordbook || wordbook.built_in) {
        return;
      }

      const confirmed = window.confirm(`确定删除词库“${wordbook.display_name}”吗？删除后对应单词和学习记录会一起移除。`);

      if (!confirmed) {
        return;
      }

      void deleteWordbookBySource(source);
    }
  });
  mainElements.uploadWordbookFileInput.addEventListener('change', () => {
    void (async () => {
      const [file] = Array.from(mainElements.uploadWordbookFileInput.files ?? []);

      if (!file) {
        return;
      }

      try {
        mainElements.wordbookUploadHint.textContent = `正在导入 ${file.name}...`;
        await importCustomWordbook(file);
      } catch (error) {
        mainState.lastErrorMessage = getErrorMessage(error);
        dependencies?.renderDashboard();
        mainElements.wordbookUploadHint.textContent = '导入失败。请使用 JSON / CSV / TXT / XLSX，并确保至少包含单词列和中文释义列，例如 word/meaning_zh、English/Translation 或 单词/中文。';
        dependencies?.setSaveHint('自定义词库导入失败，请检查文件格式后重试。');
      } finally {
        mainElements.uploadWordbookFileInput.value = '';
      }
    })();
  });
}
