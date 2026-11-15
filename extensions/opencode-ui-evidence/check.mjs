// Isolated build/check harness. No installation or changes to Pi packages.
import { readFileSync, writeFileSync, mkdirSync, symlinkSync, existsSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const here = path.dirname(fileURLToPath(import.meta.url));
const deps = process.env.PI_TEST_MODULES ?? '/var/home/ahsan/.pi/agent/install/releases/1.5.2/node_modules';
const compiler = process.env.PI_TEST_TYPESCRIPT ?? '/tmp/pi-lens-step08-home/tools/node_modules/typescript';
const scratch = '/tmp/opencode-ui-check';
mkdirSync(scratch, { recursive: true });
if (!existsSync(path.join(scratch, 'node_modules'))) symlinkSync(deps, path.join(scratch, 'node_modules'));
const source = path.join(here, '../opencode-ui.ts');
const config = {
  compilerOptions: {
    target: 'ES2023', module: 'NodeNext', moduleResolution: 'NodeNext', strict: true,
    skipLibCheck: true, noEmit: true, allowImportingTsExtensions: true,
    types: ['node'], typeRoots: [path.join(deps, '@types')], baseUrl: scratch,
    paths: Object.fromEntries(['pi-coding-agent', 'pi-tui'].map(name =>
      [`@earendil-works/${name}`, [path.join(deps, `@earendil-works/${name}/dist/index.d.ts`)]])),
  }, files: [source],
};
writeFileSync(path.join(scratch, 'tsconfig.json'), JSON.stringify(config, null, 2));
execFileSync(process.execPath, [path.join(compiler, 'bin/tsc'), '-p', path.join(scratch, 'tsconfig.json')], { stdio: 'inherit' });
console.log('TypeScript strict check: PASS');
const { default: ts } = await import(path.join(compiler, 'lib/typescript.js'));
writeFileSync(path.join(scratch, 'opencode-ui.mjs'), ts.transpileModule(readFileSync(source, 'utf8'), {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2023 },
}).outputText);
execFileSync(process.execPath, ['--test', '--test-isolation=none', '--test-reporter=tap', path.join(here, 'tests.mjs')], {
  stdio: 'inherit', cwd: scratch, env: { ...process.env, PI_TEST_MODULES: deps },
});
