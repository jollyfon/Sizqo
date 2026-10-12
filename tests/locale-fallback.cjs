const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {
  applyUiLanguage,
  normalizeLocale,
  normalizePreference,
  readPreference,
  resolveUiLanguage,
  translateText,
} = require('../frontend/i18n.js');

const frontend = path.join(__dirname, '..', 'frontend');
const html = fs.readFileSync(path.join(frontend, 'index.html'), 'utf8');
const compareHtml = fs.readFileSync(path.join(frontend, 'compare.html'), 'utf8');
const appJs = fs.readFileSync(path.join(frontend, 'app.js'), 'utf8');

function assertEnglishStaticMarkup(markup, filename) {
  const visibleMarkup = markup
    .replace(/<!--[\s\S]*?-->/g, '')
    .replace(/<script\b[\s\S]*?<\/script>/gi, '')
    .replace(/<style\b[\s\S]*?<\/style>/gi, '');
  const textNodes = [...visibleMarkup.matchAll(/>([^<>]+)</g)].map(match => match[1].trim()).filter(Boolean);
  const attributes = [...visibleMarkup.matchAll(/\b(?:title|aria-label|aria-description|placeholder|alt)="([^"]*)"/g)].map(match => match[1]);
  const untranslated = [...textNodes, ...attributes]
    .map(value => translateText(value, 'en'))
    .filter(value => /[\u3400-\u9fff]/.test(value));
  assert.deepEqual(untranslated, [], `${filename} has visible static Chinese strings without English translations`);
}

assertEnglishStaticMarkup(html, 'index.html');
assertEnglishStaticMarkup(compareHtml, 'compare.html');

assert.match(html, /<html lang="en">/, 'English remains a safe initial document language');
assert.match(html, /<script src="i18n\.js"><\/script>/);
assert.match(html, /id="languagePreference" class="settings-select"/);
assert.match(html, /<option value="auto">自动（跟随系统）<\/option>/);
assert.match(html, /<option value="zh">简体中文<\/option>/);
assert.match(html, /<option value="en">英语<\/option>/);
assert.match(html, /SizqoI18n\.applyToDocument\(document\)/);
assert.match(html, /<h2 data-i18n-zh="拖拽图片、视频或文件夹到此处">Drop images, videos, or folders here<\/h2>/);
assert.match(html, /<p data-i18n-zh="自动识别媒体类型 · 文件夹递归扫描 · 大文件分析不阻塞界面">Media types detected automatically · folders scanned recursively<\/p>/);
assert.match(html, /<span data-i18n-zh="选择媒体文件">Choose Media<\/span>/);
assert.match(html, /<span data-i18n-zh="选择文件夹">Choose Folder<\/span>/);
assert.match(compareHtml, /<script src="i18n\.js"><\/script>/, 'the comparison window uses the same language preference');
assert.match(compareHtml, /SizqoI18n\.applyToDocument\(document\)/);
assert.match(appJs, /localizeUiText\('压缩中…'\)/, 'runtime UI messages go through the translation catalog');

assert.equal(normalizeLocale('C.UTF-8'), '');
assert.equal(normalizeLocale('en_US.UTF-8'), 'en-us');
assert.equal(normalizePreference('invalid'), 'auto');
assert.equal(resolveUiLanguage(['C', '']), 'en');
assert.equal(resolveUiLanguage(['en-US']), 'en');
assert.equal(resolveUiLanguage(['fr-FR']), 'en');
assert.equal(resolveUiLanguage(['zh-CN', 'en-US']), 'zh');
assert.equal(resolveUiLanguage(['zh-CN'], 'en'), 'en', 'manual English overrides the system locale');
assert.equal(resolveUiLanguage(['en-US'], 'zh'), 'zh', 'manual Chinese overrides the system locale');
assert.equal(readPreference({ getItem: () => 'zh' }), 'zh');
assert.equal(readPreference({ getItem: () => 'invalid' }), 'auto');
assert.equal(translateText('压缩设置', 'en'), 'Compression settings');
assert.equal(translateText('原图对比', 'en'), 'Image comparison');
assert.equal(translateText('压缩设置', 'zh'), '压缩设置');
assert.equal(translateText('已保存到: ', 'en') + '原格式.png', 'Saved to: 原格式.png', 'user file names remain untouched');

