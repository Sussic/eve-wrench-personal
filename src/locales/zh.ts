export default {
  // 通用
  common: {
    close: '关闭',
    loading: '加载中...',
    refresh: '刷新',
    clear: '清除',
    cancel: '取消',
    enabled: '已启用',
    disabled: '已禁用'
  },
  // 对话框
  dialog: {
    confirm: '确认',
    continue: '继续',
    cancel: '取消',
    input: '输入',
    ok: '确定',
    selectEveFolder: '选择EVE设置文件夹',
    copySettings: '复制设置',
    copySettingsDesc: '将设置从"{source}"复制到 {count} 个目标？',
    copy: '复制',
    createBackup: '创建备份',
    createBackupDesc: '为 {name} 输入备份名称',
    backupName: '备份名称',
    create: '创建',
    deleteBackup: '删除备份',
    deleteBackupDesc: '确定要删除"{name}"吗？',
    delete: '删除',
    restoreBackup: '恢复备份',
    restoreBackupDesc: '将"{backup}"恢复到 {target}？这将覆盖当前设置。',
    restore: '恢复',
    applyBackup: '应用备份',
    applyBackupDesc: '将"{backup}"应用到 {target}？这将覆盖当前设置。',
    apply: '应用',
    exportSettings: '导出EVE设置',
    importSettings: '导入EVE设置'
  },
  // 提示消息
  toast: {
    dataRefreshed: '数据已刷新',
    dataRefreshedDesc: '找到 {servers} 个服务器和 {backups} 个备份',
    loadDataFailed: '加载数据失败',
    customPathSet: '自定义路径已设置',
    setPathFailed: '设置路径失败',
    pathReset: '路径已重置',
    pathResetDesc: '使用默认EVE设置位置',
    resetPathFailed: '重置路径失败',
    noSourceSelected: '未选择源',
    noSourceSelectedDesc: '请先选择源再添加目标',
    typeMismatch: '类型不匹配',
    typeMismatchDesc: '目标必须与源类型相同',
    invalidTarget: '无效目标',
    invalidTargetDesc: '不能将同一文件同时作为源和目标',
    settingsCopied: '设置已复制',
    settingsCopiedDesc: '成功复制到 {count} 个目标',
    copyFailed: '复制失败',
    backupCreated: '备份已创建',
    backupCreatedDesc: '"{name}" 已保存',
    backupFailed: '备份失败',
    backupDeleted: '备份已删除',
    backupDeletedDesc: '"{name}" 已移除',
    deleteFailed: '删除失败',
    backupRestored: '备份已恢复',
    backupRestoredDesc: '"{name}" 已应用',
    restoreFailed: '恢复失败',
    backupApplied: '备份已应用',
    backupAppliedDesc: '"{backup}" 已应用到 {target}',
    applyFailed: '应用失败',
    settingsExported: '设置已导出',
    settingsExportedDesc: '已导出 {count} 个文件到 {path}',
    exportFailed: '导出失败',
    importAnalysisFailed: '导入分析失败',
    settingsImported: '设置已导入',
    settingsImportedDesc: '已导入 {imported} 个文件，跳过 {skipped} 个，备份 {backedUp} 个',
    importFailed: '导入失败',
    settingUpdated: '设置已更新',
    settingUpdatedDesc: '始终显示标签 {status}',
    updateSettingFailed: '更新设置失败'
  },
  // 标题栏
  titleBar: {
    settings: '设置',
    backups: '备份',
    toggleTheme: '切换主题'
  },
  // 备份相关
  backup: {
    name: '名称',
    date: '日期'
  },
  // 设置相关
  settings: {
    language: '语言',
    setCustomPath: '设置自定义文件夹路径',
    eveSettingsFolder: 'EVE设置文件夹',
    changeFolder: '更改文件夹...',
    resetToDefault: '重置为默认'
  },
  // 导入/导出
  importExport: {
    import: '导入',
    export: '导出',
    importSettings: '导入设置...',
    exportSettings: '导出设置...',
  },
  // 更新相关
  update: {
    available: '有可用更新',
    download: '下载',
    updateAvailable: '有可用更新',
    newVersionAvailable: 'EVE Wrench 有新版本可用。更新以获取最新功能和错误修复。',
    later: '稍后'
  },
  // 复制面板
  copyPanel: {
    source: '源',
    targets: '目标',
    noSourceSelected: '未选择源',
    noTargetsSelected: '未选择目标',
    copying: '复制中...',
    copySettings: '复制设置'
  },
  // 表格相关
  table: {
    name: '名称',
    modified: '修改时间',
    addAll: '全部添加'
  },
  // 账户/角色列表
  list: {
    accounts: '账户',
    characters: '角色'
  },
  // 操作相关
  actions: {
    source: '源',
    target: '目标',
    createBackup: '创建备份',
    restoreFromBackup: '从备份恢复',
    noBackupsAvailable: '无可用备份',
    useAsSource: '用作源',
    applyTo: '应用到...',
    delete: '删除'
  },
  // 导入对话框
  importDialog: {
    title: '导入设置',
    foundFiles: '在归档中找到 {count} 个文件。查看并选择要覆盖的冲突项。',
    newFiles: '新文件',
    conflicts: '冲突',
    unchanged: '未更改',
    selectAll: '全选',
    deselectAll: '取消全选',
    overwrite: '覆盖'
  },
  // 额外设置
  extra: {
    title: '额外',
    alwaysShowBracketText: '始终显示标签文本',
    alwaysShowBracketTextDesc: '在游戏内所有图标上显示舰船名称，而不仅仅是选中的目标。在视野中有200+飞行员时可能会影响性能。重启EVE客户端后生效。'
  },
  // 空状态
  empty: {
    noEveInstallations: '未找到EVE安装',
    noEveInstallationsDesc: '请确保已安装EVE Online并至少启动过一次。'
  }
}
