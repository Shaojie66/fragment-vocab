// SRS 算法单元测试

import { describe, it, expect } from 'vitest';
import { calculateReviewUpdate, isCardDue, isInSkipCooldown } from './srsEngine';
import type { SrsCard } from '../../shared/types';

describe('SRS Engine', () => {
  const now = new Date('2024-03-12T10:00:00Z');

  // 创建测试用的基础卡片
  const createCard = (overrides: Partial<SrsCard> = {}): SrsCard => ({
    id: 1,
    word_id: 1,
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

  describe('calculateReviewUpdate', () => {
    it('新词答对后进入 stage 0', () => {
      const card = createCard({ status: 'new', stage: 0 });
      const update = calculateReviewUpdate(card, 'know', now);

      expect(update.status).toBe('learning');
      expect(update.stage).toBe(1);
      expect(update.correct_streak).toBe(1);
      expect(update.lifetime_correct).toBe(1);
      expect(update.lifetime_wrong).toBe(0);
      expect(update.due_at).toBeDefined();
    });

    it('stage 2 答错后回退到 stage 1', () => {
      const card = createCard({
        status: 'learning',
        stage: 2,
        correct_streak: 3,
        lifetime_correct: 5,
        lifetime_wrong: 1,
      });
      const update = calculateReviewUpdate(card, 'dont_know', now);

      expect(update.status).toBe('learning');
      expect(update.stage).toBe(1);
      expect(update.correct_streak).toBe(0); // 答错重置连续答对
      expect(update.lifetime_correct).toBe(5);
      expect(update.lifetime_wrong).toBe(2);
    });

    it('stage 0 答错后不低于 stage 0', () => {
      const card = createCard({
        status: 'learning',
        stage: 0,
        lifetime_wrong: 2,
      });
      const update = calculateReviewUpdate(card, 'dont_know', now);

      expect(update.stage).toBe(0); // 不会低于 0
      expect(update.lifetime_wrong).toBe(3);
    });

    it('stage 4 答对后进入 mastered', () => {
      const card = createCard({
        status: 'learning',
        stage: 4,
        correct_streak: 4,
        lifetime_correct: 10,
      });
      const update = calculateReviewUpdate(card, 'know', now);

      expect(update.status).toBe('mastered');
      expect(update.stage).toBe(5);
      expect(update.due_at).toBeUndefined(); // mastered 不需要复习
      expect(update.correct_streak).toBe(5);
    });

    it('跳过后 30 分钟内不会再次出现', () => {
      const card = createCard({ status: 'learning', stage: 1 });
      const update = calculateReviewUpdate(card, 'skip', now);

      expect(update.last_result).toBe('skip');
      expect(update.skip_cooldown_until).toBeDefined();

      const cooldownTime = new Date(update.skip_cooldown_until!);
      const expectedTime = new Date(now.getTime() + 30 * 60 * 1000);
      expect(cooldownTime.getTime()).toBe(expectedTime.getTime());
    });

    it('跳过不改变 stage 和 due_at', () => {
      const card = createCard({
        status: 'learning',
        stage: 2,
        due_at: '2024-03-12T12:00:00Z',
      });
      const update = calculateReviewUpdate(card, 'skip', now);

      expect(update.status).toBe('learning');
      expect(update.stage).toBe(2);
      expect(update.due_at).toBe('2024-03-12T12:00:00Z');
    });

    it('答对后清除 skip_cooldown_until', () => {
      const card = createCard({
        status: 'learning',
        stage: 1,
        skip_cooldown_until: '2024-03-12T11:00:00Z',
      });
      const update = calculateReviewUpdate(card, 'know', now);

      expect(update.skip_cooldown_until).toBeUndefined();
    });

    it('答错后清除 skip_cooldown_until', () => {
      const card = createCard({
        status: 'learning',
        stage: 1,
        skip_cooldown_until: '2024-03-12T11:00:00Z',
      });
      const update = calculateReviewUpdate(card, 'dont_know', now);

      expect(update.skip_cooldown_until).toBeUndefined();
    });
  });

  describe('isCardDue', () => {
    it('新词视为到期', () => {
      const card = createCard({ status: 'new', due_at: undefined });
      expect(isCardDue(card, now)).toBe(true);
    });

    it('mastered 卡片不到期', () => {
      const card = createCard({ status: 'mastered' });
      expect(isCardDue(card, now)).toBe(false);
    });

    it('due_at 早于当前时间视为到期', () => {
      const card = createCard({
        status: 'learning',
        due_at: '2024-03-12T09:00:00Z', // 1 小时前
      });
      expect(isCardDue(card, now)).toBe(true);
    });

    it('due_at 晚于当前时间不到期', () => {
      const card = createCard({
        status: 'learning',
        due_at: '2024-03-12T11:00:00Z', // 1 小时后
      });
      expect(isCardDue(card, now)).toBe(false);
    });

    it('due_at 等于当前时间视为到期', () => {
      const card = createCard({
        status: 'learning',
        due_at: now.toISOString(),
      });
      expect(isCardDue(card, now)).toBe(true);
    });
  });

  describe('isInSkipCooldown', () => {
    it('没有 skip_cooldown_until 时返回 false', () => {
      const card = createCard({ skip_cooldown_until: undefined });
      expect(isInSkipCooldown(card, now)).toBe(false);
    });

    it('skip_cooldown_until 早于当前时间返回 false', () => {
      const card = createCard({
        skip_cooldown_until: '2024-03-12T09:00:00Z', // 1 小时前
      });
      expect(isInSkipCooldown(card, now)).toBe(false);
    });

    it('skip_cooldown_until 晚于当前时间返回 true', () => {
      const card = createCard({
        skip_cooldown_until: '2024-03-12T11:00:00Z', // 1 小时后
      });
      expect(isInSkipCooldown(card, now)).toBe(true);
    });

    it('skip_cooldown_until 等于当前时间返回 false', () => {
      const card = createCard({
        skip_cooldown_until: now.toISOString(),
      });
      expect(isInSkipCooldown(card, now)).toBe(false);
    });
  });
});
