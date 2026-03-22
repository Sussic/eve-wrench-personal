export default {
  // Common
  common: {
    close: 'Close',
    loading: 'Loading...',
    refresh: 'Refresh',
    clear: 'Clear',
    cancel: 'Cancel',
    enabled: 'enabled',
    disabled: 'disabled'
  },
  // Dialog
  dialog: {
    confirm: 'Confirm',
    continue: 'Continue',
    cancel: 'Cancel',
    input: 'Input',
    ok: 'OK',
    selectEveFolder: 'Select EVE Settings Folder',
    copySettings: 'Copy Settings',
    copySettingsDesc: 'Copy settings from "{source}" to {count} target(s)?',
    copy: 'Copy',
    createBackup: 'Create Backup',
    createBackupDesc: 'Enter a name for this backup of {name}',
    backupName: 'Backup name',
    create: 'Create',
    deleteBackup: 'Delete Backup',
    deleteBackupDesc: 'Are you sure you want to delete "{name}"?',
    delete: 'Delete',
    restoreBackup: 'Restore Backup',
    restoreBackupDesc: 'Restore "{backup}" to {target}? This will overwrite current settings.',
    restore: 'Restore',
    applyBackup: 'Apply Backup',
    applyBackupDesc: 'Apply "{backup}" to {target}? This will overwrite current settings.',
    apply: 'Apply',
    exportSettings: 'Export EVE Settings',
    importSettings: 'Import EVE Settings'
  },
  // Toast messages
  toast: {
    dataRefreshed: 'Data refreshed',
    dataRefreshedDesc: 'Found {servers} server(s) and {backups} backup(s)',
    loadDataFailed: 'Failed to load data',
    customPathSet: 'Custom path set',
    setPathFailed: 'Failed to set path',
    pathReset: 'Path reset',
    pathResetDesc: 'Using default EVE settings location',
    resetPathFailed: 'Failed to reset path',
    noSourceSelected: 'No source selected',
    noSourceSelectedDesc: 'Select a source first before adding targets',
    typeMismatch: 'Type mismatch',
    typeMismatchDesc: 'Target must be the same type as source',
    invalidTarget: 'Invalid target',
    invalidTargetDesc: 'Cannot use the same file as source and target',
    settingsCopied: 'Settings copied',
    settingsCopiedDesc: 'Successfully copied to {count} target(s)',
    copyFailed: 'Copy failed',
    backupCreated: 'Backup created',
    backupCreatedDesc: '"{name}" has been saved',
    backupFailed: 'Backup failed',
    backupDeleted: 'Backup deleted',
    backupDeletedDesc: '"{name}" has been removed',
    deleteFailed: 'Delete failed',
    backupRestored: 'Backup restored',
    backupRestoredDesc: '"{name}" has been applied',
    restoreFailed: 'Restore failed',
    backupApplied: 'Backup applied',
    backupAppliedDesc: '"{backup}" has been applied to {target}',
    applyFailed: 'Apply failed',
    settingsExported: 'Settings exported',
    settingsExportedDesc: 'Exported {count} file(s) to {path}',
    exportFailed: 'Export failed',
    importAnalysisFailed: 'Import analysis failed',
    settingsImported: 'Settings imported',
    settingsImportedDesc: 'Imported {imported} file(s), skipped {skipped}, backed up {backedUp}',
    importFailed: 'Import failed',
    settingUpdated: 'Setting updated',
    settingUpdatedDesc: 'Brackets always show {status}',
    updateSettingFailed: 'Failed to update setting'
  },
  // Title Bar
  titleBar: {
    settings: 'Settings',
    backups: 'Backups',
    toggleTheme: 'Toggle theme'
  },
  // Backup related
  backup: {
    name: 'Name',
    date: 'Date'
  },
  // Settings related
  settings: {
    language: 'Language',
    setCustomPath: 'Set custom folder path',
    eveSettingsFolder: 'EVE Settings Folder',
    changeFolder: 'Change folder...',
    resetToDefault: 'Reset to default'
  },
  // Import/Export
  importExport: {
    import: 'Import',
    export: 'Export',
    importSettings: 'Import settings...',
    exportSettings: 'Export settings...',
  },
  // Update related
  update: {
    available: 'Update Available',
    download: 'Download',
    updateAvailable: 'Update Available',
    newVersionAvailable: 'A new version of EVE Wrench is available. Update to get the latest features and bug fixes.',
    later: 'Later'
  },
  // Copy Panel
  copyPanel: {
    source: 'Source',
    targets: 'Targets',
    noSourceSelected: 'No source selected',
    noTargetsSelected: 'No targets selected',
    copying: 'Copying...',
    copySettings: 'Copy Settings'
  },
  // Table related
  table: {
    name: 'Name',
    modified: 'Modified',
    addAll: 'Add all'
  },
  // Account/Character list
  list: {
    accounts: 'Accounts',
    characters: 'Characters'
  },
  // Actions
  actions: {
    source: 'Source',
    target: 'Target',
    createBackup: 'Create backup',
    restoreFromBackup: 'Restore from backup',
    noBackupsAvailable: 'No backups available',
    useAsSource: 'Use as source',
    applyTo: 'Apply to...',
    delete: 'Delete'
  },
  // Import Dialog
  importDialog: {
    title: 'Import Settings',
    foundFiles: 'Found {count} file(s) in the archive. Review and select which conflicts to overwrite.',
    newFiles: 'New files',
    conflicts: 'Conflicts',
    unchanged: 'Unchanged',
    selectAll: 'Select all',
    deselectAll: 'Deselect all',
    overwrite: 'overwrite'
  },
  // Extra settings
  extra: {
    title: 'Extra',
    alwaysShowBracketText: 'Always Show Bracket Text',
    alwaysShowBracketTextDesc: 'Show ship labels on all brackets in space, not just selected targets. May impact performance with 200+ pilots on grid. Requires client restart.'
  },
  // Empty state
  empty: {
    noEveInstallations: 'No EVE Installations Found',
    noEveInstallationsDesc: 'Make sure EVE Online is installed and has been launched at least once.'
  }
}
