import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { stripVTControlCharacters } from 'node:util';
import { createHash } from 'node:crypto';
const deps = process.env.PI_TEST_MODULES;
const lib = name => `${deps}/@earendil-works/${name}/dist/`;
const pi = await import(lib('pi-coding-agent') + 'index.js');
const tui = await import(lib('pi-tui') + 'index.js');
const themes = await import(lib('pi-coding-agent') + 'modes/interactive/theme/theme.js');
const loader = await import(lib('pi-coding-agent') + 'core/extensions/loader.js');
const { withBuiltInRenderers } = await import(lib('pi-coding-agent') + 'core/tools/renderers/index.js');
const { KeybindingsManager } = await import(lib('pi-coding-agent') + 'core/keybindings.js');
const faux = await import(lib('pi-ai') + 'providers/faux.js');
const ui = await import('/tmp/opencode-ui-check/opencode-ui.mjs');
themes.initTheme('dark', false);
const plain = lines => lines.map(stripVTControlCharacters);
const hash = data => createHash('sha256').update(JSON.stringify(data)).digest('hex');
const terminal = { rows: 30, columns: 100 };
const host = { terminal, mode: 'fullscreen', requestRender() {} };
const keys = new KeybindingsManager({}, '/tmp/opencode-ui-check/absent-keybindings.json');
const editorTheme = () => themes.getEditorTheme();
function pair() {
  const base = new pi.CustomEditor(host, editorTheme(), keys);
  const styled = new ui.OpenCodeEditor(host, editorTheme(), keys, new ui.ThemeAdapter(() => themes.theme), () => 'Chat · Fixture · high');
  base.setPaddingX(1); styled.setPaddingX(1);
  base.focused = styled.focused = true;
  return [base, styled];
}
function checkWidth(lines, width) {
  for (const line of lines) assert.ok(tui.visibleWidth(line) <= width, `width ${width}: ${JSON.stringify(line)}`);
}
function fixture(mode = 'fullscreen') {
  const handlers = new Map();
  const widgets = new Map();
  const notices = [];
  let factory, editor, header, footer, resolver;
  const entries = [];
  const localHost = { ...host, terminal: { ...terminal }, mode };
  const context = {
    hasUI: true, mode: 'tui', cwd: '/tmp/opencode-ui-check/project',
    model: { id: 'fixture', name: 'Fixture', provider: 'fixture', contextWindow: 200000 },
    thinkingLevel: 'high',
    modelRegistry: { isUsingOAuth: () => false },
    sessionManager: { getBranch: () => entries, getEntries: () => entries },
    getContextUsage: () => ({ contextWindow: 200000, percent: 18 }),
    ui: {
      get theme() { return themes.theme; },
      getEditorText: () => editor?.getText() ?? '',
      getEditorComponent: () => factory,
      setEditorComponent(value) { factory = value; editor = value?.(localHost, editorTheme(), keys); },
      setWidget(name, value) { if (value) widgets.set(name, value(localHost, themes.theme)); else widgets.delete(name); },
      setHeader(value) { header?.dispose?.(); header = value?.(localHost, themes.theme); },
      setFooter(value) { footer?.dispose?.(); footer = value?.(localHost, themes.theme, {}); },
      notify: (...args) => notices.push(args),
    },
  };
  // Throw if UI code registers or invokes ANY unapproved API.
  const api = new Proxy({
    on(name, callback) { handlers.set(name, callback); },
    registerToolRenderer(value) { resolver = value; },
    getThinkingLevel: () => 'high',
  }, { get(target, key) { assert.ok(key in target, `forbidden API ${String(key)}`); return target[key]; } });
  ui.default(api);
  return {
    context, entries, widgets, handlers, notices, host: localHost,
    get editor() { return editor; }, get header() { return header; }, get footer() { return footer; },
    get resolver() { return resolver; },
    async emit(type, event = {}) { assert.equal(await handlers.get(type)?.({ type, ...event }, context), undefined); },
  };
}

