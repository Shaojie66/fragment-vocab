import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { triggerScheduler } from '../domain/scheduler/triggerScheduler';
import { mainState } from './state';

interface EventDependencies {
  onSchedulerStateChange: (isStarted: boolean) => void;
  refreshDashboard: () => Promise<void>;
  renderDashboard: () => void;
}

let dependencies: EventDependencies | null = null;
let schedulerStarted = false;
let refreshTimer: number | null = null;
let schedulerUnlistenFns: Array<() => void> = [];

export function initializeEvents(nextDependencies: EventDependencies) {
  dependencies = nextDependencies;

  window.addEventListener('beforeunload', () => {
    stopScheduler();
  });
}

export function startDashboardRefreshPolling() {
  if (refreshTimer !== null || !dependencies) {
    return;
  }

  refreshTimer = window.setInterval(() => {
    void dependencies?.refreshDashboard();
  }, 30000);
}

export function isSchedulerStarted(): boolean {
  return schedulerStarted;
}

export async function initScheduler() {
  if (schedulerStarted || !dependencies) {
    return;
  }

  triggerScheduler.syncPauseUntil(mainState.currentDashboard?.pause_until);
  triggerScheduler.start(async () => {
    await invoke('show_card_window');
    triggerScheduler.markCardShown();
    dependencies?.renderDashboard();
  });

  schedulerUnlistenFns.push(await listen('card-window-hidden', async () => {
    triggerScheduler.markCardHidden();
    await dependencies?.refreshDashboard();
  }));

  schedulerUnlistenFns.push(await listen('scheduler-paused', async (event) => {
    const minutes = Number(event.payload);

    if (!Number.isNaN(minutes) && minutes > 0) {
      triggerScheduler.pause(minutes);
      await dependencies?.refreshDashboard();
    }
  }));

  schedulerStarted = true;
  dependencies.onSchedulerStateChange(true);
}

export function stopScheduler() {
  triggerScheduler.stop();
  schedulerStarted = false;
  schedulerUnlistenFns.splice(0).forEach((unlisten) => {
    unlisten();
  });
  dependencies?.onSchedulerStateChange(false);
}
