// SRS 复习算法引擎
// 负责阶段流转、下次复习时间计算、mastered 判定

import type { SrsCard, ReviewResult, ReviewUpdate } from '../../shared/types';

// SRS 阶段间隔定义（分钟）
const STAGE_INTERVALS = [
  10,        // Stage 0: 10 分钟
  1440,      // Stage 1: 1 天
  4320,      // Stage 2: 3 天
  10080,     // Stage 3: 7 天
  20160,     // Stage 4: 14 天
];

const SKIP_COOLDOWN_MINUTES = 30;
const MASTERED_STAGE = 5; // Stage 4 答对后进入 mastered

/**
 * 计算复习结果后的卡片更新
 */
export function calculateReviewUpdate(
  card: SrsCard,
  result: ReviewResult,
  now: Date = new Date()
): ReviewUpdate {
  const nowISO = now.toISOString();
  
  // 处理跳过
  if (result === 'skip') {
    const skipCooldownUntil = new Date(now.getTime() + SKIP_COOLDOWN_MINUTES * 60 * 1000);
    return {
      status: card.status,
      stage: card.stage,
      due_at: card.due_at,
      last_seen_at: nowISO,
      last_result: 'skip',
      correct_streak: card.correct_streak,
      lifetime_correct: card.lifetime_correct,
      lifetime_wrong: card.lifetime_wrong,
      skip_cooldown_until: skipCooldownUntil.toISOString(),
    };
  }

  // 处理答对
  if (result === 'know') {
    const newStage = card.stage + 1;
    const newCorrectStreak = card.correct_streak + 1;
    const newLifetimeCorrect = card.lifetime_correct + 1;

    // 判断是否进入 mastered
    if (newStage >= MASTERED_STAGE) {
      return {
        status: 'mastered',
        stage: newStage,
        due_at: undefined, // mastered 不再需要复习
        last_seen_at: nowISO,
        last_result: 'know',
        correct_streak: newCorrectStreak,
        lifetime_correct: newLifetimeCorrect,
        lifetime_wrong: card.lifetime_wrong,
        skip_cooldown_until: undefined,
      };
    }

    // 计算下次复习时间
    const intervalMinutes = STAGE_INTERVALS[newStage];
    const dueAt = new Date(now.getTime() + intervalMinutes * 60 * 1000);

    return {
      status: 'learning',
      stage: newStage,
      due_at: dueAt.toISOString(),
      last_seen_at: nowISO,
      last_result: 'know',
      correct_streak: newCorrectStreak,
      lifetime_correct: newLifetimeCorrect,
      lifetime_wrong: card.lifetime_wrong,
      skip_cooldown_until: undefined,
    };
  }

  // 处理答错
  if (result === 'dont_know') {
    // 回退一个阶段，但不低于 Stage 0
    const newStage = Math.max(0, card.stage - 1);
    const newLifetimeWrong = card.lifetime_wrong + 1;

    // 计算下次复习时间
    const intervalMinutes = STAGE_INTERVALS[newStage];
    const dueAt = new Date(now.getTime() + intervalMinutes * 60 * 1000);

    return {
      status: card.status === 'new' ? 'learning' : card.status,
      stage: newStage,
      due_at: dueAt.toISOString(),
      last_seen_at: nowISO,
      last_result: 'dont_know',
      correct_streak: 0, // 答错重置连续答对计数
      lifetime_correct: card.lifetime_correct,
      lifetime_wrong: newLifetimeWrong,
      skip_cooldown_until: undefined,
    };
  }

  // 不应该到达这里
  throw new Error(`Unknown review result: ${result}`);
}

/**
 * 判断卡片是否已到期
 */
export function isCardDue(card: SrsCard, now: Date = new Date()): boolean {
  if (card.status === 'mastered') {
    return false;
  }
  
  if (!card.due_at) {
    // 新词视为到期
    return card.status === 'new';
  }

  return new Date(card.due_at) <= now;
}

/**
 * 判断卡片是否在跳过冷却期内
 */
export function isInSkipCooldown(card: SrsCard, now: Date = new Date()): boolean {
  if (!card.skip_cooldown_until) {
    return false;
  }
  
  return new Date(card.skip_cooldown_until) > now;
}