test('native editor text, cursor and submission stay equal across Unicode, paste, wrapping and resize', () => {
  const editors = pair();
  for (const text of ['hello', 'x'.repeat(1000), 'e\u0301 😀 中文', Array.from({ length: 100 }, (_, i) => `line ${i}`).join('\n'), '[image #1]']) {
    for (const e of editors) e.setText(text);
    for (const width of [1, 2, 5, 20, 60, 80, 100, 120, 160]) {
      let nativeRows;
      try { nativeRows = editors[0].render(width); }
      catch (error) {
        // A wide grapheme in a one-column layout recurses in native Pi 1.5.2.
        // Verify the exact upstream failure rather than treating it as a UI pass.
        assert.ok(width < 5 && error instanceof RangeError);
        assert.throws(() => editors[1].render(width), RangeError);
        continue;
      }
      const styledRows = editors[1].render(width);
      // Pi itself can overflow a one-cell viewport at an end-of-line cursor.
      // Assert native parity there; strict width checks cover usable viewports.
      if (width < 5) assert.deepEqual(styledRows.slice(1, -1).map(tui.visibleWidth), nativeRows.slice(1, -1).map(tui.visibleWidth));
      else checkWidth(styledRows, width);
      assert.deepEqual(editors[0].getCursor(), editors[1].getCursor());
      assert.equal(editors[0].getExpandedText(), editors[1].getExpandedText());
    }
  }
  const sequences = ['\x1b[200~one\ntwo😀\x1b[201~', '\x1b[D', '\x7f', 'z', '\x1b[Z', '\x1b[A', '\x1b[B', '\x1b[3~', '\x1a'];
  for (const e of editors) e.setText('');
  for (const sequence of sequences) {
    for (const e of editors) { e.render(80); e.handleInput(sequence); }
    assert.equal(editors[0].getExpandedText(), editors[1].getExpandedText());
    assert.deepEqual(editors[0].getCursor(), editors[1].getCursor());
  }
  let submissions = [];
  editors.forEach((e, i) => { e.onSubmit = text => { submissions[i] = text; }; e.setText('exact prompt'); e.handleInput('\r'); });
  assert.deepEqual(submissions, ['exact prompt', 'exact prompt']);
});

test('ghost placeholder preserves real empty text and the native IME cursor cell', () => {
  const [base, styled] = pair();
  const original = base.render(80), decorated = styled.render(80);
  assert.equal(styled.getText(), '');
  assert.match(plain(decorated).join('\n'), /Ask anything/);
  assert.ok(decorated[1].includes(tui.renderFakeCursor(' ')), 'both native fake-cursor boundary markers survive');
  for (const lines of [original, decorated]) {
    assert.equal(lines.filter(line => line.includes(tui.CURSOR_MARKER)).length, 1);
    const row = lines.find(line => line.includes(tui.CURSOR_MARKER));
    assert.equal(tui.visibleWidth(row.slice(0, row.indexOf(tui.CURSOR_MARKER))), 1);
  }
  styled.setText('typed');
  assert.doesNotMatch(plain(styled.render(80)).join('\n'), /Ask anything/);
});

test('dashboard uses the original Pi brand pixels and a decorated frame', async () => {
  const nativeLogo = await import(lib('pi-coding-agent') + 'modes/interactive/components/pi-logo.js');
  const logo = ui.stockPiLogo(themes.theme, false);
  assert.deepEqual(plain(logo), plain(nativeLogo.piLogoLines()));
  assert.deepEqual(logo.map(tui.visibleWidth), [4, 4]);
  const mode = themes.theme.getColorMode();
  for (const colour of [tui.rgbColor(228, 138, 122), tui.rgbColor(79, 142, 179), tui.rgbColor(234, 182, 93)]) {
    assert.ok(logo.join('').includes(tui.foregroundAnsi(colour, mode)), 'native brand colour retained');
  }
  const f = fixture(); await f.emit('session_start');
  const lines = plain(f.header.render(100));
  assert.ok(lines.some(line => line.includes('╭') && line.includes('✦')));
  assert.ok(lines.some(line => line.includes('▀▀█')));
  for (const line of lines.filter(line => line.trim())) assert.equal(tui.visibleWidth(line), 59);
});

