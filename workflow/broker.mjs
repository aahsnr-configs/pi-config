import { spawn } from 'node:child_process';
import { mkdir, mkdtemp, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { checkIds } from './handoffs.mjs';
import { editable, permitted } from './policy.mjs';

const helper = name => fileURLToPath(new URL(name, import.meta.url));
const env = Object.freeze({ PATH: '/usr/bin', LC_ALL: 'C.UTF-8', HOME: '/nonexistent' });

export function runFixed(file, args, { input = '', signal, lockFd, milliseconds = 20000, noMatch = false } = {}) {
  signal?.throwIfAborted();
  return new Promise((resolve, reject) => {
    const child = spawn(file, args, { env, cwd: '/', stdio: ['pipe', 'pipe', 'ignore', ...(lockFd === undefined ? [] : [lockFd])] });
    let error;
    let length = 0;
    const output = [];
    const stop = reason => {
      error ??= reason;
      // The check supervisor handles TERM, kills and reaps Bubblewrap, and only
      // then exits (still holding the same flock). No timed early release.
      child.kill(file === '/usr/bin/python3' && args.includes(helper('check_runner.py')) ? 'SIGTERM' : 'SIGKILL');
    };
    const abort = () => stop(signal.reason);
    const deadline = setTimeout(() => stop(new Error('Workflow subprocess timed out')), milliseconds);
    signal?.addEventListener('abort', abort, { once: true });
    child.on('error', failure => { error ??= failure; });
    child.stdout.on('data', data => {
      length += data.length;
      if (length > 4_000_000) stop(new Error('Workflow subprocess output exceeded its limit'));
      else output.push(data);
    });
    child.stdin.on('error', () => {});
    child.stdin.end(input);
    child.once('close', (code, cause) => {
      clearTimeout(deadline);
      signal?.removeEventListener('abort', abort);
      if (error || (code !== 0 && !(noMatch && code === 1))) reject(error ?? new Error(`Workflow subprocess rejected the request (${code ?? cause})`));
      else resolve(Buffer.concat(output).toString('utf8'));
    });
    if (signal?.aborted) abort();
  });
}

async function request(script, args, options) {
  const output = await runFixed('/usr/bin/python3', ['-E', '-s', '-B', helper(script), ...args], options);
  try { return JSON.parse(output); }
  catch { throw new Error('Invalid workspace helper result'); }
}
const manifest = (root, signal) => request('private_workspace.py', ['manifest', root], { signal });

export async function createWorkspace(source, scratch, lockFd, signal) {
  const folder = await mkdtemp(join(scratch, 'pass-'));
  const baseline = join(folder, 'baseline');
  const working = join(folder, 'working');
  const checkRoot = join(folder, 'checks');
  const cleanup = () => rm(folder, { recursive: true, force: true }); // uniquely created by this call
  try {
    await Promise.all([baseline, working, checkRoot].map(path => mkdir(path, { mode: 0o700 })));
    const original = await request('private_workspace.py', ['copy', source, baseline], { signal });
    await request('private_workspace.py', ['copy', baseline, working], { signal });
    const current = await manifest(source, signal);
    if (current.fingerprint !== original.fingerprint) throw new Error('Workspace drifted while copying');
    return { working, cleanup, broker: createBroker({ source, baseline, working, checkRoot, original, lockFd }) };
  } catch (error) {
    await cleanup();
    throw error;
  }
}

async function pinnedRead(working, path, signal) {
  if (!permitted(path)) throw new Error('Path denied');
  return await request('workspace.py', ['read', working, path], { signal });
}
async function readText(working, path, signal) {
  const value = await pinnedRead(working, path, signal);
  if (value.missing) return value;
  if (value.text.includes('\0')) throw new Error('Binary file denied');
  // Pass pinned bytes over stdin. bat never opens a candidate by path.
  const text = await runFixed('/usr/bin/bat', ['--no-config', '--plain', '--paging', 'never',
    '--color', 'never', '--wrap', 'never', '--language', 'txt', '--tabs', '0', '-'], { input: value.text, signal });
  if (text !== value.text) throw new Error('File cannot be rendered as exact text');
  return { ...value, text };
}

async function listFiles(working, signal) {
  const names = await runFixed('/usr/bin/fd', ['--no-ignore', '--type', 'file', '--color', 'never',
    '--print0', '--base-directory', working, '--', '.'], { signal });
  const { files } = await manifest(working, signal);
  return names.split('\0').filter(name => name.startsWith('./')).map(name => name.slice(2))
    .filter(path => permitted(path) && Object.hasOwn(files, path)).sort();
}
async function searchFiles(working, query, signal) {
  if (typeof query !== 'string' || !query.trim() || query.length > 100 || /\p{Cc}/u.test(query)) {
    throw new Error('Expected a short literal query');
  }
  const results = [];
  const candidates = await listFiles(working, signal);
  for (const path of candidates.slice(0, 64)) {
    const { text } = await pinnedRead(working, path, signal);
    if (text === undefined || text.includes('\0')) continue;
    const output = await runFixed('/usr/bin/rg', ['--no-config', '--json', '--fixed-strings', '--max-count', '3',
      '--', query, '-'], { input: text, signal, noMatch: true });
    let matches;
    try {
      matches = output.trim().split('\n').filter(Boolean).map(line => JSON.parse(line))
        .filter(record => record.type === 'match').map(({ data }) => ({ line: data.line_number, snippet: data.lines.text.slice(0, 320) }));
    } catch { throw new Error('Invalid search result'); }
    if (matches.length) results.push({ path, matches });
    if (results.length === 12) break;
  }
  return results;
}

function createBroker({ source, baseline, working, checkRoot, original, lockFd }) {
  async function fingerprint(signal) {
    const live = await manifest(source, signal);
    const copy = await manifest(working, signal);
    return `${live.fingerprint}:${copy.fingerprint}`;
  }
  const assertSource = async signal => {
    const current = await manifest(source, signal);
    if (current.fingerprint !== original.fingerprint) throw new Error('Source drifted; start a fresh Plan. No patch emitted.');
  };
  const diff = async signal => request('private_workspace.py', ['patch', baseline, working], { signal });
  async function write(args, signal) {
    const { path, expected, text } = args;
    if (!editable(path) || typeof text !== 'string' || text.includes('\0') ||
        Buffer.byteLength(text) > 262144 || (expected !== null && !/^[a-f0-9]{64}$/.test(expected))) throw new Error('Invalid write arguments');
    await assertSource(signal);
    return request('private_workspace.py', ['write', working, path], { input: JSON.stringify({ expected, text }), signal });
  }
  async function check(id, signal) {
    if (!checkIds.includes(id)) throw new Error('Unknown approved check ID');
    const before = await fingerprint(signal);
    const flags = lockFd === undefined ? [] : ['--hold-lock'];
    const copyFingerprint = before.split(':')[1];
    const value = await request('check_runner.py', [working, checkRoot, id, copyFingerprint, ...flags], { signal, lockFd, milliseconds: 130000 });
    if (before !== await fingerprint(signal)) throw new Error('Workspace changed during check');
    return { id, status: value.status, fingerprint: before };
  }
  return {
    read: (path, signal) => readText(working, path, signal),
    search: (query, signal) => searchFiles(working, query, signal),
    list: signal => listFiles(working, signal),
    write, diff, check, fingerprint, assertSource,
  };
}
