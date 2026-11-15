// Real Pi InteractiveMode in an isolated Kitty test session. No user credentials,
// global packages/settings, network providers or live repository are loaded.
// Test-only timers/files drive screenshots; none are part of opencode-ui.ts.
import { mkdirSync, readFileSync, writeFileSync, existsSync, symlinkSync } from 'node:fs';
import path from 'node:path';
const root = '/tmp/opencode-ui-kitty';
mkdirSync(root + '/agent', { recursive: true });
mkdirSync(root + '/agent/bin', { recursive: true });
for (const name of ['fd', 'rg']) if (!existsSync(root + '/agent/bin/' + name)) symlinkSync('/usr/bin/' + name, root + '/agent/bin/' + name);
process.on('exit', code => { writeFileSync(root + '/exit', String(code)); });
process.on('uncaughtException', error => { writeFileSync(root + '/fatal', error.stack ?? String(error)); process.exit(1); });
process.on('unhandledRejection', error => { writeFileSync(root + '/fatal', error?.stack ?? String(error)); process.exit(1); });
process.env.PI_CODING_AGENT_DIR = root + '/agent';
process.env.PI_OFFLINE = '1';
const deps = process.env.PI_TEST_MODULES ?? '/var/home/ahsan/.pi/agent/install/releases/1.5.2/node_modules';
const base = `${deps}/@earendil-works/pi-coding-agent/dist/`;
const pi = await import(base + 'index.js');
const { loadExtensionFromFactory } = await import(base + 'core/extensions/loader.js');
const { InteractiveMode } = await import(base + 'modes/interactive/interactive-mode.js');
const { ProcessTerminal } = await import(`${deps}/@earendil-works/pi-tui/dist/index.js`);
const { fauxProvider, fauxAssistantMessage, fauxToolCall } = await import(`${deps}/@earendil-works/pi-ai/dist/providers/faux.js`);
const { default: extension } = await import('/tmp/opencode-ui-check/opencode-ui.mjs');

class FixtureTerminal extends ProcessTerminal {
  input;
  start(input, resize) { this.input = input; super.start(input, resize); }
  deliver(data) { this.input?.(data); }
}
const terminal = new FixtureTerminal();
const modelRuntime = await pi.ModelRuntime.create({ authPath: root + '/agent/absent-auth.json',
  modelsPath: null, modelsStorePath: root + '/agent/absent-models.json', refreshOnCreate: false, allowModelNetwork: false });
const provider = fauxProvider({ provider: 'ui-test-fixture', models: [{ id: 'fixture', name: 'Local UI fixture', reasoning: true, contextWindow: 200000 }], tokensPerSecond: Infinity });
modelRuntime.registerNativeProvider(provider.provider);
let liveContext;
const extensionRuntime = pi.createExtensionRuntime();
const extensions = [await loadExtensionFromFactory(extension, root, pi.createEventBus(), extensionRuntime, '<opencode-ui>'),
  await loadExtensionFromFactory(api => {
    api.on('session_start', (_event, ctx) => { liveContext = ctx; });
  }, root, pi.createEventBus(), extensionRuntime, '<screenshot-controller>')];
const resourceLoader = {
  getExtensions: () => ({ extensions, errors: [], runtime: extensionRuntime }),
  getSkills: () => ({ skills: [], diagnostics: [] }), getPrompts: () => ({ prompts: [], diagnostics: [] }),
  getThemes: () => ({ themes: [], diagnostics: [] }), getAgentsFiles: () => ({ agentsFiles: [] }),
  getSystemPrompt: () => 'Local deterministic UI fixture. Never call a network provider.', getSystemPromptSource: () => undefined,
  getAppendSystemPrompt: () => [], getAppendSystemPromptSources: () => [], extendResources() {}, async reload() {},
};
const settingsManager = pi.SettingsManager.inMemory({ tuiMode: 'fullscreen', quietStartup: true, theme: 'dark',
  lastChangelogVersion: pi.VERSION, editorPaddingX: 0, fullscreenExitOutput: 'none',
  compaction: { enabled: false }, retry: { enabled: false }, cacheWarming: { enabled: false } });
const services = { cwd: root, agentDir: root + '/agent', settingsManager, modelRuntime, resourceLoader, diagnostics: [] };
const { session } = await pi.createAgentSession({ ...services, model: provider.getModel(), thinkingLevel: 'high',
  sessionManager: pi.SessionManager.inMemory(root), tools: ['read', 'edit', 'write', 'bash'] });
const runtime = new pi.AgentSessionRuntime(session, services, async () => { throw new Error('Isolated screenshot fixture does not switch sessions.'); });
const mode = new InteractiveMode(runtime, { tuiMode: 'fullscreen', terminal });
await mode.init();
writeFileSync(root + '/ready', 'ready');
let previous = '';
let busy = false;
const interval = setInterval(async () => {
  if (busy || !existsSync(root + '/command')) return;
  const command = readFileSync(root + '/command', 'utf8').trim();
  if (command === previous) return;
  previous = command; busy = true;
  try {
    if (command === 'popup') {
      liveContext.ui.setEditorText('/'); terminal.deliver('\t');
    } else if (command === 'file-popup') {
      terminal.deliver('\x1b');
      writeFileSync(root + '/example.ts', 'const original = 1;\n');
      writeFileSync(root + '/😀-example.ts', 'const emoji = 1;\n');
      liveContext.ui.setEditorText('@'); terminal.deliver('\t');
    } else if (command === 'chat') {
      liveContext.ui.setToolsExpanded(true);
      terminal.deliver('\x1b'); liveContext.ui.setEditorText('');
      writeFileSync(root + '/example.ts', 'const original = 1;\n');
      provider.setResponses([
        fauxAssistantMessage([fauxToolCall('read', { path: root + '/example.ts', offset: 1, limit: 10 }, { id: 'read-screen' })], { stopReason: 'toolUse' }),
        fauxAssistantMessage([fauxToolCall('edit', { path: root + '/example.ts', oldText: 'original = 1', newText: 'original = 2' }, { id: 'edit-screen' })], { stopReason: 'toolUse' }),
        fauxAssistantMessage([fauxToolCall('read', { path: root + '/missing.txt' }, { id: 'error-screen' })], { stopReason: 'toolUse' }),
        fauxAssistantMessage('# Local UI fixture\n\nThe file read and edit completed. The missing-file read failed.\n\n| Surface | Rendering |\n|---|---|\n| Tools | Native output and themed cards |\n| Markdown | Native Pi |\n\n```ts\nconst original = 2;\n```\n\n[Pi documentation](https://pi.dev)\n'),
      ]);
      await session.prompt('Local fixture: read example.ts, edit its value, and demonstrate a missing-file error.');
    } else if (command === 'dialog') {
      void liveContext.ui.confirm('Fixture confirmation', 'Native dialog coexistence test. This is not a sandbox permission prompt.');
    } else if (command === 'stop') {
      clearInterval(interval); mode.stop(); await runtime.dispose(); process.exit(0);
    }
    writeFileSync(root + '/state', command);
  } catch (error) { writeFileSync(root + '/error', error.stack ?? String(error)); }
  finally { busy = false; }
}, 100);
