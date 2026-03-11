// 核心类型定义

export interface Word {
  id: number;
  word: string;
  phonetic?: string;
  part_of_speech?: string;
  meaning_zh: string;
  source: string;
  difficulty: number;
  created_at: string;
}

export interface SrsCard {
  id: number;
  word_id: number;
  status: 'new' | 'learning' | 'mastered';
  stage: number;
  due_at?: string;
  last_seen_at?: string;
  last_result?: 'know' | 'dont_know' | 'skip';
  correct_streak: number;
  lifetime_correct: number;
  lifetime_wrong: number;
  skip_cooldown_until?: string;
  updated_at: string;
}

export interface WordWithCard {
  word: Word;
  card: SrsCard;
}

export type ReviewResult = 'know' | 'dont_know' | 'skip';

export interface ReviewUpdate {
  status: 'new' | 'learning' | 'mastered';
  stage: number;
  due_at?: string;
  last_seen_at: string;
  last_result: ReviewResult;
  correct_streak: number;
  lifetime_correct: number;
  lifetime_wrong: number;
  skip_cooldown_until?: string;
}

export interface TriggerCondition {
  idleSeconds: number;
  minIdleSeconds: number;
  isPaused: boolean;
  isInSilentPeriod: boolean;
  hasAvailableCard: boolean;
}
