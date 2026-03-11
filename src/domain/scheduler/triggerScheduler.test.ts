// 触发调度器单元测试

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { TriggerScheduler } from './triggerScheduler';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

describe('TriggerScheduler', () => {
  let scheduler: TriggerScheduler;

  beforeEach(() => {
    scheduler = new TriggerScheduler();
    vi.clearAllMocks();
  });

  afterEach(() => {
    scheduler.stop();
  });

  describe('pause and resume', () => {
    it('暂停后不触发弹卡', async () => {
      scheduler.pause(60);
      
      // Mock idle 检测返回足够的空闲时间
      vi.mocked(invoke).mockResolvedValue(100);

      // 手动调用检查逻辑（绕过定时器）
      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(false);
    });

    it('暂停过期后自动恢复', async () => {
      // 设置白天时段
      const dayTime = new Date('2024-03-12T10:00:00+08:00');
      vi.setSystemTime(dayTime);
      
      // 暂停到 1 秒前（已过期）
      const pauseUntil = new Date(dayTime.getTime() - 1000);
      scheduler.pause(0);
      (scheduler as any).state.pauseUntil = pauseUntil;
      
      vi.mocked(invoke).mockResolvedValue(100);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(true); // 暂停已过期，应该触发
      
      vi.useRealTimers();
    });

    it('手动恢复后可以触发', async () => {
      scheduler.pause(60);
      scheduler.resume();
      
      vi.mocked(invoke).mockResolvedValue(100);
      
      // 设置白天时段
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

      // Mock 当前时间为凌晨 2 点（本地时间）
      const nightTime = new Date('2024-03-12T02:00:00+08:00');
      vi.setSystemTime(nightTime);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(false);

      vi.useRealTimers();
    });

    it('白天时段可以触发', async () => {
      vi.mocked(invoke).mockResolvedValue(100);

      // Mock 当前时间为上午 10 点（本地时间）
      const dayTime = new Date('2024-03-12T10:00:00+08:00');
      vi.setSystemTime(dayTime);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(true);

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
      
      // 设置白天时段
      const dayTime = new Date('2024-03-12T10:00:00+08:00');
      vi.setSystemTime(dayTime);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(true);
      
      vi.useRealTimers();
    });
  });

  describe('idle detection', () => {
    it('idle 时间达到阈值（90 秒）时触发', async () => {
      vi.mocked(invoke).mockResolvedValue(90);
      
      // 设置白天时段
      const dayTime = new Date('2024-03-12T10:00:00+08:00');
      vi.setSystemTime(dayTime);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(true);
      
      vi.useRealTimers();
    });

    it('idle 时间超过阈值时触发', async () => {
      vi.mocked(invoke).mockResolvedValue(120);
      
      // 设置白天时段
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
    it('距离上次展示 >= 25 分钟时兜底触发', async () => {
      // 设置白天时段
      const dayTime = new Date('2024-03-12T10:00:00+08:00');
      vi.setSystemTime(dayTime);
      
      // 设置上次展示时间为 30 分钟前
      const lastShowTime = new Date(dayTime.getTime() - 30 * 60 * 1000);
      (scheduler as any).state.lastShowTime = lastShowTime;
      
      // idle 时间不足
      vi.mocked(invoke).mockResolvedValue(60);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(true);
      
      vi.useRealTimers();
    });

    it('距离上次展示 < 25 分钟时不兜底触发', async () => {
      // 设置上次展示时间为 20 分钟前
      const lastShowTime = new Date(Date.now() - 20 * 60 * 1000);
      (scheduler as any).state.lastShowTime = lastShowTime;
      
      // idle 时间不足
      vi.mocked(invoke).mockResolvedValue(60);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(false);
    });

    it('首次运行（无上次展示时间）不兜底触发', async () => {
      // 不设置 lastShowTime
      
      // idle 时间不足
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
      // 未暂停
      // 白天时段
      const dayTime = new Date('2024-03-12T10:00:00+08:00');
      vi.setSystemTime(dayTime);
      
      // 无浮卡展示
      // idle 时间足够
      vi.mocked(invoke).mockResolvedValue(100);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(true);

      vi.useRealTimers();
    });

    it('任一条件不满足时不触发', async () => {
      // 暂停中
      scheduler.pause(60);
      
      // 其他条件都满足
      const dayTime = new Date('2024-03-12T10:00:00+08:00');
      vi.setSystemTime(dayTime);
      vi.mocked(invoke).mockResolvedValue(100);

      const shouldTrigger = await (scheduler as any).shouldTriggerCard();
      expect(shouldTrigger).toBe(false);

      vi.useRealTimers();
    });
  });
});
