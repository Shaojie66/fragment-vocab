// 卡片选择器
// 负责选择下一张要展示的卡片：到期复习词优先，新词补位，跳过冷却处理

import type { WordWithCard } from '../../shared/types';
import { isCardDue, isInSkipCooldown } from '../srs/srsEngine';

/**
 * 从候选卡片中选择下一张要展示的卡片
 * 
 * 优先级：
 * 1. 到期的复习词（learning 状态且 due_at <= now）
 * 2. 新词（new 状态，受每日配额限制）
 * 3. 排除跳过冷却期内的卡片
 * 
 * @param candidates 候选卡片列表
 * @param now 当前时间
 * @param todayNewCount 今日已学新词数
 * @param dailyNewLimit 每日新词配额（默认 15）
 */
export function selectNextCard(
  candidates: WordWithCard[],
  now: Date = new Date(),
  todayNewCount: number = 0,
  dailyNewLimit: number = 15
): WordWithCard | null {
  if (candidates.length === 0) {
    return null;
  }

  // 过滤掉跳过冷却期内的卡片
  const availableCards = candidates.filter(
    (item) => !isInSkipCooldown(item.card, now)
  );

  if (availableCards.length === 0) {
    return null;
  }

  // 分离到期复习词和新词
  const dueCards: WordWithCard[] = [];
  const newCards: WordWithCard[] = [];

  for (const item of availableCards) {
    if (item.card.status === 'mastered') {
      continue; // 跳过已掌握的词
    }

    // 到期复习词：learning 状态且已到期
    if (item.card.status === 'learning' && isCardDue(item.card, now)) {
      dueCards.push(item);
    }
    // 新词：new 状态（不检查到期）
    else if (item.card.status === 'new') {
      newCards.push(item);
    }
  }

  // 优先返回到期复习词
  if (dueCards.length > 0) {
    // 按 due_at 排序，最早到期的优先
    dueCards.sort((a, b) => {
      const aTime = a.card.due_at ? new Date(a.card.due_at).getTime() : 0;
      const bTime = b.card.due_at ? new Date(b.card.due_at).getTime() : 0;
      return aTime - bTime;
    });
    return dueCards[0];
  }

  // 没有到期复习词时，返回新词（检查配额）
  if (newCards.length > 0) {
    // 检查今日新词配额
    if (todayNewCount >= dailyNewLimit) {
      return null; // 已达配额上限
    }
    
    // 按难度排序，简单的优先
    newCards.sort((a, b) => a.word.difficulty - b.word.difficulty);
    return newCards[0];
  }

  return null;
}

/**
 * 检查是否有可用的卡片
 */
export function hasAvailableCard(
  candidates: WordWithCard[],
  now: Date = new Date(),
  todayNewCount: number = 0,
  dailyNewLimit: number = 15
): boolean {
  return selectNextCard(candidates, now, todayNewCount, dailyNewLimit) !== null;
}
