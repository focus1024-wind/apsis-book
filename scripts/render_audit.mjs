// Renders every route in both languages and both themes, at two widths.
//
// 在两种语言与两种主题下、两种宽度下渲染每一条路由。
//
// Run it from the project root after a release build:
//
//     cargo build --release
//     node scripts/render_audit.mjs
//
// 在发布构建之后、于项目根目录运行。
//
// It writes one line per combination to /tmp/matrix.jsonl and reports what it
// found. A run that is interrupted keeps what it collected and skips it on
// the next run, so a long audit can be resumed.
//
// 每个组合向 /tmp/matrix.jsonl 写入一行并报告结果。被中断的一轮会保留已收集的
// 内容，下一次运行时跳过它们，因此长时间的审计可以续跑。
//
// Three things this must not do, each learned by getting it wrong.
//
// 有三件事它绝不能做，每一件都是犯错之后才明白的。
//
// It must never await a `Runtime.evaluate` that navigates: a call that
// destroys its own execution context never gets a reply, so the promise never
// settles and the run hangs until the environment kills it. The preferences
// are therefore set once per language on a page that stays put, and each
// route is reached by an ordinary navigation.
//
// 绝不能 await 一个会导航的 `Runtime.evaluate`：这样的调用会销毁自己的执行
// 上下文，回复永远不到，promise 永不结算，整轮运行挂住直到被环境杀掉。因此偏好
// 对每种语言只在一个不会移动的页面上设置一次，之后每条路由都以普通导航到达。
//
// It must not count text that the browser does not draw. A browser draws the
// text of a leaf element and drops text with no element around it; counting
// the latter made the formula check pass whatever the renderer produced.
//
// 绝不能统计浏览器不绘制的文本。浏览器绘制叶子元素内的文本，并丢弃没有元素包裹的
// 文本；统计后者会使公式检查无论在渲染器产出什么时都通过。
//
// It must not compare a formula against the source that is carried beside it
// for copying, which would compare the source with itself.
//
// 绝不能拿公式与它旁边为复制而携带的源码比对，那等于拿源码与自身比对。
import { spawn } from 'node:child_process';
import { appendFileSync, writeFileSync, readFileSync, existsSync } from 'node:fs';

const PORT = Number(process.env.AUDIT_PORT || 10300);
const CDP = Number(process.env.AUDIT_CDP || 9600);
const OUT = process.env.AUDIT_OUT || '/tmp/matrix.jsonl';
const PROFILE = process.env.AUDIT_PROFILE || '/tmp/cprofM';

const BIN = process.env.AUDIT_BIN || 'target/release/apsis-book';
const srv = spawn(BIN,
  ['--host', '127.0.0.1', '--port', String(PORT)], { stdio: 'ignore' });
const chrome = spawn('/usr/bin/google-chrome',
  ['--headless=new', '--disable-gpu', '--no-sandbox', '--disable-dev-shm-usage',
   '--disable-extensions', '--disable-background-networking',
   `--user-data-dir=${PROFILE}`, `--remote-debugging-port=${CDP}`,
   '--hide-scrollbars', 'about:blank'], { stdio: 'ignore' });

const cleanup = () => { try { chrome.kill('SIGKILL'); } catch {} try { srv.kill('SIGKILL'); } catch {} };
process.on('exit', cleanup);
process.on('SIGINT', () => { cleanup(); process.exit(1); });
process.on('SIGTERM', () => { cleanup(); process.exit(1); });

const wait = ms => new Promise(r => setTimeout(r, ms));
// Wait for the service, then for the browser.
for (let i = 0; i < 40; i++) {
  await wait(250);
  try { if ((await fetch(`http://127.0.0.1:${PORT}/health`)).ok) break; } catch {}
}
let ready = false;
for (let i = 0; i < 60 && !ready; i++) {
  await wait(500);
  try { ready = (await fetch(`http://127.0.0.1:${CDP}/json/version`)).ok; } catch {}
}
if (!ready) { console.log('chrome 未就绪'); cleanup(); process.exit(1); }