test('startup composer targets two-thirds height and absorbs native popup growth', () => {
  for (const rows of [18, 24, 30, 40, 50, 90, 180]) {
    for (const editorRows of [3, 5, 9]) {
      const footerRows = 1;
      const spacer = ui.spacerRows(rows, editorRows, footerRows);
      const top = rows - editorRows - footerRows - spacer;
      if (rows - Math.floor(rows * 2 / 3) >= editorRows + footerRows) assert.equal(top, Math.floor(rows * 2 / 3));
      else assert.equal(spacer, 0, 'oversized native editor takes priority over position');
    }
  }
  assert.equal(ui.spacerRows(12), 0);
});

test('native slash completions preserve rows, keyboard insertion, paging, mouse and viewport capture', async () => {
  const commands = Array.from({ length: 16 }, (_, i) => ({ name: `test${i}`, description: `native description ${i}` }));
  const editors = pair();
  for (const e of editors) {
    e.setAutocompleteProvider(new tui.CombinedAutocompleteProvider(commands, '/tmp/opencode-ui-check', null));
    e.setText('/'); e.handleInput('\t');
  }
  await new Promise(resolve => setTimeout(resolve, 25));
  assert.ok(editors.every(e => e.isShowingAutocomplete()));
  assert.ok(editors.every(e => e.capturesViewportInput));
  for (const sequence of ['\x1b[B', '\x1b[6~', '\x1b[5~', '\x1b[A']) {
    for (const e of editors) { e.render(80); e.handleInput(sequence); }
    const [a, b] = editors.map(e => plain(e.render(80)));
    assert.deepEqual(a.slice(3), b.slice(3), 'native below-editor suggestion rows unchanged');
  }
  const mouse = { type: 'click', button: 'left', x: 6, y: 4, width: 80, height: 15, shift: false, alt: false, ctrl: false };
  assert.deepEqual(editors[0].handleMouse(mouse), editors[1].handleMouse(mouse));
  for (const e of editors) e.handleInput('\t');
  assert.equal(editors[0].getText(), editors[1].getText());
  assert.ok(editors.every(e => !e.isShowingAutocomplete()));
});

test('native file completion uses the host provider, including emoji names and no matches', async () => {
  mkdirSync('/tmp/opencode-ui-check/files', { recursive: true });
  writeFileSync('/tmp/opencode-ui-check/files/😀.ts', 'fixture');
  writeFileSync('/tmp/opencode-ui-check/files/😀-two.ts', 'fixture');
  const editors = pair();
  for (const prefix of ['@files/', '@nothing-matches-this']) {
    for (const e of editors) {
      e.setAutocompleteProvider(new tui.CombinedAutocompleteProvider([], '/tmp/opencode-ui-check', '/usr/bin/fd'));
      e.setText(prefix); e.handleInput('\t');
    }
    await new Promise(resolve => setTimeout(resolve, 30));
    assert.equal(editors[0].isShowingAutocomplete(), editors[1].isShowingAutocomplete());
    if (prefix === '@files/') {
      assert.ok(editors.every(e => e.isShowingAutocomplete()), 'matching file suggestions are actually open');
      for (const e of editors) assert.match(plain(e.render(80)).join('\n'), /😀/);
    } else assert.ok(editors.every(e => !e.isShowingAutocomplete()), 'no-match menu is actually closed');
    for (const e of editors) { e.render(80); e.handleInput('\t'); }
    assert.equal(editors[0].getText(), editors[1].getText());
    for (const e of editors) e.handleInput('\x1b');
  }
});

test('mouse positions on native text remain unchanged', () => {
  const editors = pair();
  for (const e of editors) { e.setText('alpha 😀 beta\nsecond line'); e.render(20); }
  for (const [x, y] of [[3, 1], [7, 1], [2, 2]]) {
    const event = { type: 'click', button: 'left', x, y, width: 20, height: 5 };
    assert.deepEqual(editors[0].handleMouse(event), editors[1].handleMouse(event));
    assert.deepEqual(editors[0].getCursor(), editors[1].getCursor());
  }
});

