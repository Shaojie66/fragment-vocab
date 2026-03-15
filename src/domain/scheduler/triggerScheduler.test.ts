// 触发调度器单元测试

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { TriggerScheduler } from './triggerScheduler';
import { createDefaultAppConfig } from '../../shared/config';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

describe('TriggerScheduler', () => {
  let scheduler: TriggerScheduler;
  let visibilityState = 'hidden';

  const createTestConfig = () => ({
    ...createDefaultAppConfig(),
    reminder: {
      ...createDefaultAppConfig().reminder,
      mode: 'custom' as const,
      using_recommended: false,
      idle_threshold_sec: 90,
      fallback_interval_min: 25,
    },
  });

  beforeEach(() => {
    scheduler = new TriggerScheduler(createTestConfig());
    vi.clearAllMocks();
    vi.spyOn(document, 'hasFocus').mockReturnValue(false);
    vi.spyOn(document, 'visibilityState', 'get').mockImplementation(() => visibilityState as DocumentVisibilityState);
    visibilityState = 'hidden';
  });

  afterEach(() => {
    scheduler.stop();
  });

  describe('pause and resume', () => {
    it('暂停后不触发弹卡', async () => {
      scheduler.pause(60);

      vi.mocked(invoke).mockResolvedValue(100);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(false);
    });

    it('暂停过期后自动恢复', async () => {
      const dayTime = new Date('2024-03-12T10:00:00+08:00');
      vi.setSystemTime(dayTime);

      const pauseUntil = new Date(dayTime.getTime() - 1000);
      scheduler.pause(0);
      (scheduler as any).state.pauseUntil = pauseUntil;

      vi.mocked(invoke).mockResolvedValue(100);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(true);

      vi.useRealTimers();
    });

    it('手动恢复后可以触发', async () => {
      scheduler.pause(60);
      scheduler.resume();

      vi.mocked(invoke).mockResolvedValue(100);

      const dayTime = new Date('2024-03-12T10:00:00+08:00');
      vi.setSystemTime(dayTime);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(true);

      vi.useRealTimers();
    });
  });

  describe('night silence', () => {
    it('夜间静默时段（23:00-07:00）不触发', async () => {
      vi.mocked(invoke).mockResolvedValue(100);

      const nightTime = new Date('2024-03-12T02:00:00+08:00');
      vi.setSystemTime(nightTime);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(false);

      vi.useRealTimers();
    });

    it('白天时段可以触发', async () => {
      vi.mocked(invoke).mockResolvedValue(100);

      const dayTime = new Date('2024-03-12T10:00:00+08:00');
      vi.setSystemTime(dayTime);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(true);

      vi.useRealTimers();
    });
  });

  describe('schedule profiles', () => {
    it('工作日使用 weekday profile 的推荐参数', async () => {
      scheduler = new TriggerScheduler({
        ...createDefaultAppConfig(),
        reminder: {
          ...createDefaultAppConfig().reminder,
          mode: 'gentle',
          using_recommended: true,
        },
        schedule: {
          ...createDefaultAppConfig().schedule,
          weekday_profile: 'balanced',
          weekend_profile: 'intensive',
        },
      });

      const weekday = new Date('2024-03-13T10:00:00+08:00');
      vi.setSystemTime(weekday);

      vi.mocked(invoke).mockResolvedValue(120);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(true);
      expect(scheduler.getSnapshot().current_mode).toBe('balanced');

      vi.useRealTimers();
    });

    it('周末使用 weekend profile 的推荐参数', async () => {
      scheduler = new TriggerScheduler({
        ...createDefaultAppConfig(),
        reminder: {
          ...createDefaultAppConfig().reminder,
          mode: 'gentle',
          using_recommended: true,
        },
        schedule: {
          ...createDefaultAppConfig().schedule,
          weekday_profile: 'gentle',
          weekend_profile: 'intensive',
        },
      });

      const weekend = new Date('2024-03-16T10:00:00+08:00');
      vi.setSystemTime(weekend);

      vi.mocked(invoke).mockResolvedValue(95);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(true);
      expect(scheduler.getSnapshot().current_mode).toBe('intensive');

      vi.useRealTimers();
    });
  });

  describe('card visibility', () => {
    it('已有浮卡展示时不触发', async () => {
      scheduler.markCardShown();

      vi.mocked(invoke).mockResolvedValue(100);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(false);
    });

    it('浮卡隐藏后可以触发', async () => {
      scheduler.markCardShown();
      scheduler.markCardHidden();

      vi.mocked(invoke).mockResolvedValue(100);

      const dayTime = new Date('2024-03-12T10:00:00+08:00');
      vi.setSystemTime(dayTime);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(true);

      vi.useRealTimers();
    });
  });

  describe('main window activity', () => {
    it('主页面处于活动状态时不触发浮卡', async () => {
      visibilityState = 'visible';
      vi.mocked(invoke).mockResolvedValue(100);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(false);
      expect(scheduler.getSnapshot().last_block_reason).toBe('main_window_active');
    });
  });

  describe('idle detection', () => {
    it('idle 时间达到阈值（90 秒）时触发', async () => {
      vi.mocked(invoke).mockResolvedValue(90);

      const dayTime = new Date('2024-03-12T10:00:00+08:00');
      vi.setSystemTime(dayTime);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(true);

      vi.useRealTimers();
    });

    it('idle 时间超过阈值时触发', async () => {
      vi.mocked(invoke).mockResolvedValue(120);

      const dayTime = new Date('2024-03-12T10:00:00+08:00');
      vi.setSystemTime(dayTime);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(true);

      vi.useRealTimers();
    });

    it('idle 时间不足时不触发', async () => {
      vi.mocked(invoke).mockResolvedValue(60);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(false);
    });
  });

  describe('fallback trigger', () => {
    it('距离上次展示 >= 45 分钟时兜底触发', async () => {
      const dayTime = new Date('2024-03-12T10:00:00+08:00');
      vi.setSystemTime(dayTime);

      const lastShowTime = new Date(dayTime.getTime() - 50 * 60 * 1000);
      (scheduler as any).state.lastShowTime = lastShowTime;

      vi.mocked(invoke).mockResolvedValue(60);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(true);

      vi.useRealTimers();
    });

    it('距离上次展示 < 45 分钟时不兜底触发', async () => {
      const lastShowTime = new Date(Date.now() - 30 * 60 * 1000);
      (scheduler as any).state.lastShowTime = lastShowTime;

      vi.mocked(invoke).mockResolvedValue(60);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(false);
    });

    it('首次运行（无上次展示时间）不兜底触发', async () => {
      vi.mocked(invoke).mockResolvedValue(60);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(false);
    });
  });

  describe('getIdleSeconds', () => {
    it('成功获取 idle 秒数', async () => {
      vi.mocked(invoke).mockResolvedValue(123.45);

      const seconds = await (scheduler as any).getIdleSeconds();
      expect(seconds).toBe(123.45);
      expect(invoke).toHaveBeenCalledWith('get_idle_seconds');
    });

    it('获取失败时返回 0', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Failed'));

      const seconds = await (scheduler as any).getIdleSeconds();
      expect(seconds).toBe(0);
    });
  });

  describe('综合场景', () => {
    it('满足所有条件时触发', async () => {
      const dayTime = new Date('2024-03-12T10:00:00+08:00');
      vi.setSystemTime(dayTime);

      vi.mocked(invoke).mockResolvedValue(100);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(true);

      vi.useRealTimers();
    });

    it('任一条件不满足时不触发', async () => {
      scheduler.pause(60);

      const dayTime = new Date('2024-03-12T10:00:00+08:00');
      vi.setSystemTime(dayTime);
      vi.mocked(invoke).mockResolvedValue(100);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(false);

      vi.useRealTimers();
    });
  });
});
