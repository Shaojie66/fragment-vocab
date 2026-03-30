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
  isWordbookPreviewLoading: boolean;
  currentWordbookSearchQuery: string;
  currentWordbookSearchOffset: number;
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
  isWordbookPreviewLoading: false,
  currentWordbookSearchQuery: '',
  currentWordbookSearchOffset: 0,
  currentWordbookSearchResults: [],
  isWordbookSearchLoading: false,
  currentExportBundle: null,
  lastErrorMessage: null,
};
