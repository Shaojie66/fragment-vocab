// 触发调度器
// 负责定时轮询并判断是否满足弹卡条件

import { invoke } from '@tauri-apps/api/core';
import { createDefaultAppConfig, getEffectiveReminderConfig } from '../../shared/config';
import type { AppConfig, SchedulerBlockReason, SchedulerSnapshot } from '../../shared/types';

const POLL_INTERVAL_MS = 15 * 1000;

interface SchedulerState {
  isPaused: boolean;
  pauseUntil?: Date;
  lastShowTime?: Date;
  isCardVisible: boolean;
  skipCooldownUntil?: Date;
  lastBlockReason: SchedulerBlockReason;
}

export class TriggerScheduler {
  private config: AppConfig;
  private state: SchedulerState = {
    isPaused: false,
    isCardVisible: false,
    lastBlockReason: 'idle_too_short',
  };

  private intervalId?: number;
  private onTrigger?: () => void | Promise<void>;

  constructor(config: AppConfig = createDefaultAppConfig()) {
    this.config = config;
  }

  start(onTrigger: () => void | Promise<void>) {
    this.onTrigger = onTrigger;

    if (this.intervalId !== undefined) {
      window.clearInterval(this.intervalId);
    }

    this.intervalId = window.setInterval(() => {
      void this.checkAndTrigger();
    }, POLL_INTERVAL_MS);

    console.log('✅ TriggerScheduler started');
  }

  stop() {
    if (this.intervalId !== undefined) {
      window.clearInterval(this.intervalId);
      this.intervalId = undefined;
    }
    console.log('⏹️  TriggerScheduler stopped');
  }

  updateConfig(config: AppConfig) {
    this.config = config;
  }

  pause(durationMinutes: number = 60) {
    const pauseUntil = new Date(Date.now() + durationMinutes * 60 * 1000);
    this.state.isPaused = true;
    this.state.pauseUntil = pauseUntil;
    this.state.lastBlockReason = 'paused';
    console.log(`⏸️  Paused until ${pauseUntil.toLocaleString()}`);
  }

  resume() {
    this.state.isPaused = false;
    this.state.pauseUntil = undefined;
    this.state.lastBlockReason = 'idle_too_short';
    console.log('▶️  Resumed');
  }

  syncPauseUntil(pauseUntil?: string) {
    if (!pauseUntil) {
      this.resume();
      return;
    }

    const nextPauseUntil = new Date(pauseUntil);
    if (Number.isNaN(nextPauseUntil.getTime()) || nextPauseUntil <= new Date()) {
      this.resume();
      return;
    }

    this.state.isPaused = true;
    this.state.pauseUntil = nextPauseUntil;
    this.state.lastBlockReason = 'paused';
  }

  markCardShown() {
    this.state.lastShowTime = new Date();
    this.state.isCardVisible = true;
    this.state.lastBlockReason = 'card_visible';
  }

  markCardHidden() {
    this.state.isCardVisible = false;
    this.state.lastBlockReason = 'idle_too_short';
  }

  getSnapshot(): SchedulerSnapshot {
    const effectiveReminder = getEffectiveReminderConfig(this.config);

    return {
      is_paused: this.state.isPaused,
      pause_until: this.state.pauseUntil?.toISOString(),
      is_card_visible: this.state.isCardVisible,
      last_show_time: this.state.lastShowTime?.toISOString(),
      last_block_reason: this.state.lastBlockReason,
      current_mode: effectiveReminder.mode,
      idle_threshold_sec: effectiveReminder.idle_threshold_sec,
      fallback_enabled: effectiveReminder.fallback_enabled,
      fallback_interval_min: effectiveReminder.fallback_interval_min,
      quiet_hours_start: this.config.schedule.quiet_hours_start,
      quiet_hours_end: this.config.schedule.quiet_hours_end,
    };
  }

  private async checkAndTrigger() {
    try {
      const shouldTrigger = await this.shouldTriggerCard();

      if (shouldTrigger && this.onTrigger) {
        console.log('🎯 Triggering card display');
        await this.onTrigger();
      }
    } catch (error) {
      console.error('❌ Error in checkAndTrigger:', error);
    }
  }

  private async shouldTriggerCard(): Promise<boolean> {
    const now = new Date();
    const effectiveReminder = getEffectiveReminderConfig(this.config, now);

    if (this.state.isPaused) {
      if (this.state.pauseUntil && now >= this.state.pauseUntil) {
        this.resume();
      } else {
        this.state.lastBlockReason = 'paused';
        return false;
      }
    }

    if (this.isInQuietHours(now)) {
      this.state.lastBlockReason = 'quiet_hours';
      return false;
    }

    if (this.isMainWindowActive()) {
      this.state.lastBlockReason = 'main_window_active';
      return false;
    }

    if (this.state.isCardVisible) {
      this.state.lastBlockReason = 'card_visible';
      return false;
    }

    if (this.state.skipCooldownUntil && now < this.state.skipCooldownUntil) {
      this.state.lastBlockReason = 'idle_too_short';
      return false;
    }

    const hasCard = await this.checkHasAvailableCard();
    if (!hasCard) {
      this.state.lastBlockReason = 'no_card';
      return false;
    }

    const idleSeconds = await this.getIdleSeconds();
    if (idleSeconds >= effectiveReminder.idle_threshold_sec) {
      this.state.lastBlockReason = 'ready';
      return true;
    }

    if (effectiveReminder.fallback_enabled && this.state.lastShowTime) {
      const minutesSinceLastShow = (now.getTime() - this.state.lastShowTime.getTime()) / (60 * 1000);
      if (minutesSinceLastShow >= effectiveReminder.fallback_interval_min) {
        this.state.lastBlockReason = 'ready';
        console.log(`⏰ Fallback trigger: ${minutesSinceLastShow.toFixed(1)} minutes since last show`);
        return true;
      }
    }

    this.state.lastBlockReason = 'idle_too_short';
    return false;
  }

  private async getIdleSeconds(): Promise<number> {
    try {
      const seconds = await invoke<number>('get_idle_seconds');
      return seconds;
    } catch (error) {
      console.error('❌ Failed to get idle seconds:', error);
      return 0;
    }
  }

  private async checkHasAvailableCard(): Promise<boolean> {
    try {
      const card = await invoke('get_next_card');
      return card !== null;
    } catch (error) {
      console.error('❌ Failed to check available card:', error);
      return false;
    }
  }

  private isInQuietHours(now: Date): boolean {
    const currentMinutes = now.getHours() * 60 + now.getMinutes();
    const start = this.parseClockToMinutes(this.config.schedule.quiet_hours_start);
    const end = this.parseClockToMinutes(this.config.schedule.quiet_hours_end);

    if (start === end) {
      return false;
    }

    if (start < end) {
      return currentMinutes >= start && currentMinutes < end;
    }

    return currentMinutes >= start || currentMinutes < end;
  }

  private parseClockToMinutes(value: string): number {
    const [hourRaw, minuteRaw] = value.split(':');
    const hour = Number(hourRaw);
    const minute = Number(minuteRaw);

    if (Number.isNaN(hour) || Number.isNaN(minute)) {
      return 0;
    }

    return hour * 60 + minute;
  }

  private isMainWindowActive(): boolean {
    return document.visibilityState === 'visible';
  }
}

export const triggerScheduler = new TriggerScheduler();