/// 意图摘要是新界面里"当前会跑什么"的唯一读数，英文界面下它不许夹汉字。
/// 逐个意图过真实编译器，而不是抄一份字符串清单 —— 清单会漂，编译器不会。
const intents = require('../frontend/intents.js');
function summaryFor(intentId, language, extra) {
  const state = Object.assign({
    intent: intentId, quality: 75, outputFormat: 'original', systemOutputFormat: 'original',
    systemImageSize: 'actual', preserveMetadata: true, maxDimensionPx: null, maxOutputBytes: null,
  }, extra || {});
  return translateText(intents.compileImage(state, true).summary, language);
}

assert.equal(summaryFor('balanced', 'en'), 'Balanced · Q75 · Original format');
assert.equal(summaryFor('balanced', 'zh'), '均衡 · Q75 · 原格式');
assert.equal(summaryFor('webSize', 'en'), 'Web size · Q72 · Original format · Longest edge 2560 px');
assert.equal(summaryFor('targetSize', 'en'), 'Target size · Budget 512 KB · Original format');
assert.equal(summaryFor('systemConvert', 'en'), 'System conversion · Original format · Actual size');

const everyImageSummary = intents.imageIntentList(true)
  .filter(intent => intent.id !== 'custom')
  .map(intent => summaryFor(intent.id, 'en'));
assert.deepEqual(everyImageSummary.filter(line => /[\u3400-\u9fff]/.test(line)), [],
  `英文界面下意图摘要残留中文：${everyImageSummary.join(' | ')}`);

// 视频那侧的"目标体积"是同一句话的形状，也一起检查。
const videoSummaries = intents.VIDEO_INTENTS
  .filter(intent => intent.id !== 'videoCustom')
  .map(intent => translateText(intents.compileVideo(
    { intent: intent.id, targetMiB: 64 }, {}, { normalizeOptions: o => Object.assign({}, o), controlSummary: () => '' }).summary, 'en'));
assert.deepEqual(videoSummaries.filter(line => /[\u3400-\u9fff]/.test(line)), [],
  `英文界面下视频意图摘要残留中文：${videoSummaries.join(' | ')}`);

function makeDocument(language, initialText, chineseText) {
  const translated = { textContent: initialText, getAttribute: () => chineseText };
  const plainText = { nodeValue: '均衡 · Q75 · 原格式' };
  const body = { querySelectorAll: selector => selector === '[data-i18n-zh]' ? [translated] : [] };
  const doc = {
    body,
    title: 'Sizqo',
    documentElement: { lang: '', dataset: {} },
    defaultView: null,
    createTreeWalker: () => {
      const nodes = [plainText];
      return { nextNode: () => nodes.shift() || null };
    },
    getElementById: () => null,
  };
  assert.equal(applyUiLanguage(doc, [language], 'auto'), language.startsWith('zh') ? 'zh' : 'en');
  return { doc, translated, plainText };
}

const english = makeDocument('C', 'Drop images, videos, or folders here', '拖拽图片、视频或文件夹到此处');
assert.equal(english.doc.documentElement.lang, 'en');
assert.equal(english.translated.textContent, 'Drop images, videos, or folders here');
assert.equal(english.plainText.nodeValue, 'Balanced · Q75 · Original format');

const chinese = makeDocument('zh-CN', 'Drop images, videos, or folders here', '拖拽图片、视频或文件夹到此处');
assert.equal(chinese.doc.documentElement.lang, 'zh-Hans');
assert.equal(chinese.translated.textContent, '拖拽图片、视频或文件夹到此处');
assert.equal(chinese.plainText.nodeValue, '均衡 · Q75 · 原格式');

console.log('Chinese and English locale checks passed');
