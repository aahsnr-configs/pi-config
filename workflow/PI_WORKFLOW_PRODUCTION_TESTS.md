# Pi workflow production readiness tests

Run this checklist as the Linux user who will use the managed workflow launcher, from the same terminal environment and network. The offline suite uses fixtures and simulated providers. The live sections make model and MCP calls and may consume provider usage. Use the disposable workspace below throughout. Keep the shell open so the commands can reuse its paths and functions.

## 1. Verify the installed packages and run the offline suite

Open a Bash terminal. Run each block separately and stop to record any failure.

```bash
set -o pipefail
agent_dir="${PI_CODING_AGENT_DIR:-$HOME/.pi/agent}"
launcher="$agent_dir/workflow/launch.py"
cd "$agent_dir"

/usr/bin/jq -e '.version == "1.5.2"' install/releases/1.5.2/node_modules/@earendil-works/pi-coding-agent/package.json
/usr/bin/jq -e '.version == "0.89.0"' npm/node_modules/pi-subagents/package.json
/usr/bin/jq -e '.version == "2.7.2"' npm/node_modules/pi-permission-modes/package.json
/usr/bin/python3 -I "$launcher" --help >/dev/null  # Python syntax/import smoke, no managed session
/usr/bin/python3 -I "$launcher" --help
```

Each version comparison should print `true`. Help should list the three positional arguments and the advisor and public research options.

```bash
runtime_dir="/run/user/$(id -u)"
stat -c 'type=%F uid=%u mode=%a' "$runtime_dir"
[[ -d "$runtime_dir" && ! -L "$runtime_dir" && -O "$runtime_dir" && -w "$runtime_dir" ]]
ls -l /usr/bin/{node,python3,flock,fd,rg,bat,bwrap,git,jq}
/usr/bin/node --version
/usr/bin/python3 -E -s -B workflow/check_runner.py --probe
```

Expect an owner-only runtime directory, typically mode `700`, Node 22.19.0 or newer, and `Bubblewrap profile available`. Confirm each required executable exists. If the Bubblewrap probe fails, record isolated verification as blocked. Leave the host isolation settings intact.

```bash
umask 077
evidence=$(mktemp -d /tmp/pi-workflow-evidence.XXXXXXXX)
/usr/bin/node --test workflow/*.test.mjs | tee "$evidence/offline-node.log"
(cd workflow && /usr/bin/python3 -B -m unittest -v workspace_test launch_test) 2>&1 | tee "$evidence/offline-python.log"
```

Require zero failures and zero skipped tests for full readiness. The installed SDK fixtures cover exact child tool permissions, model and thinking selection, cancellation and timeout, and approval and receipt handling. The broker and dispatch suites cover protected paths, stale fingerprints, scoped writes, no host fallback for tests, and bounded MCP requests. The Orchestrator suite covers reuse and the limit of one approved repair. A managed Codex shell can block paths or namespaces needed by these suites. Run them in the intended Pi terminal and record the actual outcome there.

## 2. Prepare the live fixture and launch

Make `OPENAI_API_KEY` available in Terminal A through your credential manager. Do not put it in the commands, the fixture, or the logs.

```bash
fixture=$(mktemp -d /tmp/pi-workflow-project.XXXXXXXX)
printf 'before\n' > "$fixture/message.txt"
printf 'untouched\n' > "$fixture/side.txt"
printf 'SYNTHETIC_PROTECTED_CANARY\n' > "$fixture/.env.fixture"
ln -s .env.fixture "$fixture/link.txt"
ln "$fixture/.env.fixture" "$fixture/hardlink.txt"

cat > "$fixture/workflow.test.cjs" <<'JS'
const { test } = require('node:test');
const assert = require('node:assert/strict');
const { existsSync, readFileSync } = require('node:fs');

test('only the approved change appears in the isolated candidate', () => {
  assert.equal(process.cwd(), '/work');
  assert.equal(process.env.HOME, '/check-scratch');
  assert.equal(process.env.OPENAI_API_KEY, undefined);
  assert.equal(existsSync('.env.fixture'), false);
  assert.equal(existsSync('link.txt'), false);
  assert.equal(existsSync('hardlink.txt'), false);
  assert.equal(readFileSync('message.txt', 'utf8'), 'after\n');
  assert.equal(readFileSync('side.txt', 'utf8'), 'untouched\n');
});
JS

launch() {
  /usr/bin/python3 -I "$launcher" "$fixture" \
    openai/gpt-6.1-terra openai/gpt-6-luna,openai/gpt-6.1-sol \
    --advisor openai/gpt-6.1-sol "$@"
}
printf 'Paste this assignment in Terminal B when instructed: fixture=%q\n' "$fixture"
```

Confirm the public URL guard rejects an ineligible URL before taking the lock or making a provider call.

```bash
launch --research --public-url 'https://localhost/example' </dev/null
printf 'invalid public URL exit: %s\n' "$?"
```

Expect exit `2` and an argument error. Start the live session next.

