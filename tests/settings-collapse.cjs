// 「更多参数」默认收起，但收起的那一行必须真有话说；意图与落盘是一对二。
//
// 旧版是一屏可折叠的"压缩设置"面板，参数是主人、意图是它的别名。新 IA 反过来：
// chips 是唯一的入口，原始参数只是意图的展开形式，默认收在 <details> 里。四条不变量：
//   ① 起手收起（HTML 写死 open 缺席），且没有任何代码路径替用户展开它；
//   ② 收起时标题行仍在报"现在会跑什么"（意图摘要），不是个空壳；
//   ③ 意图接管期间，不属于这个意图的原始控件不许"看得见但没人读"；
//      切到自定义时接管提示消失、控件铺的是此刻真正生效的那套值；
//   ④ 意图与它的原始字段一起落盘、一起读回；读回的意图必须在**当前平台**可选列表里。
const assert = require('node:assert/strict');
const vm = require('node:vm');
const fs = require('node:fs');
const {
  source, slice, installIntentModel, installSystemMode,
} = require('./app-slices.cjs');

const html = fs.readFileSync('frontend/index.html', 'utf8');
const css = fs.readFileSync('frontend/style.css', 'utf8');

// ── ① 默认收起：写在 HTML 里，不靠 JS 事后补 ──────────────────────────
const detailsTag = html.match(/<details[^>]*id="intentAdvanced"[^>]*>/);
assert.ok(detailsTag, '找不到「更多参数」#intentAdvanced');
assert.ok(!/\bopen\b/.test(detailsTag[0]),
  '#intentAdvanced 必须不带 open 起手，否则一进来又是一屏参数顶在队列前面');
assert.doesNotMatch(source, /\.open\s*=\s*true/,
  '不许有任何代码路径替用户展开 details（"默认收起"会在那些路径上悄悄失效）');
assert.ok(css.includes('.intent-advanced'), '收起的样式在位：details 没有 CSS 就等于没设计');

// ── 假 DOM：只给这一节真会碰到的格子 ──────────────────────────────────
// innerHTML 不解析：真实代码用 querySelector 往格子上写字段，所以按 class 找、
// 找不到就造一个，并且真的挂进 children —— 「接管提示加了又撤掉」这条断言
// 依赖的就是它能被找到、也能真的消失。
function el(tag, extra) {
  const node = {
    tagName: tag, id: (extra && extra.id) || '', children: [], parts: {}, dataset: {},
    style: {}, attrs: {}, className: '', title: '', hidden: false, value: '', checked: false,
    textContent: '', innerHTML: '', handlers: {}, _parent: null,
    classList: {
      _set: new Set(),
      add(...n) { n.forEach(x => this._set.add(x)); },
      remove(...n) { n.forEach(x => this._set.delete(x)); },
      toggle(n, force) { (force === undefined ? !this._set.has(n) : !!force) ? this._set.add(n) : this._set.delete(n); },
      contains(n) { return this._set.has(n); },
    },
    appendChild(c) { c._parent = node; node.children.push(c); return c; },
    insertBefore(c) { c._parent = node; node.children.unshift(c); return c; },
    setAttribute(k, v) { node.attrs[k] = v; },
    addEventListener(t, fn) { (node.handlers[t] = node.handlers[t] || []).push(fn); },
    querySelector(sel) {
      // 找不到就返回 null —— 真实代码靠这个区分"提示已经在了"和"还没加过"。
      if (sel[0] === '.') {
        const cls = sel.slice(1);
        return node.children.find(c => (c.className || '').split(/\s+/).includes(cls)) || null;
      }
      return node.parts[sel] || null;
    },
    remove() {
      if (!node._parent) return;
      const kids = node._parent.children;
      const at = kids.indexOf(node);
      if (at >= 0) kids.splice(at, 1);
      node._parent = null;
    },
  };
  Object.defineProperty(node, 'firstChild', { get: () => node.children[0] || null });
  return node;
}

