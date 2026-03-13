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
export type RecommendedReminderMode = 'gentle' | 'balanced' | 'intensive';
export type FeedbackType = 'too_many_reminders' | 'too_few_reminders' | 'not_interested_word';

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

export type ReminderMode = RecommendedReminderMode | 'custom';
export type RevealOrder = 'en-first' | 'zh-first';
export type StartBehavior = 'show-main' | 'minimize-to-tray';
export type SchedulerBlockReason =
  | 'paused'
  | 'quiet_hours'
  | 'main_window_active'
  | 'card_visible'
  | 'no_card'
  | 'idle_too_short'
  | 'ready';

export interface ReminderConfig {
  mode: ReminderMode;
  using_recommended: boolean;
  idle_threshold_sec: number;
  fallback_enabled: boolean;
  fallback_interval_min: number;
}

export interface ScheduleConfig {
  quiet_hours_start: string;
  quiet_hours_end: string;
  weekday_profile?: RecommendedReminderMode;
  weekend_profile?: RecommendedReminderMode;
}

export interface LearningConfig {
  daily_new_limit: number;
  review_first: boolean;
  allow_new_when_no_due: boolean;
}

export interface CardPreferences {
  auto_hide_sec: number;
  show_phonetic: boolean;
  reveal_order: RevealOrder;
  allow_skip: boolean;
  shortcuts_enabled: boolean;
}

export interface SystemPreferences {
  launch_at_login: boolean;
  start_behavior: StartBehavior;
  tray_enabled: boolean;
}

export interface AppConfig {
  reminder: ReminderConfig;
  schedule: ScheduleConfig;
  learning: LearningConfig;
  card: CardPreferences;
  system: SystemPreferences;
}

export interface TodayStats {
  total_reviews: number;
  know_count: number;
  dont_know_count: number;
  skip_count: number;
  new_words_today: number;
  due_cards_count: number;
  mastered_count: number;
  accuracy: number;
}

export interface DashboardState {
  app_config: AppConfig;
  today_stats: TodayStats;
  pause_until?: string;
  needs_onboarding: boolean;
  recommendation: RecommendationSummary;
  recent_feedback: FeedbackRecord[];
}

export interface SchedulerSnapshot {
  is_paused: boolean;
  pause_until?: string;
  is_card_visible: boolean;
  last_show_time?: string;
  last_block_reason: SchedulerBlockReason;
  current_mode: ReminderMode;
  idle_threshold_sec: number;
  fallback_enabled: boolean;
  fallback_interval_min: number;
  quiet_hours_start: string;
  quiet_hours_end: string;
}

export interface FeedbackRecord {
  feedback_type: FeedbackType;
  source: 'console' | 'card';
  created_at: string;
  card_id?: number;
  word?: string;
}

export interface RecommendationSummary {
  base_mode: RecommendedReminderMode;
  suggested_mode: RecommendedReminderMode;
  explanation: string;
  reasons: string[];
  source: 'static' | 'adaptive';
}

export interface TeamTemplate {
  id: string;
  name: string;
  description: string;
  summary: string;
  config: AppConfig;
}

export interface ExportBundle {
  file_name_hint: string;
  summary_text: string;
  config_json: string;
}

export type WordQuizMode = 'zh_to_en_choice' | 'en_to_zh_choice';

export interface WordQuizOption {
  id: string;
  label: string;
  detail?: string;
}

export interface WordCardData {
  word_id: number;
  card_id: number;
  word: string;
  phonetic?: string;
  part_of_speech?: string;
  meaning_zh: string;
  quiz_mode: WordQuizMode;
  prompt: string;
  prompt_hint?: string;
  options: WordQuizOption[];
  correct_option_id: string;
  explanation_title: string;
  explanation_detail: string;
}

export interface WordbookImportSummary {
  imported_count: number;
  skipped_count: number;
  total_count: number;
  source: string;
  format: string;
}

export interface WordbookListItem {
  source: string;
  display_name: string;
  total_words: number;
  enabled: boolean;
  built_in: boolean;
  first_created_at?: string;
  last_created_at?: string;
}

export interface WordbookWordItem {
  id: number;
  word: string;
  phonetic?: string;
  part_of_speech?: string;
  meaning_zh: string;
  difficulty: number;
  created_at: string;
}