test('model/effort and the real downward scroll count coexist on multiline input', () => {
  const [, styled] = pair();
  styled.setText(Array.from({ length: 100 }, (_, i) => `line ${i}`).join('\n'));
  styled.render(80);
  for (let i = 0; i < 100; i++) styled.handleInput('\x1b[A');
  const text = plain(styled.render(80)).join('\n');
  assert.match(text, /Fixture.*high.*↓ \d+/);
});

test('phase transitions, permission coexistence, tree navigation and 30 reload cycles clean owned UI', async () => {
  const f = fixture();
  for (let i = 0; i < 30; i++) {
    await f.emit('session_start');
    assert.equal(f.widgets.size, 2);
    assert.ok(f.header.render(100).length);
    f.editor.setText('merely typing');
    assert.ok(f.header.render(100).length);
    await f.emit('input', { text: ' ', images: [], source: 'interactive' });
    assert.ok(f.header.render(100).length);
    await f.emit('agent_start');
    assert.equal(f.header.render(100).length, 0);
    assert.equal(f.widgets.get('opencode-ui:dock').render(100).length, 0);
    await f.emit('tool_execution_start', { toolCallId: '1', toolName: 'read', args: {} });
    assert.ok(f.widgets.get('opencode-ui:activity').render(100).length);
    await f.emit('ui_prompt_start');
    assert.equal(f.widgets.get('opencode-ui:activity').render(100).length, 0);
    await f.emit('ui_prompt_end');
    await f.emit('tool_execution_end', { toolCallId: '1', isError: true });
    await f.emit('agent_settled', { aborted: false });
    await f.emit('session_tree');
    assert.ok(f.header.render(100).length);
    await f.emit('session_shutdown');
    assert.equal(f.widgets.size, 0);
    assert.equal(f.editor, undefined);
    assert.equal(f.header, undefined);
    assert.equal(f.footer, undefined);
  }
  assert.equal(f.notices.length, 0);
});

test('nonempty resume, regular-mode guard and existing custom editor are respected', async () => {
  const resumed = fixture(); resumed.entries.push({ type: 'compaction' });
  await resumed.emit('session_start');
  assert.equal(resumed.header.render(80).length, 0);
  const regular = fixture('regular'); await regular.emit('session_start');
  assert.equal(regular.editor, undefined); assert.equal(regular.widgets.size, 0);
  const owner = fixture();
  owner.context.ui.setEditorComponent(() => ({ getText: () => 'old editor' }));
  await owner.emit('session_start');
  assert.equal(owner.editor.getText(), 'old editor'); assert.equal(owner.header, undefined);
});

test('composer shell mode agrees with native whitespace handling', async () => {
  const f = fixture(); await f.emit('session_start');
  f.editor.setText('  !pwd');
  assert.match(plain(f.editor.render(80)).join('\n'), /Shell.*Fixture/);
});

test('other footer/header owners are not reclaimed or removed at shutdown', async () => {
  const f = fixture(); await f.emit('session_start');
  const replacement = { render: () => ['owned elsewhere'], invalidate() {} };
  f.context.ui.setFooter(() => replacement);
  f.context.ui.setHeader(() => replacement);
  await f.emit('thinking_level_select'); await f.emit('session_shutdown');
  assert.equal(f.footer, replacement); assert.equal(f.header, replacement);
  assert.equal(f.notices.length, 1);
});

test('workflow interleaving, nested IDs, failures, interruptions and bounded retention', () => {
  const store = new ui.WorkflowStore();
  store.start('a', 'read'); store.start('b', 'write', 'a');
  store.end('b', true, 20); store.end('a', false, 30);
  assert.equal(store.records.get('b').parent, 'a');
  assert.equal(store.records.get('b').state, 'failed');
  assert.equal(store.records.get('a').durationMs, 30);
  store.start('c', 'bash'); store.settle(true);
  assert.equal(store.records.get('c').state, 'interrupted');
  for (let i = 0; i < 1000; i++) store.start(String(i), 'read');
  assert.equal(store.records.size, 128);
  store.reset(); store.end('late-old-session-id', false);
  assert.equal(store.records.size, 0);
});

