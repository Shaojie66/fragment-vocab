import type {
  AppConfig,
  RecommendedReminderMode,
  ReminderMode,
  SchedulerBlockReason,
} from './types';

const REMINDER_PRESETS: Record<
  RecommendedReminderMode,
  Pick<AppConfig['reminder'], 'idle_threshold_sec' | 'fallback_enabled' | 'fallback_interval_min'>
> = {
  gentle: {
    idle_threshold_sec: 180,
    fallback_enabled: true,
    fallback_interval_min: 45,
  },
  balanced: {
    idle_threshold_sec: 120,
    fallback_enabled: true,
    fallback_interval_min: 30,
  },
  intensive: {
    idle_threshold_sec: 90,
    fallback_enabled: true,
    fallback_interval_min: 20,
  },
};

export function createDefaultAppConfig(): AppConfig {
  return {
    reminder: {
      mode: 'gentle',
      using_recommended: true,
      ...REMINDER_PRESETS.gentle,
    },
    schedule: {
      quiet_hours_start: '23:00',
      quiet_hours_end: '07:00',
      weekday_profile: 'gentle',
      weekend_profile: 'balanced',
    },
    learning: {
      daily_new_limit: 10,
      review_first: true,
      allow_new_when_no_due: true,
    },
    card: {
      auto_hide_sec: 10,
      show_phonetic: true,
      reveal_order: 'en-first',
      allow_skip: true,
      shortcuts_enabled: true,
    },
    system: {
      launch_at_login: false,
      start_behavior: 'show-main',
      tray_enabled: true,
    },
  };
}

export function isRecommendedReminderMode(value: string | undefined | null): value is RecommendedReminderMode {
  return value === 'gentle' || value === 'balanced' || value === 'intensive';
}

export function getReminderPreset(
  mode: RecommendedReminderMode,
): Pick<AppConfig['reminder'], 'idle_threshold_sec' | 'fallback_enabled' | 'fallback_interval_min'> {
  return REMINDER_PRESETS[mode];
}

export function applyModePreset(baseConfig: AppConfig, mode: RecommendedReminderMode): AppConfig {
  return {
    ...baseConfig,
    reminder: {
      ...baseConfig.reminder,
      ...getReminderPreset(mode),
      mode,
      using_recommended: true,
    },
  };
}

export function isWeekendDay(date: Date = new Date()): boolean {
  const day = date.getDay();
  return day === 0 || day === 6;
}

export function getScheduleSegmentLabel(date: Date = new Date()): string {
  return isWeekendDay(date) ? '周末' : '工作日';
}

export function getActiveRecommendedMode(
  config: AppConfig,
  date: Date = new Date(),
): RecommendedReminderMode {
  const fallbackMode = isRecommendedReminderMode(config.reminder.mode) ? config.reminder.mode : 'gentle';
  const candidate = isWeekendDay(date) ? config.schedule.weekend_profile : config.schedule.weekday_profile;
  return isRecommendedReminderMode(candidate) ? candidate : fallbackMode;
}

export function getEffectiveReminderConfig(config: AppConfig, date: Date = new Date()): AppConfig['reminder'] {
  if (!config.reminder.using_recommended || config.reminder.mode === 'custom') {
    return config.reminder;
  }

  const activeMode = getActiveRecommendedMode(config, date);
  return {
    ...config.reminder,
    ...getReminderPreset(activeMode),
    mode: activeMode,
    using_recommended: true,
  };
}

export function getModeLabel(mode: ReminderMode): string {
  switch (mode) {
    case 'gentle':
      return '克制';
    case 'balanced':
      return '平衡';
    case 'intensive':
      return '强化';
    case 'custom':
      return '自定义';
  }
}

export function getBlockReasonLabel(reason: SchedulerBlockReason): string {
  switch (reason) {
    case 'paused':
      return '暂停中';
    case 'quiet_hours':
      return '夜间静默中';
    case 'main_window_active':
      return '主页面使用中';
    case 'card_visible':
      return '当前有卡片展示中';
    case 'no_card':
      return '当前无可学习卡片';
    case 'idle_too_short':
      return '当前空闲时间不足';
    case 'ready':
      return '已满足提醒条件';
  }
}
