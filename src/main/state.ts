import { createDefaultAppConfig } from '../shared/config';
import type {
  AppConfig,
  DashboardState,
  ExportBundle,
  SearchResult,
  TeamTemplate,
  WordbookListItem,
  WordbookWordItem,
} from '../shared/types';

export interface MainState {
  currentConfig: AppConfig;
  currentDashboard: DashboardState | null;
  currentTemplates: TeamTemplate[];
  currentWordbooks: WordbookListItem[];
  currentWordbookPreviewSource: string | null;
  currentWordbookPreviewWords: WordbookWordItem[];
  currentWordbookPreviewOffset: number;
  isWordbookPreviewLoading: boolean;
  currentWordbookSearchQuery: string;
  currentWordbookSearchResults: SearchResult[];
  isWordbookSearchLoading: boolean;
  currentExportBundle: ExportBundle | null;
  lastErrorMessage: string | null;
}

export const mainState: MainState = {
  currentConfig: createDefaultAppConfig(),
  currentDashboard: null,
  currentTemplates: [],
  currentWordbooks: [],
  currentWordbookPreviewSource: null,
  currentWordbookPreviewWords: [],
  currentWordbookPreviewOffset: 0,
  isWordbookPreviewLoading: false,
  currentWordbookSearchQuery: '',
  currentWordbookSearchResults: [],
  isWordbookSearchLoading: false,
  currentExportBundle: null,
  lastErrorMessage: null,
};
