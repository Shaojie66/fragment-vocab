import { invoke } from '@tauri-apps/api/core';

const GITHUB_RELEASES_URL = 'https://api.github.com/repos/Shaojie66/fragment-vocab/releases/latest';
const CHECK_INTERVAL_MS = 24 * 60 * 60 * 1000; // 24 hours
const STORAGE_KEY = 'update_check_last_time';
const DISMISSED_VERSION_KEY = 'update_check_dismissed_version';
const CURRENT_VERSION = '0.1.0';

interface GitHubRelease {
  tag_name: string;
  html_url: string;
  name: string;
  body: string;
}

export interface UpdateInfo {
  available: boolean;
  version?: string;
  url?: string;
  name?: string;
}

function compareVersions(current: string, latest: string): boolean {
  const normalize = (v: string) => v.replace(/^v/, '');
  const currentParts = normalize(current).split('.').map(Number);
  const latestParts = normalize(latest).split('.').map(Number);

  for (let i = 0; i < Math.max(currentParts.length, latestParts.length); i++) {
    const c = currentParts[i] || 0;
    const l = latestParts[i] || 0;
    if (l > c) {
      return true;
    }
    if (l < c) {
      return false;
    }
  }

  return false;
}

function getLastCheckTime(): number {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? Number(raw) : 0;
  } catch {
    return 0;
  }
}

function setLastCheckTime() {
  try {
    localStorage.setItem(STORAGE_KEY, String(Date.now()));
  } catch {
    // ignore
  }
}

function getDismissedVersion(): string | null {
  try {
    return localStorage.getItem(DISMISSED_VERSION_KEY);
  } catch {
    return null;
  }
}

export function dismissVersion(version: string) {
  try {
    localStorage.setItem(DISMISSED_VERSION_KEY, version);
  } catch {
    // ignore
  }
}

export async function checkForUpdate(force = false): Promise<UpdateInfo> {
  if (!force) {
    const lastCheck = getLastCheckTime();
    if (Date.now() - lastCheck < CHECK_INTERVAL_MS) {
      return { available: false };
    }
  }

  try {
    const response = await fetch(GITHUB_RELEASES_URL, {
      headers: { Accept: 'application/vnd.github.v3+json' },
    });

    if (!response.ok) {
      return { available: false };
    }

    const release: GitHubRelease = await response.json();
    setLastCheckTime();

    const latestVersion = release.tag_name.replace(/^v/, '');
    const isNewer = compareVersions(CURRENT_VERSION, latestVersion);

    if (!isNewer) {
      return { available: false };
    }

    const dismissed = getDismissedVersion();
    if (dismissed === latestVersion) {
      return { available: false };
    }

    return {
      available: true,
      version: latestVersion,
      url: release.html_url,
      name: release.name || `v${latestVersion}`,
    };
  } catch {
    return { available: false };
  }
}

export function openReleaseUrl(url: string) {
  void invoke('plugin:opener|open_url', { url }).catch(() => {
    // fallback: open in default browser via window.open
    window.open(url, '_blank');
  });
}
