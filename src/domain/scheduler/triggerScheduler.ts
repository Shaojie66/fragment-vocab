// 触发调度器
// 负责定时轮询并判断是否满足弹卡条件

import { invoke } from '@tauri-apps/api/core';

// 调度配置
const POLL_INTERVAL_MS = 15 * 1000; // 15 秒轮询一次
const IDLE_THRESHOLD_SECONDS = 90; // 90 秒空闲阈值
const FALLBACK_TRIGGER_MINUTES = 25; // 25 分钟兜底触发
const NIGHT_SILENCE_START = 23; // 23:00 开始静默
const NIGHT_SILENCE_END = 7; // 07:00 结束静默

// 调度器状态
interface SchedulerState {
  isPaused: boolean;
  pauseUntil?: Date;
  lastShowTime?: Date;
  isCardVisible: boolean;
  skipCooldownUntil?: Date;
}

export class TriggerScheduler {
  private state: SchedulerState = {
    isPaused: false,
    isCardVisible: false,
  };
  
  private intervalId?: number;
  private onTrigger?: () => void;

  /**
   * 启动调度器
   * @param onTrigger 触发弹卡时的回调函数
   */
  start(onTrigger: () => void) {
    this.onTrigger = onTrigger;
    
    // 每 15 秒轮询一次
    this.intervalId = window.setInterval(() => {
      this.checkAndTrigger();
    }, POLL_INTERVAL_MS);
    
    console.log('✅ TriggerScheduler started');
  }

  /**
   * 停止调度器
   */
  stop() {
    if (this.intervalId !== undefined) {
      window.clearInterval(this.intervalId);
      this.intervalId = undefined;
    }
    console.log('⏹️  TriggerScheduler stopped');
  }

  /**
   * 暂停调度（1 小时）
   */
  pause(durationMinutes: number = 60) {
    const pauseUntil = new Date(Date.now() + durationMinutes * 60 * 1000);
    this.state.isPaused = true;
    this.state.pauseUntil = pauseUntil;
    console.log(`⏸️  Paused until ${pauseUntil.toLocaleString()}`);
  }

  /**
   * 恢复调度
   */
  resume() {
    this.state.isPaused = false;
    this.state.pauseUntil = undefined;
    console.log('▶️  Resumed');
  }

  /**
   * 标记卡片已展示
   */
  markCardShown() {
    this.state.lastShowTime = new Date();
    this.state.isCardVisible = true;
  }

  /**
   * 标记卡片已隐藏
   */
  markCardHidden() {
    this.state.isCardVisible = false;
  }

  /**
   * 检查并触发弹卡
   */
  private async checkAndTrigger() {
    try {
      const shouldTrigger = await this.shouldTriggerCard();
      
      if (shouldTrigger && this.onTrigger) {
        console.log('🎯 Triggering card display');
        this.onTrigger();
      }
    } catch (error) {
      console.error('❌ Error in checkAndTrigger:', error);
    }
  }

  /**
   * 判断是否应该弹卡
   */
  private async shouldTriggerCard(): Promise<boolean> {
    const now = new Date();

    // 1. 检查是否已暂停
    if (this.state.isPaused) {
      // 检查暂停是否已过期
      if (this.state.pauseUntil && now >= this.state.pauseUntil) {
        this.resume();
      } else {
        return false;
      }
    }

    // 2. 检查是否在夜间静默时段（23:00-07:00）
    const hour = now.getHours();
    if (hour >= NIGHT_SILENCE_START || hour < NIGHT_SILENCE_END) {
      return false;
    }

    // 3. 检查是否已有浮卡展示
    if (this.state.isCardVisible) {
      return false;
    }

    // 4. 检查是否处于跳过冷却期
    if (this.state.skipCooldownUntil && now < this.state.skipCooldownUntil) {
      return false;
    }

    // 5. 检查是否有可展示的卡片（需要从数据库查询）
    // 这里简化处理，实际应该调用数据库查询
    // const hasCard = await this.checkHasAvailableCard();
    // if (!hasCard) {
    //   return false;
    // }

    // 6. 检查 idle 条件（90 秒）
    const idleSeconds = await this.getIdleSeconds();
    if (idleSeconds >= IDLE_THRESHOLD_SECONDS) {
      return true;
    }

    // 7. 兜底触发：距离上次展示 >= 25 分钟
    if (this.state.lastShowTime) {
      const minutesSinceLastShow = (now.getTime() - this.state.lastShowTime.getTime()) / (60 * 1000);
      if (minutesSinceLastShow >= FALLBACK_TRIGGER_MINUTES) {
        console.log(`⏰ Fallback trigger: ${minutesSinceLastShow.toFixed(1)} minutes since last show`);
        return true;
      }
    }

    return false;
  }

  /**
   * 获取系统 idle 秒数
   */
  private async getIdleSeconds(): Promise<number> {
    try {
      const seconds = await invoke<number>('get_idle_seconds');
      return seconds;
    } catch (error) {
      console.error('❌ Failed to get idle seconds:', error);
      return 0;
    }
  }

  /**
   * 检查是否有可用卡片（简化版，实际需要查询数据库）
   */
  // private async checkHasAvailableCard(): Promise<boolean> {
  //   // TODO: 实现数据库查询
  //   // 这里暂时返回 true，实际应该：
  //   // 1. 查询所有候选卡片
  //   // 2. 调用 hasAvailableCard() 判断
  //   return true;
  // }
}

// 导出单例
export const triggerScheduler = new TriggerScheduler();
