// 媒体过滤 chips 与左侧导航的点击流回归。
//
// 旧版把「全部/图片/视频/历史」做成同一排 tab，历史是一个媒体类别、又是一个页面，
// 两件事挤在一条控件上。新 IA 把它们分开：
//   · 顶部 chips 只在**本轮内部**过滤（all / image / video）；
//   · 左侧 rail 才负责换页（本轮 / 历史 / 设置），历史页自己另有一组过滤 chips。
// 曾被真实咬到过的是同一件事的另一个面：setXxx 只改变量不画界面。所以这里全部
// 用 app.js 原文跑一遍点击流 —— 谁负责重绘，由这次重绘本身证明。
const assert = require('node:assert/strict');
const vm = require('node:vm');
const {
  slice, installQueueCore, installMediaWorkspaceStubs,
} = require('./app-slices.cjs');

function makeEl(tag, dataset = {}) {
  const classes = new Set();
  return {
    tagName: tag, dataset, style: {}, children: [], handlers: {}, attrs: {},
    hidden: false, textContent: '',
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
    setAttribute(key, value) { this.attrs[key] = value; },
    removeAttribute(key) { delete this.attrs[key]; },
    querySelector: () => null,
  };
}

// 顶部媒体过滤 chips（只过滤，不换页）。
const filterChips = ['all', 'image', 'video'].map(view => {
  const button = makeEl('button', { mediaFilter: view });
  if (view === 'all') button.classList.add('active');
  return button;
});
// 左侧导航（换页）。
const railButtons = ['workspace', 'history', 'settings'].map(key => {
  const button = makeEl('button', { rail: key });
  if (key === 'workspace') button.classList.add('active');
  return button;
});
const els = {
  imageIntentGroup: makeEl('div'), videoIntentGroup: makeEl('div'),
  fileQueueList: makeEl('div'), queueSummary: makeEl('div'),
  mainView: makeEl('div'), historyView: makeEl('div'), settingsView: makeEl('div'),
  titlebarContext: makeEl('span'),
};
const context = vm.createContext({
  console, Set, Map, Promise,
  files: [], inputPaths: [], queueItems: new Map(), fileRows: {},
  currentView: 'main', mediaFilter: 'all', historyFilter: 'all',
  pendingAutoCompress: false, queueRevision: 0, executionSession: null,
  document: {
    getElementById: id => els[id] || null,
    querySelectorAll: selector => {
      if (selector === '#mediaFilterChips [data-media-filter]') return filterChips;
      if (selector === '.rail-button') return railButtons;
      return [];
    },
    querySelector: () => null,
  },
  refreshHistory: () => { context._historyRefreshed = (context._historyRefreshed || 0) + 1; },
  loadRetentionSetting: () => {}, loadCpuSetting: () => {}, initUpdatePanel: () => {},
});
installMediaWorkspaceStubs(context);
installQueueCore(context);
// applyQueueView 在「队列渲染」段（queue-view.cjs 已覆盖它的过滤行为），
// 这里只关心切换时它有没有被真的调用。
const applyQueueViewCalls = [];
context.applyQueueView = () => { applyQueueViewCalls.push(context.mediaFilter); };
// 被测代码：媒体过滤 + 页面导航，全部是 app.js 原文。
vm.runInContext(slice('function setMediaFilter(', '// ─── 队列状态'), context);
vm.runInContext(slice('var VIEWS = ', '// ─── 压缩状态机'), context);

const chipOn = view => filterChips.find(b => b.dataset.mediaFilter === view).classList.contains('active');
const railOn = key => railButtons.find(b => b.dataset.rail === key).classList.contains('active');

/// 往队列里放几张图 / 几段视频：意图组的收放看的就是这个比例。
const seed = (images, videos) => {
  context.files = [];
  context.queueItems = new Map();
  Array.from({ length: images }, (_, i) => `/a-${i}.png`).forEach(path => {
    context.files.push(path);
    context.queueItems.set(path, { path, mediaKind: 'image', state: 'pending', result: null });
  });
  Array.from({ length: videos }, (_, i) => `/v-${i}.mp4`).forEach(path => {
    context.files.push(path);
    context.queueItems.set(path, { path, mediaKind: 'video', state: 'pending', result: null });
  });
};