const ids = {};
['imageIntentChips', 'videoIntentChips', 'imageIntentNote', 'videoIntentNote',
  'imageIntentSummary', 'videoIntentSummary', 'intentTargetRow', 'targetBytesField',
  'targetDimField', 'targetMiBField', 'imageTargetKib', 'imageDimCap', 'videoTargetMiBInput',
  'intentAdvanced', 'imageIntentGroup', 'videoIntentGroup', 'imageAdvancedRows',
  'videoAdvancedRows', 'systemInfoBtn', 'autoCompressLabel', 'autoCompress',
  'outputFormat', 'systemOutputFormat', 'systemImageSize', 'preserveMetadata',
].forEach(id => { ids[id] = el('div', { id }); });
ids.imageIntentGroup.hidden = false;
ids.videoIntentGroup.hidden = true;
ids.autoCompress.checked = false;
ids.imageTargetKib.value = '';
ids.imageDimCap.value = '';

// 原始控件行：data-intent-raw 决定它归哪个意图管。
const rawRows = {
  custom: el('div'), system: el('div'),
};
rawRows.custom.dataset.intentRaw = 'image';
rawRows.system.dataset.intentRaw = 'system';

const qualitySlider = el('input'); qualitySlider.value = '75';
const qualityValue = el('span');
const store = {};
const context = vm.createContext({
  console, Set, Map, Promise, JSON, Math, Number, String, Array, Object, parseFloat, isNaN,
  document: {
    getElementById: id => ids[id] || (id === 'qualitySlider' ? qualitySlider : id === 'qualityValue' ? qualityValue : null),
    createElement: tag => el(tag),
    querySelector: () => ({ value: 'replace' }),
    querySelectorAll: sel => (sel === '[data-intent-raw]' ? Object.values(rawRows) : []),
  },
  window: {
    SizqoI18n: { translateText: v => v },
    SizqoVideo: { DEFAULT_OPTIONS: {}, normalizeOptions: o => Object.assign({}, o) },
  },
  localStorage: {
    getItem: k => (k in store ? store[k] : null),
    setItem: (k, v) => { store[k] = String(v); },
    removeItem: k => { delete store[k]; },
  },
  location: { reload() { context._reloaded = true; } },
  navigator: { userAgent: 'Mozilla/5.0 (Macintosh)' },
  updateQualitySlider, qualitySlider, qualityValue,
  iconMarkup: name => '<svg class="symbol-icon"><use href="#icon-' + name + '"/></svg>',
  outputSuffixInput: el('input'),
  currentOutputMode: () => 'replace',
  getOutputSuffix: () => '_compressed',
  applyVideoOptions() {}, renderRunBar() {}, renderStartButton() {}, renderDetailPane() {},
});
function updateQualitySlider() { qualityValue.textContent = qualitySlider.value + '%'; }

installIntentModel(context);
installSystemMode(context);
// 跑的就是「意图条」这一节真实代码：chips、接管提示都在里面。
vm.runInContext(slice('// ─── 意图条', '// ─── Drag and drop'), context);
// 落盘与读回在同一节末尾：意图变了要立刻存，读回来要立刻铺成它跑得动的值。
vm.runInContext(slice('function saveCompressSettings(', '// Init'), context);
// 落盘与读回在文件末尾那一节 —— 意图变了要跟着它一起存，这里也装真的那段。
vm.runInContext(slice('function saveCompressSettings(', '// Init'), context);
context.isMacOS = () => true;

