// 媒体 tab（全部/图片/视频/历史）点击切换回归：
// setMediaFilter 只改变量不画界面曾是真 bug——高亮、两侧设置面板、队列过滤都不动。
// 这里用真实切片跑一遍点击流，谁负责重绘必须由这次重绘本身证明。
const assert = require('node:assert/strict');
const vm = require('node:vm');
const {
  slice, source, installQueueCore, installMediaWorkspaceStubs,
} = require('./app-slices.cjs');

function makeEl(tag, dataset = {}) {
  const classes = new Set();
  return {
    tagName: tag, dataset, style: {}, children: [], handlers: {},
    classList: {
      add: (...names) => names.forEach(n => classes.add(n)),
      remove: (...names) => names.forEach(n => classes.delete(n)),
      toggle: (name, force) => {
        const on = force === undefined ? !classes.has(name) : !!force;
        on ? classes.add(name) : classes.delete(name);
        return on;
      },
      contains: name => classes.has(name),
    },
    addEventListener(type, fn) { this.handlers[type] = fn; },
    appendChild(child) { this.children.push(child); return child; },
  };
}

const navButtons = ['all', 'image', 'video', 'history'].map(view => {
  const button = makeEl('button', { mediaView: view });
  if (view === 'all') button.classList.add('active');
  return button;
});
const els = {
  settingsPanel: makeEl('div'), videoSettingsPanel: makeEl('div'),
  queuePanel: makeEl('div'), fileQueueList: makeEl('div'),
  queueFailedOnly: { checked: false }, queueSortKey: { value: 'import' },
  queueSortDirection: { value: 'asc' }, queueSummary: makeEl('div'),
  historyViewBtn: makeEl('button'), settingsViewBtn: makeEl('button'),
  mainView: makeEl('div'), historyView: makeEl('div'), settingsView: makeEl('div'),
};
const context = vm.createContext({
  console, Set, Map, Promise,
  files: [], inputPaths: [], queueItems: new Map(), fileRows: {},
  currentView: 'main', historyFilter: 'all', processingMode: 'advanced',
  settingsPanel: els.settingsPanel, videoSettingsPanel: els.videoSettingsPanel,
  pendingAutoCompress: false, queueRevision: 0, executionSession: null,
  localizeUiText: value => value,
  document: {
    getElementById: id => els[id] || null,
    querySelectorAll: selector => selector === '[data-media-view]' ? navButtons : [],
    querySelector: () => null,
  },
  refreshHistory: () => {}, loadRetentionSetting: () => {}, loadCpuSetting: () => {}, initUpdatePanel: () => {},
});
installMediaWorkspaceStubs(context);
installQueueCore(context);
// applyQueueView 在「队列渲染」段（queue-view.cjs 已覆盖它的过滤行为），
// 这里只关心切换时它有没有被真的调用。
const applyQueueViewCalls = [];
context.applyQueueView = () => { applyQueueViewCalls.push(context.mediaFilter); };
// 被测代码：媒体过滤 + 视图切换，全部是 app.js 原文。
vm.runInContext(slice('function setMediaFilter(', '// ─── 队列状态'), context);
vm.runInContext(slice('function showView(', '// ─── 压缩状态机'), context);
vm.runInContext('var VIEWS = ["main", "history", "settings"];', context);

const activeOf = view => navButtons.find(b => b.dataset.mediaView === view).classList.contains('active');

(async () => {
  assert.ok(activeOf('all') && !activeOf('video'), '初始高亮在「全部」');

  context.setMediaFilter('video');
  assert.equal(context.mediaFilter, 'video', '点击后内部过滤值必须变化');
  assert.ok(activeOf('video') && !activeOf('all'), '高亮必须跟到「视频」');
  assert.equal(els.videoSettingsPanel.style.display, 'block', '视频设置面板必须显示');
  assert.equal(els.settingsPanel.style.display, 'none', '图片设置面板必须隐藏');
  assert.equal(els.queuePanel.style.display, 'none', '空队列时面板照实隐藏（updateMediaWorkspace 的真实行为）');
  assert.deepEqual(applyQueueViewCalls, ['video'], '切换必须触发队列视图重绘');
  assert.equal(context.currentView, 'main', '停留在主视图');

  context.setMediaFilter('image');
  assert.ok(activeOf('image') && !activeOf('video'), '切「图片」高亮跟随');
  assert.equal(els.settingsPanel.style.display, 'block', '图片设置面板显示');
  assert.equal(els.videoSettingsPanel.style.display, 'none', '视频设置面板隐藏');
  assert.equal(applyQueueViewCalls[applyQueueViewCalls.length - 1], 'image', '每次切换都触发重绘');

  context.showView('history');
  assert.equal(context.currentView, 'history');
  assert.ok(activeOf('history'), '进入历史页时媒体导航的「历史」高亮');
  assert.ok(!activeOf('image'), '其他 tab 不再高亮');

  context.setMediaFilter('all');
  assert.equal(context.currentView, 'main', '切过滤自动回主视图');
  assert.ok(activeOf('all') && !activeOf('history'), '历史高亮退位给「全部」');
  assert.equal(applyQueueViewCalls[applyQueueViewCalls.length - 1], 'all', '回到全部同样触发重绘');

  console.log('PASS: 媒体 tab 点击 → 高亮/两侧面板/主视图全部跟着切');
})().catch(error => { console.error(error); process.exit(1); });