(async () => {
  assert.ok(chipOn('all') && !chipOn('video'), '初始过滤在「全部」');
  assert.ok(railOn('workspace') && !railOn('history'), '初始页面是本轮');

  // ── 只有视频的一轮：图片目标组是装饰，收起 ────────────────────
  seed(0, 2);
  context.setMediaFilter('video');
  assert.equal(context.mediaFilter, 'video', '点击后过滤值必须变化');
  assert.ok(chipOn('video') && !chipOn('all'), '高亮必须跟到「视频」');
  assert.equal(els.videoIntentGroup.hidden, false, '视频目标组在视频过滤下显示');
  assert.equal(els.imageIntentGroup.hidden, true, '图片目标组让位');
  assert.equal(context.currentView, 'main', '过滤 chips 不换页');
  assert.deepEqual(applyQueueViewCalls, ['video'], '切换必须触发队列视图重绘');
  // aria-pressed 是这组 chips 的语义状态，不能只靠 class。
  assert.equal(filterChips.find(b => b.dataset.mediaFilter === 'video').attrs['aria-pressed'], 'true');
  assert.equal(filterChips.find(b => b.dataset.mediaFilter === 'all').attrs['aria-pressed'], 'false');

  // ── 切「图片」：反过来，且每一次切换都重绘 ────────────────────
  context.setMediaFilter('image');
  assert.ok(chipOn('image') && !chipOn('video'), '切「图片」高亮跟随');
  assert.equal(els.imageIntentGroup.hidden, false);
  assert.equal(els.videoIntentGroup.hidden, true, '视频目标组在图片过滤下收起');
  assert.equal(applyQueueViewCalls[applyQueueViewCalls.length - 1], 'image', '每次切换都触发重绘');

  // ── 混合的一轮 + 「全部」：两组都留着，因为两种都真要用 ────────
  seed(3, 1);
  context.setMediaFilter('all');
  assert.equal(els.imageIntentGroup.hidden, false, '有图片就得能选图片目标');
  assert.equal(els.videoIntentGroup.hidden, false, '有视频就得能选视频目标');

  // 只有图片时，视频目标组同样不该摆着。
  seed(3, 0);
  context.setMediaFilter('all');
  assert.equal(els.imageIntentGroup.hidden, false);
  assert.equal(els.videoIntentGroup.hidden, true, '队列里没有视频，视频目标组就是假控件');

  // ── 换页归左侧导航：只管 pane 的 hidden 与高亮，不动过滤值 ────
  context.showView('history');
  assert.equal(context.currentView, 'history');
  assert.ok(railOn('history') && !railOn('workspace'), '进入历史页时导航高亮跟随');
  assert.equal(railButtons.find(b => b.dataset.rail === 'history').attrs['aria-current'], 'page');
  assert.ok(!('aria-current' in railButtons.find(b => b.dataset.rail === 'workspace').attrs));
  assert.equal(els.mainView.hidden, true, '本轮收起来');
  assert.equal(els.historyView.hidden, false);
  assert.equal(context._historyRefreshed, 1, '进历史页必须重新读盘');
  assert.equal(context.mediaFilter, 'all', '换页不碰本轮的媒体过滤');

  context.showView('settings');
  assert.equal(els.historyView.hidden, true, '离开历史页要把它收起来');
  assert.equal(els.settingsView.hidden, false);

  // 过滤 chips 永远把人带回本轮（在历史页点它不该把历史页过滤成空的）。
  context.setMediaFilter('image');
  assert.equal(context.currentView, 'main', '点媒体过滤自动回本轮');
  assert.ok(railOn('workspace') && !railOn('settings'), '导航高亮跟着换回本轮');
  assert.equal(applyQueueViewCalls[applyQueueViewCalls.length - 1], 'image');

  // 非法值不落进变量：它会把两组意图组都藏起来，界面就成了没有控件的界面。
  context.setMediaFilter('nonsense');
  assert.equal(context.mediaFilter, 'all', '过滤值只有三种，认不出一律回「全部」');
  assert.equal(els.titlebarContext.textContent, '媒体工作台', '标题栏那行跟着真实页面走');

  console.log('PASS: 过滤 chips 只管本轮内过滤、意图组跟着队列成分收放、换页归左侧导航且每次都真的重绘');
})().catch(error => { console.error(error); process.exit(1); });