```bash
launch | tee "$evidence/live-workflow.jsonl"
```

The first JSON record should be `ready` with `mode: "plan"`, Terra as core, Luna as evidence, Sol as advisor, and `research: false`. A `ready` record proves startup only. The prompts below are what exercise the live provider.

### Confirm the per-user lock from Terminal B

Run this while Terminal A remains open at the Pi prompt.

```bash
other_fixture=$(mktemp -d /tmp/pi-workflow-contender.XXXXXXXX)
/usr/bin/python3 -I "$launcher" "$other_fixture" \
  openai/gpt-6.1-terra openai/gpt-6-luna </dev/null
printf 'launcher exit: %s\n' "$?"
```

Expect an immediate busy message and exit `75`, even though the workspace is different. Repeat this during the cancellation step if the cancellation is slow enough.

## 3. Exercise the live workflow

Enter the following commands in **Terminal A's managed Pi prompt** one line at a time. Wait for the result before the next line except during the cancellation step.

### Plan, approval, scoped writes, and patch export

```text
/status
/plan
/build
!pwd
/reload
/subagent
/advisor decision Check whether this change can stay confined to message.txt.
Change message.txt from before to after with a trailing newline. In Plan, list the available files, look for the word before and read message.txt. Limit the proposed write scope to message.txt and include only node-tests. Leave side.txt and the fixture code alone.
/plan
/advisor conflict Review the resulting change against the approved scope.
/build
/status
/patch
```

- The initial `/plan` returns `plan: null`. The early `/build` errors with no approved Plan. Shell and unmanaged commands return `denied`.
- The ordinary prompt produces a plan with only `message.txt` in `paths` and `node-tests` in `checks`. Inspect it before `/build`. Plan uses Luna and Terra, and the requested Sol call is confined to that Plan pass. Build uses Terra and Luna with one additional Sol call after the separate advisor request. Inspect `role-usage` records when the provider reports usage. The offline fixture verifies the exact thinking levels.
- Build returns `build-result` with only `message.txt` and `verification.status: "passed"`. `/status` returns to Plan. `/patch` emits `type: "patch"`, and its check receipt fingerprint equals the patch fingerprint.

### Promote completed work, then start a fresh Orchestrator

```text
/orchestrator resume
/patch
/orchestrator Change only message.txt from before to after with one trailing newline. Include only node-tests. The Verifier should request the approved check and report its result and any gaps.
/status
/orchestrator resume
/plan
/build
/orchestrator approve
/status
/patch
```

- The first promotion should reuse the matching Plan and Build evidence. The independent Reviewer and Verifier finish the promoted task. Expect `completed`, the Verifier's summary, and a patch.
- The fresh run starts with Scouter and Planner and pauses with `approval-required` at Worker. Resuming at the approval gate keeps it there. `/plan` and `/build` fail while Orchestrator is active.
- Approving that plan runs Worker, Reviewer, and Verifier in order. Require a `completed` record with a Verifier summary and passing check receipt, a scoped patch, and a return to Plan. If Reviewer requests changes, review its findings and allow one same-scope repair with `/orchestrator approve`. The offline suite exercises the reject and repair paths deterministically.

### Reject stale plans and failing candidates

In Terminal A, enter:

```text
/plan Change only message.txt from before to after and include only node-tests.
/plan
```

After the plan returns, paste the printed `fixture=...` assignment into Terminal B and run:

```bash
printf 'external drift\n' > "$fixture/side.txt"
```

In Terminal A, enter `/build`, then `/patch`. Expect `stale` and an error for the patch. There should be no Build role dispatch for this approval. Restore the disposable fixture from Terminal B:

```bash
printf 'untouched\n' > "$fixture/side.txt"
```

In Terminal A, enter the following. Confirm the plan still includes `node-tests` before approving the deliberately failing candidate.

```text
/plan Create a deliberate failing candidate for this disposable fixture. Change only message.txt to wrong with a trailing newline. The existing node-tests check must run against the candidate. Leave all other files as they are.
/plan
/build
/patch
```

Expect `checks-failed` with a failed receipt and an error for `/patch`. The source still contains `before`.

### Cancel active work and exit cleanly

Enter these lines in quick succession while the Plan is still running:

```text
/plan Inspect the permitted fixture files and prepare a plan to change only message.txt.
/cancel
/plan
/status
```

Expect `cancel` with `cancelling: true` and a cancellation result. If the child is still draining, the bare `/plan` returns `busy`. If it has already drained, `/plan` responds immediately with the current Plan state. A new command must never be queued. If planning finished before cancellation, repeat the sequence. After the operation settles, `/status` must report `busy: false`, then enter `/quit`. The research launch in the next section confirms the lock is released.

### Validate the recorded patch in a second disposable workspace

Back in Terminal A's shell, run:

```bash
/usr/bin/rg -x 'before' "$fixture/message.txt"
/usr/bin/rg -x 'untouched' "$fixture/side.txt"
/usr/bin/jq -Rr 'fromjson? | select(.type == "role-usage") | [.role,.model] | @tsv' "$evidence/live-workflow.jsonl"
jq -Rse 'split("\n") | map(fromjson? | select(.type == "patch")) |
  last | .paths == ["message.txt"] and .verification.status == "passed" and
  .fingerprint == .verification.checks[0].fingerprint' "$evidence/live-workflow.jsonl"
jq -Rsj 'split("\n") | map(fromjson? | select(.type == "patch")) | last | .patch' \
  "$evidence/live-workflow.jsonl" > "$evidence/review.patch"

apply_fixture=$(mktemp -d /tmp/pi-workflow-apply.XXXXXXXX)
cp "$fixture/message.txt" "$fixture/side.txt" "$apply_fixture/"
/usr/bin/bat --plain --paging never --language diff "$evidence/review.patch"
/usr/bin/git -C "$apply_fixture" apply --check "$evidence/review.patch"
/usr/bin/git -C "$apply_fixture" apply "$evidence/review.patch"
/usr/bin/rg -x 'after' "$apply_fixture/message.txt"
/usr/bin/rg -x 'untouched' "$apply_fixture/side.txt"
/usr/bin/rg -x 'before' "$fixture/message.txt"
```

The only change should be `before` to `after` in the second disposable workspace. Keep the log and patch in the evidence directory.

## 4. Exercise live public research

Use only the public query, URL, and library ID below. If you use authenticated Context7 in production, make `CONTEXT7_API_KEY` available through your credential manager before this block.

```bash
launch --research \
  --public-query 'Node.js path.join versus path.resolve normalization' \
  --public-url 'https://nodejs.org/docs/latest-v24.x/api/path.html' \
  --context7-library '/nodejs/node' | tee "$evidence/live-research.jsonl"
```

At the managed prompt, enter:

```text
/plan Document the distinction between Node.js path.join and path.resolve. Ask the evidence role to use keenable-search, exa-search, context7-docs, and exa-fetch. In the plan summary, name each successful source and one concrete result, and label any failed lookup. Plan only.
/plan
/quit
```

Expect `research: true` at startup. Read the plan summary for actual source references and any tool failures. Model prose alone cannot establish that every endpoint worked. The following probe calls the same broker and transport directly with public inputs. A fresh broker for each action ensures the two fetch providers are each contacted.

```bash
cd "$agent_dir"
/usr/bin/node --input-type=module <<'JS' | tee "$evidence/mcp-probe.jsonl"
import { callPublicMcp } from './workflow/child-mcp.ts';
import { publicResearch } from './workflow/research.mjs';

const connect = (url, tool, args, signal) =>
  callPublicMcp(url, tool, args, signal, fetch, process.env.CONTEXT7_API_KEY);
const input = {
  query: 'Node.js path.join versus path.resolve normalization',
  url: 'https://nodejs.org/docs/latest-v24.x/api/path.html',
  libraryId: '/nodejs/node',
};
for (const id of ['keenable-search', 'keenable-fetch', 'exa-search', 'exa-fetch', 'context7-docs']) {
  try {
    const lookup = publicResearch('https://api.keenable.ai/mcp-public', input, connect);
    const result = await lookup(id, AbortSignal.timeout(12000));
    const body = result.publicUntrustedEvidence;
    if (!/(path\.(join|resolve)|node: ?path)/i.test(body) ||
        /\b(rate limit exceeded|unauthorized|invalid api key|no documentation found)\b/i.test(body)) {
      throw new Error('No usable public documentation');
    }
    console.log(JSON.stringify({ id, status: 'passed', server: result.server, chars: body.length }));
  } catch {
    console.log(JSON.stringify({ id, status: 'failed' }));
    process.exitCode = 1;
  }
}
JS
```

Require all five rows to pass. A successful direct probe plus missing evidence in the managed run means that workflow integration still needs investigation. If the managed run reports success and a direct probe fails, record the provider and failure rather than counting the model's report as proof. The offline MCP fixture verifies that the broker rejects unlisted actions and bounds responses.

## 5. Decide whether to use the configuration

Mark it ready for the tested environment only after all of these are true:

- The offline suites passed with zero skipped tests, and the Bubblewrap probe passed.
- A live Plan and Build produced an approved, passing patch. Applying it to the second disposable workspace changed exactly one file, and the original fixture remained unchanged.
- Fresh and promoted Orchestrator runs completed with scoped approvals and the expected handoffs. A second launcher could not enter while a managed session held the lock.
- A stale Plan and a failing candidate produced no patch. Cancellation drained, and a new session could start after exit.
- The managed research flow used the public input provided at launch, and the direct probes succeeded for every research provider you intend to enable.

Record the date, Pi and extension versions, selected model IDs, and any skipped, failed, or unverified item alongside the logs. Repeat these checks after changing packages, providers, permissions, or the intended runtime environment. View the evidence location with `printf '%s\n' "$evidence"`.