const target = await (await fetch(`http://127.0.0.1:${CDP}/json/new?about:blank`, { method: 'PUT' })).json();
const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise(r => ws.onopen = r);
let id = 0; const pending = new Map();
ws.onmessage = e => {
  const m = JSON.parse(e.data);
  if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); }
};
const send = (method, params = {}, ms = 15000) => new Promise(resolve => {
  const i = ++id;
  const timer = setTimeout(() => { pending.delete(i); resolve(null); }, ms);
  pending.set(i, m => { clearTimeout(timer); resolve(m); });
  ws.send(JSON.stringify({ id: i, method, params }));
});
const ev = async expr => {
  const r = await send('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true });
  return r?.result?.result?.value;
};
const goto = async url => {
  await send('Page.navigate', { url });
  for (let i = 0; i < 40; i++) {
    await wait(150);
    if (await ev('document.readyState') === 'complete') return true;
  }
  return false;
};

await send('Page.enable');
await send('Runtime.enable');

// What the page shows, per route.
const REPORT = `(() => {
  const LEAF = new Set(['mi','mo','mn','mtext','mspace','ms']);
  const collect = node => {
    let out = '';
    for (const child of node.childNodes) {
      if (child.nodeType === 3) {
        if (LEAF.has((node.tagName || '').toLowerCase())) out += child.textContent;
      } else if (child.nodeType === 1) {
        out += collect(child);
      }
    }
    return out;
  };
  const normalise = value => {
    const BS = String.fromCharCode(92);
    const raw = (value || '')
      .replace(/&amp;/g, '&').replace(/&lt;/g, '<').replace(/&gt;/g, '>');
    let src = '', i = 0;
    while (i < raw.length) {
      if (raw[i] === BS) {
        i++;
        let name = '';
        while (i < raw.length && /[A-Za-z]/.test(raw[i])) { name += raw[i]; i++; }
        if ((name === 'begin' || name === 'end') && raw[i] === '{') {
          while (i < raw.length && raw[i] !== '}') i++;
          i++;
        }
        continue;
      }
      src += raw[i]; i++;
    }
    return src.replace(/sqrt/g, '').replace(/[^0-9A-Za-z]/g, '');
  };
  const lost = [...document.querySelectorAll('.math')].map(m => {
    const drawn = m.querySelector('math');
    const shown = (drawn ? collect(drawn) : '').replace(/\\s+/g, '');
    const source = normalise(m.getAttribute('data-tex'));
    const missing = [...source].filter(c => !shown.includes(c)).join('');
    // A caret or an underscore left as an operator is a script that was
    // never attached. The same character inside an mtext is literal text, as
    // in a variable name written with an escaped underscore, which is drawn
    // as it was written and is correct.
    const marker = drawn
      && (drawn.innerHTML.includes('<mo>^</mo>') || drawn.innerHTML.includes('<mo>_</mo>'));
    return missing || (marker ? '^/_' : '');
  }).filter(Boolean);
  return JSON.stringify({
    length: (document.body.innerText || '').trim().length,
    overflow: document.documentElement.scrollWidth > document.documentElement.clientWidth + 2,
    lost: lost.length,
    lostWhat: lost.slice(0, 2).join('|'),
  });
})()`;

const B = `http://127.0.0.1:${PORT}`;
const docs = await (await fetch(`${B}/api/docs`)).json();
const flows = (await (await fetch(`${B}/api/graph/flows`)).json()).flows;
const api = await (await fetch(`${B}/api/api`)).json();
const tools = (await (await fetch(`${B}/api/tools`)).json()).tools;
const ids = [];
for (const g of docs.groups) for (const s of g.sections) {
  ids.push(...s.documents.map(d => d.id));
  for (const sub of s.subsections) ids.push(...sub.documents.map(d => d.id));
}
const routes = [
  '/', '/docs', '/graph', '/flows', '/api', '/tools',
  '/search?q=orbit', '/search?q=%E8%BD%A8%E9%81%93', '/nowhere',
  ...ids.map(i => `/docs/${i}`),
  ...flows.map(f => `/flows/${f.id}`),
  ...api.crates.map(c => `/api/${c.name}`),
  ...tools.map(t => `/tools/${t.id}`),
];

const errors = [];
ws.addEventListener('message', e => {
  const m = JSON.parse(e.data);
  if (m.method === 'Runtime.exceptionThrown') {
    errors.push(m.params.exceptionDetails.exception?.description?.split('\n')[0]
      || m.params.exceptionDetails.text);
  }
});

if (!existsSync(OUT)) writeFileSync(OUT, '');
const done = new Set(
  readFileSync(OUT, 'utf8').split('\n').filter(Boolean)
    .map(l => { try { const r = JSON.parse(l); return `${r.w}|${r.lang}|${r.theme}|${r.route}`; } catch { return null; } })
    .filter(Boolean),
);

let total = 0, bad = 0, skipped = 0;
for (const [w, h] of [[1440, 900], [390, 844]]) {
  await send('Emulation.setDeviceMetricsOverride', { width: w, height: h, deviceScaleFactor: 1, mobile: w < 600 });
  for (const lang of ['zh', 'en']) {
    for (const theme of ['dark', 'light']) {
      await goto(`${B}/#/`);
      await ev(`localStorage.setItem('apsis-book-language','${lang}');localStorage.setItem('apsis.theme','${theme}');true`);
      for (const route of routes) {
        const key = `${w}|${lang}|${theme}|${route}`;
        if (done.has(key)) { skipped++; continue; }
        errors.length = 0;
        const loaded = await goto(`${B}/#${route}`);
        await wait(220);
        total++;
        const report = await ev(REPORT);
        const d = report ? JSON.parse(report) : null;
        const row = {
          w, lang, theme, route, loaded,
          length: d?.length ?? -1,
          overflow: d?.overflow ?? null,
          lost: d?.lost ?? -1,
          lostWhat: d?.lostWhat ?? '',
          errors: [...new Set(errors)].slice(0, 2),
        };
        const problem = !loaded || d === null || d.length < 120 || d.overflow || d.lost > 0 || errors.length > 0;
        if (problem) bad++;
        appendFileSync(OUT, JSON.stringify(row) + '\n');
      }
    }
  }
}
console.log(`本次新增 ${total} 组 | 跳过的已完成组 ${skipped} | 有问题 ${bad}`);
ws.close(); cleanup();
process.exit(0);