test('renderer chaining preserves custom, interactive, security, edit and unknown renderers by identity', () => {
  const resolve = ui.toolViewResolver();
  const interactive = { renderShell: 'self', renderCall: () => new tui.Text('interactive') };
  for (const name of ['render_widget', 'read', 'bash', 'edit', 'permission', 'mcp__fixture', 'unknown']) {
    let calls = 0;
    assert.equal(resolve(name, () => { calls++; return interactive; }), interactive);
    assert.equal(calls, 1);
  }
  assert.equal(resolve('bash', () => undefined), undefined);
  assert.equal(resolve('edit', () => undefined), undefined);
  assert.equal(resolve('mcp__fixture', () => undefined), undefined);
  assert.equal(resolve('read', () => undefined), undefined, 'missing renderer retains the native fallback');
});

test('actual core call renderers preserve all arguments, expansion, result callbacks and component caches', () => {
  const resolve = ui.toolViewResolver();
  const cases = [
    [pi.createReadTool, { path: 'file.txt', offset: 5, limit: 20 }, /5/],
    [pi.createGrepTool, { path: '.', pattern: 'needle', glob: '*.ts', limit: 10 }, /needle/],
    [pi.createFindTool, { path: '.', pattern: '*.ts' }, /\*\.ts/],
    [pi.createLsTool, { path: '.', limit: 10 }, /ls/],
    [pi.createWriteTool, { path: 'file.ts', content: 'const x = 1;' }, /const x/],
    [pi.createBashTool, { command: 'printf deterministic' }, /deterministic/],
  ];
  for (const [create, args, expected] of cases) {
    const tool = create('/tmp/opencode-ui-check');
    const native = withBuiltInRenderers(tool.name, tool);
    const decorated = resolve(native.name, () => native);
    assert.equal(decorated.renderResult, native.renderResult);
    const context = { args, state: {}, cwd: '/tmp/opencode-ui-check', toolCallId: native.name,
      executionStarted: true, argsComplete: true, isPartial: false, expanded: true, showImages: false,
      isError: false, outputPad: 1, keyText: () => 'ctrl+o', keyHint: () => '(ctrl+o)', invalidate() {}, lastComponent: undefined };
    const card = decorated.renderCall(args, themes.theme, context);
    assert.ok(card instanceof ui.PanelCard, `${native.name} uses a real Box card`);
    const original = native.renderCall(args, themes.theme, { ...context, state: {} });
    assert.deepEqual(card.content.render(80), original.render(80));
    assert.match(plain(card.render(100)).join('\n'), expected);
    checkWidth(card.render(60), 60);
    const again = decorated.renderCall(args, themes.theme, { ...context, lastComponent: card });
    assert.equal(again.content, card.content, `${native.name} native cache survives decoration`);
  }
});

test('cost includes documented usage entries once; footer formats unknown/subscription and Unicode widths', async () => {
  const f = fixture();
  f.entries.push(...['usage', 'compaction', 'branch_summary'].map(type => ({ type, usage: { cost: { total: 0.1 } } })),
    { type: 'message', message: { role: 'assistant', usage: { cost: { total: 0.2 } } } },
    { type: 'message', message: { role: 'toolResult', usage: { cost: { total: 0.3 } } } });
  assert.ok(Math.abs(ui.sessionCost(f.context) - 0.8) < 1e-10);
  await f.emit('session_start');
  for (const width of [1, 2, 5, 20, 40, 60, 80, 100, 160]) checkWidth(f.footer.render(width), width);
  const text = plain(f.footer.render(100)).join('\n');
  assert.match(text, /Cost \$0.800 est.*Context 18% \/ 200k/);
  assert.equal(ui.displayDirectory('/home/ann-other/project', '/home/ann'), '/home/ann-other/project');
  f.context.modelRegistry.isUsingOAuth = () => true;
  assert.equal(ui.sessionCost(f.context), undefined);
});

