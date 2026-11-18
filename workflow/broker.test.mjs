import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, readFile, rm, symlink, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { here } from './host-policy.mjs';
import { createWorkspace, runFixed } from './broker.mjs';

async function fixture(t) {
  const root = await mkdtemp(join(tmpdir(), 'managed-broker-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const src = join(root, 'source');
  const scratch = join(root, 'scratch');
  await mkdir(src, { mode: 0o700 });
  await mkdir(scratch, { mode: 0o700 });
  await writeFile(join(src, 'a.txt'), 'before without newline');
  await writeFile(join(src, '.env.fixture'), 'mock protected value');
  await writeFile(join(src, 'package.json'), '{"type":"module"}');
  await writeFile(join(src, 'AGENTS.md'), 'mock protected policy');
  await symlink('.env.fixture', join(src, 'link.txt'));
  const copy = await createWorkspace(src, scratch);
  return { root, src, copy };
}

test('private copy filters protected paths and yields an applicable patch with no live writes', async t => {
  const { root, src, copy } = await fixture(t);
  const { broker } = copy;
  assert.deepEqual(await broker.list(), ['a.txt']);
  assert.deepEqual(await broker.search('before'), [{ path: 'a.txt', matches: [{ line: 1, snippet: 'before without newline' }] }]);
  assert.deepEqual(await broker.search('--before'), []);
  await assert.rejects(broker.read('package.json'), /Path denied/);
  await assert.rejects(broker.read('.env.fixture'), /Path denied/);
  assert.deepEqual(await broker.read('link.txt'), { missing: true });
  const original = await broker.read('a.txt');
  await assert.rejects(broker.write({ path: 'a.txt', expected: 'f'.repeat(64), text: 'stale' }), /rejected/);
  await broker.write({ path: 'a.txt', expected: original.sha256, text: 'after\n' });
  await broker.write({ path: 'b.txt', expected: null, text: 'created\n' });
  const delta = await broker.diff();
  assert.deepEqual(delta.paths, ['a.txt', 'b.txt']);
  assert.match(delta.patch, /No newline at end of file/);
  assert.equal(await readFile(join(src, 'a.txt'), 'utf8'), original.text);
  const patch = join(root, 'review.patch');
  await writeFile(patch, delta.patch);
  // --directory's absolute path is intentionally avoided. The command only
  // checks the fixture; it never applies the patch anywhere.
  const script = 'import os, subprocess, sys; os.chdir(sys.argv[1]); sys.exit(subprocess.run(["/usr/bin/git","apply","--check",sys.argv[2]]).returncode)';
  await runFixed('/usr/bin/python3', ['-I', '-c', script, src, patch]);
  await writeFile(join(src, 'a.txt'), 'external edit');
  await assert.rejects(broker.assertSource(), /Source drifted/);
  assert.equal(await readFile(join(src, 'a.txt'), 'utf8'), 'external edit');
});

test('Bubblewrap can produce a fingerprinted receipt from an isolated benign fixture', async t => {
  const { root, src } = await fixture(t);
  await writeFile(join(src, 'harmless.test.js'), `
    const assert = require('node:assert/strict');
    require('node:test')('private runtime', () => {
      assert.equal(process.cwd(), '/work');
      assert.equal(process.env.HOME, '/check-scratch');
      assert.equal(require('node:fs').existsSync(${JSON.stringify(src)}), false);
      assert.equal(require('node:fs').existsSync('/work/.env.fixture'), false);
    });\n`);
  const fresh = await createWorkspace(src, join(root, 'scratch'));
  const receipt = await fresh.broker.check('node-tests');
  if (receipt.status === 'not-tested') {
    t.skip('Bubblewrap unavailable; no host fallback');
    return;
  }
  assert.deepEqual(receipt, { id: 'node-tests', status: 'passed', fingerprint: await fresh.broker.fingerprint() });
});

test('the check supervisor refuses a candidate whose fingerprint changed before its copy', async t => {
  const { root, copy } = await fixture(t);
  const fingerprint = await copy.broker.fingerprint();
  await writeFile(join(copy.working, 'a.txt'), 'changed');
  const output = await runFixed('/usr/bin/python3', ['-B', here('check_runner.py'), copy.working, root,
    'node-tests', fingerprint.split(':')[1]]);
  assert.equal(output.trim(), '{"status": "not-tested"}');
});

test('Bubblewrap checks do not fall back to the host', async t => {
  const { root, src, copy } = await fixture(t);
  const target = join(root, 'not-host.txt');
  await writeFile(join(src, 'sandbox.test.js'), `require('node:fs').writeFileSync(${JSON.stringify(target)}, 'wrong');\n`);
  const again = await createWorkspace(src, join(root, 'scratch'));
  const result = await again.broker.check('node-tests');
  // If namespaces are available, the test fails inside the read-only copy.
  // Otherwise the helper must decline, never run the test on the host.
  assert.ok(['not-tested', 'failed'].includes(result.status), result.status);
  assert.equal(result.fingerprint, await again.broker.fingerprint());
  await assert.rejects(readFile(target), /ENOENT/);
  await copy.cleanup();
});