(async () => {
  // ── ② 收起的那一行真的在说话 ────────────────────────────────────────
  context.imageIntent.intent = 'balanced';
  context.renderIntentBar();
  assert.equal(ids.imageIntentSummary.textContent, '均衡 · Q75 · 原格式',
    '折叠之上那行摘要就是用户唯一要读的结论，不能是空的');
  assert.match(ids.imageIntentNote.textContent, /EXIF/,
    '有代价的意图把代价写在同一行，不藏在展开之后');

  // ── ③ 接管提示 + 不相关的原始行不显示 ───────────────────────────────
  const lockNote = () => ids.imageAdvancedRows.children
    .find(child => (child.className || '').includes('advanced-lock-note'));
  assert.ok(lockNote(), '均衡意图下必须写明这些参数此刻归谁');
  assert.match(lockNote().textContent, /当前由「均衡」接管/, '提示要说清是哪个意图接管');
  assert.equal(rawRows.custom.hidden, true, '不是自定义时，自定义的原始控件不该摆在那儿');
  assert.equal(rawRows.system.hidden, true, '不是系统转换时，系统转换的控件同样收起');

  // 切到自定义：接管提示消失，控件铺的是**此刻真正生效**的那套值。
  context.setImageIntent('custom');
  assert.equal(context.imageIntent.intent, 'custom');
  assert.ok(!lockNote(), '自定义之下没有意图接管，提示必须撤掉');
  assert.equal(rawRows.custom.hidden, false, '自定义时它的原始控件才该出现');
  assert.equal(rawRows.system.hidden, true);
  assert.equal(qualitySlider.value, '75');

  // 从预算模式接手：编译出来的质量是 0（"由阶梯决定"），滑块上留 0 是个假刻度。
  context.setImageIntent('targetSize');
  assert.equal(context.imageRequestFromIntent().options.quality, 0, '预算模式下质量归阶梯，不是用户给的数');
  context.setImageIntent('custom');
  assert.ok(Number(qualitySlider.value) > 0, '接手的是能用的数，不是预算模式那个 0');
  assert.equal(ids.imageTargetKib.value, String(Math.round(
    context.imageRequestFromIntent().options.maxOutputBytes / 1024)),
    '自定义里那格体积就是刚被接管的那预算');

  // ── ④ 落盘 ↔ 读回是一对二 ──────────────────────────────────────────
  context.setImageIntent('webSize');
  context.imageIntent.maxDimensionPx = 1800;
  context.videoIntent.intent = 'videoTarget';
  context.videoIntent.targetMiB = 64;
  ids.autoCompress.checked = true;
  context.outputSuffixInput.value = '_min';
  context.saveCompressSettings();
  const saved = JSON.parse(store['sizqo-settings']);
  assert.equal(saved.version, 2);
  assert.equal(saved.imageIntent.intent, 'webSize');
  assert.equal(saved.imageIntent.maxDimensionPx, 1800, '意图和被它接手的原始字段一起存');
  assert.equal(saved.videoIntent.targetMiB, 64);
  assert.equal(saved.autoCompress, true);
  assert.equal(saved.outputSuffix, '_min');

  // 换一台"刚启动"的机器：清掉内存态，只留落盘那份。
  context.imageIntent.intent = 'balanced';
  context.imageIntent.maxDimensionPx = null;
  context.videoIntent.intent = 'videoSmall';
  context.videoIntent.targetMiB = 100;
  ids.autoCompress.checked = false;
  context.outputSuffixInput.value = '';
  context.loadCompressSettings();
  assert.equal(context.imageIntent.intent, 'webSize', '意图回来了');
  assert.equal(context.imageIntent.maxDimensionPx, 1800, '它接手的字段也回来了');
  assert.equal(context.videoIntent.intent, 'videoTarget');
  assert.equal(context.videoIntent.targetMiB, 64);
  assert.equal(ids.autoCompress.checked, true);
  assert.equal(context.outputSuffixInput.value, '_min');
  assert.equal(qualitySlider.value, '72', '控件铺成读回来的意图真正会跑的值');

  // systemConvert 只在 macOS 存在：在别的系统恢复它 = 一个看不见却驱动整条请求的意图。
  store['sizqo-settings'] = JSON.stringify({ version: 2, imageIntent: { intent: 'systemConvert' } });
  context.imageIntent.intent = 'balanced';
  context.isMacOS = () => false;
  context.loadCompressSettings();
  assert.equal(context.imageIntent.intent, 'balanced', '非 macOS 不许恢复一个没有的意图');
  context.isMacOS = () => true;
  context.loadCompressSettings();
  context.renderIntentBar();   // 启动流程在读完存档之后紧接着就画一次（updateMediaWorkspace）
  assert.equal(context.imageIntent.intent, 'systemConvert', 'macOS 上同一份存档就该恢复');
  assert.equal(rawRows.system.hidden, false, '系统转换的控件跟着意图一起出现');

  // 坏存档不许把界面带崩，也不许留下半个意图。
  store['sizqo-settings'] = '{ not json';
  context.imageIntent.intent = 'balanced';
  context.loadCompressSettings();
  assert.equal(context.imageIntent.intent, 'balanced');

  // 「还原默认设置」清两套：只清图片会留下"看着恢复了、视频还是上次的"。
  store['sizqo-video-settings'] = '{"crf":23}';
  context.resetSettings();
  assert.ok(!('sizqo-settings' in store) && !('sizqo-video-settings' in store));
  assert.equal(context._reloaded, true);

  console.log('PASS: 更多参数默认收起且只有点击能展开、收起行有摘要、接管提示与原始行一起跟意图、意图+字段成对落盘读回');
})().catch(error => { console.error(error); process.exitCode = 1; });
