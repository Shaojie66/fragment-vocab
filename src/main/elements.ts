function queryRequiredElement<T extends Element>(selector: string): T {
  const element = document.querySelector<T>(selector);

  if (!element) {
    throw new Error(`Missing required element: ${selector}`);
  }

  return element;
}

export const mainElements = {
  modeSelect: queryRequiredElement<HTMLSelectElement>('#modeSelect'),
  idleThresholdInput: queryRequiredElement<HTMLInputElement>('#idleThresholdInput'),
  fallbackEnabledInput: queryRequiredElement<HTMLInputElement>('#fallbackEnabledInput'),
  fallbackIntervalInput: queryRequiredElement<HTMLInputElement>('#fallbackIntervalInput'),
  dailyNewLimitInput: queryRequiredElement<HTMLInputElement>('#dailyNewLimitInput'),
  reviewFirstInput: queryRequiredElement<HTMLInputElement>('#reviewFirstInput'),
  allowNewWhenNoDueInput: queryRequiredElement<HTMLInputElement>('#allowNewWhenNoDueInput'),
  quietStartInput: queryRequiredElement<HTMLInputElement>('#quietStartInput'),
  quietEndInput: queryRequiredElement<HTMLInputElement>('#quietEndInput'),
  weekdayProfileInput: queryRequiredElement<HTMLSelectElement>('#weekdayProfileInput'),
  weekendProfileInput: queryRequiredElement<HTMLSelectElement>('#weekendProfileInput'),
  autoHideInput: queryRequiredElement<HTMLInputElement>('#autoHideInput'),
  revealOrderSelect: queryRequiredElement<HTMLSelectElement>('#revealOrderSelect'),
  showPhoneticInput: queryRequiredElement<HTMLInputElement>('#showPhoneticInput'),
  allowSkipInput: queryRequiredElement<HTMLInputElement>('#allowSkipInput'),
  shortcutsEnabledInput: queryRequiredElement<HTMLInputElement>('#shortcutsEnabledInput'),
  animationsEnabledInput: queryRequiredElement<HTMLInputElement>('#animationsEnabledInput'),
  autoPronounceInput: queryRequiredElement<HTMLInputElement>('#autoPronounceInput'),
  launchAtLoginInput: queryRequiredElement<HTMLInputElement>('#launchAtLoginInput'),
  startBehaviorSelect: queryRequiredElement<HTMLSelectElement>('#startBehaviorSelect'),
  trayEnabledInput: queryRequiredElement<HTMLInputElement>('#trayEnabledInput'),
  themeSelect: queryRequiredElement<HTMLSelectElement>('#themeSelect'),
  exportLearningDataBtn: queryRequiredElement<HTMLButtonElement>('#exportLearningDataBtn'),
  importLearningDataBtn: queryRequiredElement<HTMLButtonElement>('#importLearningDataBtn'),
  importLearningDataFileInput: queryRequiredElement<HTMLInputElement>('#importLearningDataFileInput'),

  pauseOneHourBtn: queryRequiredElement<HTMLButtonElement>('#pauseOneHourBtn'),
  pauseTodayBtn: queryRequiredElement<HTMLButtonElement>('#pauseTodayBtn'),
  resumeBtn: queryRequiredElement<HTMLButtonElement>('#resumeBtn'),
  openStatsBtn: queryRequiredElement<HTMLButtonElement>('#openStatsBtn'),
  startSchedulerBtn: queryRequiredElement<HTMLButtonElement>('#startSchedulerBtn'),
  stopSchedulerBtn: queryRequiredElement<HTMLButtonElement>('#stopSchedulerBtn'),
  saveConfigBtn: queryRequiredElement<HTMLButtonElement>('#saveConfigBtn'),
  restoreRecommendedBtn: queryRequiredElement<HTMLButtonElement>('#restoreRecommendedBtn'),

  heroSummary: queryRequiredElement<HTMLElement>('#heroSummary'),
  statusChip: queryRequiredElement<HTMLElement>('#statusChip'),
  strategyChip: queryRequiredElement<HTMLElement>('#strategyChip'),
  recommendationChip: queryRequiredElement<HTMLElement>('#recommendationChip'),
  modePill: queryRequiredElement<HTMLElement>('#modePill'),
  recommendationText: queryRequiredElement<HTMLElement>('#recommendationText'),
  recommendationReasonList: queryRequiredElement<HTMLElement>('#recommendationReasonList'),
  saveHint: queryRequiredElement<HTMLElement>('#saveHint'),
  stateBanner: queryRequiredElement<HTMLElement>('#stateBanner'),
  stateBannerTitle: queryRequiredElement<HTMLElement>('#stateBannerTitle'),
  stateBannerBody: queryRequiredElement<HTMLElement>('#stateBannerBody'),
  dailyGoalCard: queryRequiredElement<HTMLElement>('#dailyGoalCard'),
  dailyGoalTrack: queryRequiredElement<HTMLElement>('#dailyGoalTrack'),
  dailyGoalFill: queryRequiredElement<HTMLElement>('#dailyGoalFill'),
  dailyGoalText: queryRequiredElement<HTMLElement>('#dailyGoalText'),
  dailyGoalStatus: queryRequiredElement<HTMLElement>('#dailyGoalStatus'),

  metricTotalReviews: queryRequiredElement<HTMLElement>('#metricTotalReviews'),
  metricAccuracy: queryRequiredElement<HTMLElement>('#metricAccuracy'),
  metricNewWords: queryRequiredElement<HTMLElement>('#metricNewWords'),
  metricDueCards: queryRequiredElement<HTMLElement>('#metricDueCards'),
  metricCurrentStreak: queryRequiredElement<HTMLElement>('#metricCurrentStreak'),

  diagCurrentStatus: queryRequiredElement<HTMLElement>('#diagCurrentStatus'),
  diagBlockReason: queryRequiredElement<HTMLElement>('#diagBlockReason'),
  diagNextReminder: queryRequiredElement<HTMLElement>('#diagNextReminder'),
  diagLastShow: queryRequiredElement<HTMLElement>('#diagLastShow'),

  teamTemplateSelect: queryRequiredElement<HTMLSelectElement>('#teamTemplateSelect'),
  teamTemplateName: queryRequiredElement<HTMLElement>('#teamTemplateName'),
  teamTemplateDescription: queryRequiredElement<HTMLElement>('#teamTemplateDescription'),
  teamTemplateSummary: queryRequiredElement<HTMLElement>('#teamTemplateSummary'),
  applyTemplateBtn: queryRequiredElement<HTMLButtonElement>('#applyTemplateBtn'),

  feedbackTooManyBtn: queryRequiredElement<HTMLButtonElement>('#feedbackTooManyBtn'),
  feedbackTooFewBtn: queryRequiredElement<HTMLButtonElement>('#feedbackTooFewBtn'),
  feedbackList: queryRequiredElement<HTMLElement>('#feedbackList'),

  generateExportBtn: queryRequiredElement<HTMLButtonElement>('#generateExportBtn'),
  copyExportSummaryBtn: queryRequiredElement<HTMLButtonElement>('#copyExportSummaryBtn'),
  copyExportJsonBtn: queryRequiredElement<HTMLButtonElement>('#copyExportJsonBtn'),
  importConfigBtn: queryRequiredElement<HTMLButtonElement>('#importConfigBtn'),
  importConfigFileInput: queryRequiredElement<HTMLInputElement>('#importConfigFileInput'),
  uploadWordbookBtn: queryRequiredElement<HTMLButtonElement>('#uploadWordbookBtn'),
  uploadWordbookFileInput: queryRequiredElement<HTMLInputElement>('#uploadWordbookFileInput'),
  wordbookUploadHint: queryRequiredElement<HTMLElement>('#wordbookUploadHint'),
  wordbookSearchInput: queryRequiredElement<HTMLInputElement>('#wordbookSearchInput'),
  wordbookSearchMeta: queryRequiredElement<HTMLElement>('#wordbookSearchMeta'),
  wordbookSearchResults: queryRequiredElement<HTMLElement>('#wordbookSearchResults'),
  wordbookList: queryRequiredElement<HTMLElement>('#wordbookList'),
  wordbookPreview: queryRequiredElement<HTMLElement>('#wordbookPreview'),
  wordbookPreviewTitle: queryRequiredElement<HTMLElement>('#wordbookPreviewTitle'),
  wordbookPreviewMeta: queryRequiredElement<HTMLElement>('#wordbookPreviewMeta'),
  wordbookPreviewList: queryRequiredElement<HTMLElement>('#wordbookPreviewList'),
  closeWordbookPreviewBtn: queryRequiredElement<HTMLButtonElement>('#closeWordbookPreviewBtn'),
  wordbookPreviewPrevBtn: queryRequiredElement<HTMLButtonElement>('#wordbookPreviewPrevBtn'),
  wordbookPreviewNextBtn: queryRequiredElement<HTMLButtonElement>('#wordbookPreviewNextBtn'),
  downloadExportJsonBtn: queryRequiredElement<HTMLButtonElement>('#downloadExportJsonBtn'),
  exportSummaryOutput: queryRequiredElement<HTMLTextAreaElement>('#exportSummaryOutput'),
  exportJsonOutput: queryRequiredElement<HTMLTextAreaElement>('#exportJsonOutput'),

  onboardingWizard: queryRequiredElement<HTMLElement>('#onboarding-wizard'),
  onboardingPrevBtn: queryRequiredElement<HTMLButtonElement>('#onboardingPrevBtn'),
  onboardingNextBtn: queryRequiredElement<HTMLButtonElement>('#onboardingNextBtn'),
  completeOnboardingBtn: queryRequiredElement<HTMLButtonElement>('#completeOnboardingBtn'),
  onboardingFrequencyInputs: Array.from(
    document.querySelectorAll<HTMLInputElement>('input[name="onboardingFrequency"]'),
  ),
  onboardingStepPanels: Array.from(document.querySelectorAll<HTMLElement>('[data-onboarding-step]')),
  onboardingStepDots: Array.from(document.querySelectorAll<HTMLButtonElement>('[data-onboarding-dot]')),

  tabButtons: Array.from(document.querySelectorAll<HTMLElement>('[data-main-tab]')),
  tabPanels: Array.from(document.querySelectorAll<HTMLElement>('[data-main-panel]')),
};

export type MainElements = typeof mainElements;
