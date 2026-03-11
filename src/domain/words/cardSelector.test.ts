// 选词逻辑单元测试

import { describe, it, expect } from 'vitest';
import { selectNextCard, hasAvailableCard } from './cardSelector';
import type { WordWithCard, Word, SrsCard } from '../../shared/types';

describe('Card Selector', () => {
  const now = new Date('2024-03-12T10:00:00Z');

  // 创建测试用的 Word
  const createWord = (id: number, difficulty: number = 5): Word => ({
    id,
    word: `word${id}`,
    meaning_zh: `definition${id}`,
    source: 'test',
    difficulty,
    created_at: now.toISOString(),
  });

  // 创建测试用的 SrsCard
  const createCard = (id: number, overrides: Partial<SrsCard> = {}): SrsCard => ({
    id,
    word_id: id,
    status: 'new',
    stage: 0,
    due_at: undefined,
    last_seen_at: undefined,
    last_result: undefined,
    correct_streak: 0,
    lifetime_correct: 0,
    lifetime_wrong: 0,
    skip_cooldown_until: undefined,
    updated_at: now.toISOString(),
    ...overrides,
  });

  // 创建测试用的 WordWithCard
  const createWordWithCard = (
    id: number,
    cardOverrides: Partial<SrsCard> = {},
    difficulty: number = 5
  ): WordWithCard => ({
    word: createWord(id, difficulty),
    card: createCard(id, cardOverrides),
  });

  describe('selectNextCard', () => {
    it('空候选列表返回 null', () => {
      const result = selectNextCard([], now);
      expect(result).toBeNull();
    });

    it('优先返回到期的复习词', () => {
      const candidates: WordWithCard[] = [
        createWordWithCard(1, { status: 'new' }), // 新词
        createWordWithCard(2, {
          status: 'learning',
          stage: 1,
          due_at: '2024-03-12T09:00:00Z', // 1 小时前到期
        }),
        createWordWithCard(3, {
          status: 'learning',
          stage: 2,
          due_at: '2024-03-12T08:00:00Z', // 2 小时前到期
        }),
      ];

      const result = selectNextCard(candidates, now);
      expect(result?.word.id).toBe(3); // 最早到期的优先
    });

    it('没有到期复习词时返回新词', () => {
      const candidates: WordWithCard[] = [
        createWordWithCard(1, { status: 'new' }, 3),
        createWordWithCard(2, { status: 'new' }, 7),
        createWordWithCard(3, {
          status: 'learning',
          stage: 1,
          due_at: '2024-03-12T11:00:00Z', // 未到期
        }),
      ];

      const result = selectNextCard(candidates, now, 0, 15);
      expect(result?.word.id).toBe(1); // 难度低的优先
    });

    it('新词配额已满时返回 null', () => {
      const candidates: WordWithCard[] = [
        createWordWithCard(1, { status: 'new' }),
        createWordWithCard(2, { status: 'new' }),
      ];

      const result = selectNextCard(candidates, now, 15, 15); // 已学 15 个，配额 15
      expect(result).toBeNull();
    });

    it('过滤掉跳过冷却期内的卡片', () => {
      const candidates: WordWithCard[] = [
        createWordWithCard(1, {
          status: 'learning',
          stage: 1,
          due_at: '2024-03-12T09:00:00Z',
          skip_cooldown_until: '2024-03-12T11:00:00Z', // 冷却中
        }),
        createWordWithCard(2, {
          status: 'learning',
          stage: 1,
          due_at: '2024-03-12T09:00:00Z',
        }),
      ];

      const result = selectNextCard(candidates, now);
      expect(result?.word.id).toBe(2); // 跳过冷却中的卡片
    });

    it('所有卡片都在冷却期时返回 null', () => {
      const candidates: WordWithCard[] = [
        createWordWithCard(1, {
          status: 'learning',
          stage: 1,
          due_at: '2024-03-12T09:00:00Z',
          skip_cooldown_until: '2024-03-12T11:00:00Z',
        }),
        createWordWithCard(2, {
          status: 'new',
          skip_cooldown_until: '2024-03-12T11:00:00Z',
        }),
      ];

      const result = selectNextCard(candidates, now);
      expect(result).toBeNull();
    });

    it('跳过已掌握的词', () => {
      const candidates: WordWithCard[] = [
        createWordWithCard(1, { status: 'mastered', stage: 5 }),
        createWordWithCard(2, { status: 'new' }),
      ];

      const result = selectNextCard(candidates, now);
      expect(result?.word.id).toBe(2);
    });

    it('有到期复习词时不返回新词', () => {
      const candidates: WordWithCard[] = [
        createWordWithCard(1, { status: 'new' }),
        createWordWithCard(2, {
          status: 'learning',
          stage: 1,
          due_at: '2024-03-12T09:00:00Z',
        }),
      ];

      const result = selectNextCard(candidates, now, 0, 15);
      expect(result?.word.id).toBe(2); // 优先复习词
    });

    it('按难度排序新词', () => {
      const candidates: WordWithCard[] = [
        createWordWithCard(1, { status: 'new' }, 8),
        createWordWithCard(2, { status: 'new' }, 3),
        createWordWithCard(3, { status: 'new' }, 5),
      ];

      const result = selectNextCard(candidates, now, 0, 15);
      expect(result?.word.id).toBe(2); // 难度 3 最低
    });

    it('按 due_at 排序到期复习词', () => {
      const candidates: WordWithCard[] = [
        createWordWithCard(1, {
          status: 'learning',
          stage: 1,
          due_at: '2024-03-12T09:30:00Z',
        }),
        createWordWithCard(2, {
          status: 'learning',
          stage: 1,
          due_at: '2024-03-12T08:00:00Z',
        }),
        createWordWithCard(3, {
          status: 'learning',
          stage: 1,
          due_at: '2024-03-12T09:00:00Z',
        }),
      ];

      const result = selectNextCard(candidates, now);
      expect(result?.word.id).toBe(2); // 最早到期
    });
  });

  describe('hasAvailableCard', () => {
    it('有可用卡片时返回 true', () => {
      const candidates: WordWithCard[] = [
        createWordWithCard(1, { status: 'new' }),
      ];

      expect(hasAvailableCard(candidates, now, 0, 15)).toBe(true);
    });

    it('无可用卡片时返回 false', () => {
      const candidates: WordWithCard[] = [
        createWordWithCard(1, {
          status: 'new',
          skip_cooldown_until: '2024-03-12T11:00:00Z',
        }),
      ];

      expect(hasAvailableCard(candidates, now, 0, 15)).toBe(false);
    });

    it('配额已满时返回 false', () => {
      const candidates: WordWithCard[] = [
        createWordWithCard(1, { status: 'new' }),
      ];

      expect(hasAvailableCard(candidates, now, 15, 15)).toBe(false);
    });
  });
});