test('themed surfaces update locally on theme change; dimensions and snapshots remain bounded', async () => {
  const f = fixture(); await f.emit('session_start');
  const before = f.editor.render(100);
  themes.initTheme('light', false);
  assert.notDeepEqual(before, f.editor.render(100));
  const snapshots = [];
  for (const [cols, rows] of [[60, 18], [80, 24], [100, 30], [120, 40], [160, 50]]) {
    f.host.terminal.columns = cols; f.host.terminal.rows = rows;
    checkWidth(f.header.render(cols), cols); checkWidth(f.editor.render(cols), cols); checkWidth(f.footer.render(cols), cols);
    const spacer = ui.spacerRows(rows, f.editor.render(cols).length, f.footer.render(cols).length);
    assert.ok(spacer >= 0 && spacer <= 34);
    snapshots.push(`${cols}x${rows}\n${plain(f.header.render(cols)).join('\n')}\n${plain(f.editor.render(cols)).join('\n')}\n[${spacer} spacer rows]\n${plain(f.footer.render(cols)).join('\n')}`);
  }
  writeFileSync(new URL('./snapshots.txt', import.meta.url), snapshots.join('\n\n'));
  themes.initTheme('dark', false);
});

test('no forbidden runtime APIs, dependencies, dynamic code or private access', () => {
  const source = readFileSync(new URL('../opencode-ui.ts', import.meta.url), 'utf8');
  assert.doesNotMatch(source, /\bas any\b|\b(?:fetch|eval|require|setInterval|setTimeout)\s*\(|\.prototype\b|child_process|node:fs|\.registerTool\s*\(|\.sendMessage\s*\(|\.sendUserMessage\s*\(|\.appendEntry\s*\(|\.setActiveTools\s*\(|registerMarkdownTransformer\s*\(/);
  const text = ui.safeLabel('file\x1b\x07\n\u202e.exe');
  assert.doesNotMatch(text, /[\x00-\x1f\x7f-\x9f\u202e]/);
});

test('Pi actual TypeScript extension loader loads one file without model-facing registrations', async () => {
  const source = new URL('../opencode-ui.ts', import.meta.url).pathname;
  const loaded = await loader.loadExtensions([source], '/tmp/opencode-ui-check');
  assert.deepEqual(loaded.errors, []);
  assert.equal(loaded.extensions.length, 1);
  const extension = loaded.extensions[0];
  assert.equal(extension.tools.size, 0);
  assert.equal(extension.commands.size, 0);
  assert.equal(extension.messageRenderers.size, 0);
  assert.equal(extension.toolRenderers.length, 1);
  const native = withBuiltInRenderers('read', undefined);
  const resolver = extension.toolRenderers[0];
  const decorated = resolver('read', () => native);
  assert.equal(decorated.renderResult, native.renderResult);
  const card = decorated.renderCall({ path: '/tmp/example.txt', offset: 3, limit: 2 }, themes.theme,
    { cwd: '/tmp', expanded: true, state: {}, isPartial: true, executionStarted: true });
  assert.match(plain(card.render(80)).join('\n'), /┃.*read.*example\.txt.*3-4/);
});

test('actual Pi SDK paired provider payload/manifest/message/token audit, UI enabled vs disabled', async () => {
  async function run(enabled) {
    const root = '/tmp/opencode-ui-check/sdk'; mkdirSync(root, { recursive: true });
    const runtime = await pi.ModelRuntime.create({ authPath: root + '/absent-auth.json', modelsPath: null,
      modelsStorePath: root + '/absent-models-store.json', refreshOnCreate: false, allowModelNetwork: false });
    const provider = faux.fauxProvider({ provider: 'ui-audit', api: 'ui-audit-api', models: [{ id: 'fixture', contextWindow: 200000 }], tokensPerSecond: Infinity });
    runtime.registerNativeProvider(provider.provider);
    const requests = [];
    provider.setResponses([
      context => { requests.push(structuredClone(context)); return faux.fauxAssistantMessage([faux.fauxToolCall('read', { path: root + '/file.txt' }, { id: 'read-1' })], { stopReason: 'toolUse', timestamp: 1 }); },
      context => { requests.push(structuredClone(context)); return faux.fauxAssistantMessage('# Done\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\n```ts\nconst x = 1;\n```\n\n[link](https://example.org)', { timestamp: 2 }); },
    ]);
    writeFileSync(root + '/file.txt', 'deterministic file content');
    const extensionRuntime = loader.createExtensionRuntime();
    const extension = enabled ? await loader.loadExtensionFromFactory(ui.default, root, pi.createEventBus(), extensionRuntime, '<opencode-ui-audit>') : undefined;
    const localConsumer = await loader.loadExtensionFromFactory(api => {
      api.on('input', event => event.text === 'local-ui-only' ? { action: 'handled' } : undefined);
    }, root, pi.createEventBus(), extensionRuntime, '<local-input-consumer>');
    const resources = {
      getExtensions: () => ({ extensions: [...(extension ? [extension] : []), localConsumer], errors: [], runtime: extensionRuntime }),
      getSkills: () => ({ skills: [], diagnostics: [] }), getPrompts: () => ({ prompts: [], diagnostics: [] }),
      getThemes: () => ({ themes: [], diagnostics: [] }), getAgentsFiles: () => ({ agentsFiles: [] }),
      getSystemPrompt: () => 'Deterministic UI audit. Read the requested file.', getSystemPromptSource: () => undefined,
      getAppendSystemPrompt: () => [], getAppendSystemPromptSources: () => [], extendResources() {}, async reload() {},
    };
    const { session } = await pi.createAgentSession({ cwd: root, agentDir: root, modelRuntime: runtime,
      model: provider.getModel(), thinkingLevel: 'off', tools: ['read'], resourceLoader: resources,
      settingsManager: pi.SettingsManager.inMemory({ compaction: { enabled: false }, retry: { enabled: false }, cacheWarming: { enabled: false } }),
      sessionManager: pi.SessionManager.inMemory(root),
    });
    const visual = fixture(); // only its supported UI host is used by real Pi events
    const errors = [];
    try {
      await session.bindExtensions({ uiContext: visual.context.ui, mode: 'tui', onError: error => errors.push(error) });
      await session.prompt('local-ui-only', { expandPromptTemplates: false });
      assert.equal(provider.state.callCount, 0);
      if (enabled) assert.ok(visual.header.render(80).length, 'handled local input leaves HOME visible');
      await session.prompt('Read file.txt, then return Markdown.', { expandPromptTemplates: false });
      const beforeRender = provider.state.callCount;
      const storedBeforeRender = JSON.stringify(session.agent.state.messages);
      for (let i = 0; i < 50; i++) {
        for (const component of [visual.header, visual.editor, visual.footer, ...visual.widgets.values()]) component?.render(60 + (i % 4) * 20);
      }
      assert.equal(provider.state.callCount, beforeRender);
      assert.equal(JSON.stringify(session.agent.state.messages), storedBeforeRender, 'stored messages unchanged byte-for-byte by redraws');
      assert.deepEqual(errors, []);
      const normalize = value => JSON.parse(JSON.stringify(value, (key, v) => ['timestamp', 'durationMs', 'responseId'].includes(key) ? undefined : v));
      const result = { requests: normalize(requests), calls: provider.state.callCount,
        manifest: normalize(session.agent.state.tools.map(tool => ({ name: tool.name, description: tool.description, parameters: tool.parameters }))),
        messages: normalize(session.agent.state.messages),
        usage: session.agent.state.messages.filter(message => message.role === 'assistant').map(message => message.usage),
      };
      return result;
    } finally { session.dispose(); }
  }
  const off = await run(false), on = await run(true);
  assert.equal(off.calls, 2); assert.equal(on.calls, 2);
  assert.deepEqual(on.requests, off.requests);
  assert.deepEqual(on.manifest, off.manifest);
  assert.deepEqual(on.messages, off.messages);
  assert.deepEqual(on.usage, off.usage);
  writeFileSync(new URL('./token-audit.json', import.meta.url), JSON.stringify({
    fixture: 'actual Pi 1.5.2 SDK and native faux provider; read + Markdown answer',
    callsOff: off.calls, callsOn: on.calls, deltaInputTokens: 0, deltaOutputTokens: 0,
    requestSignatureOff: hash(off.requests), requestSignatureOn: hash(on.requests),
    manifestSignatureOff: hash(off.manifest), manifestSignatureOn: hash(on.manifest),
    messagesSignatureOff: hash(off.messages), messagesSignatureOn: hash(on.messages),
    limitation: 'Controlled fixture only; full installed-stack and live billed-provider audit not performed.',
  }, null, 2) + '\n');
});
