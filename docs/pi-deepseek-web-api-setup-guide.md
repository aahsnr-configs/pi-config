# Pi Coding Agent — DeepSeek Web API setup guide (Linux / Kitty)

**Verified against upstream documentation on 16 November 2026.**  
**Scope:** Install and configure **`kittors/deepseek-web-api` only** so DeepSeek Chat can be selected directly in Pi. This guide does **not** install `web-ai-skills`, a browser worker, or a second agent.

> **Important:** This is an **unofficial** adapter for DeepSeek's consumer website, *not* the official paid DeepSeek API. It depends on private web endpoints, browser authentication, and proof-of-work behavior that can change. Successful local installation does not guarantee successful DeepSeek requests, ongoing access, or permission under the service's terms. Use only with an account you are authorized to access. Do not entrust it with unattended file modifications until you have tested its simulated tool calls.

## 0. What this setup will do

Your Pi workflow will be:

```text
Kitty terminal → Pi Coding Agent → custom "deepseek-web" provider
                         ↓
             http://127.0.0.1:8787/v1
                         ↓
               deepseek-web-api (local Node.js process)
                         ↓
          authenticated DeepSeek consumer web session
                         ↓
                DeepSeek response returned to Pi
```

- DeepSeek becomes **selectable with `/model`**; your existing OpenAI/other Pi providers remain available.
- The proxy has its own dedicated Chrome/Chromium login profile; you **do not** need to launch a browser manually with `--remote-debugging-port` for the standard setup.
- The adapter translates OpenAI-compatible Chat Completions to the DeepSeek Web interface. Tool calls are **prompt-simulated**, not native function calls.
- No new Pi extension, skill, `AGENTS.md` instruction, or prompt template is necessary. This avoids extra instructions in Pi's model context.
- The adapter's model metadata shows **zero API cost**, since it does not use the paid developer API. This is **not** unlimited free service and does **not** mean token usage is zero; DeepSeek still processes context, and website limits apply.

### Paths used throughout

| Item | Path |
|---|---|
| Existing Pi configuration | `~/.pi/agent/` |
| New adapter checkout | `~/.local/opt/deepseek-web-api/` |
| Adapter authentication, session state, and local proxy key | `~/.local/opt/deepseek-web-api/data/` (Git-ignored) |
| Pi provider configuration | `~/.pi/agent/models.json` |
| Pi configuration backups | `~/.local/state/pi-config-backups/` (outside the Pi Git repository) |
| Local HTTP address | `http://127.0.0.1:8787` |
| Pi `baseUrl` (note the required `/v1`) | `http://127.0.0.1:8787/v1` |

**Shell note for Kitty/fish users:** All fenced terminal commands below use **Bash** syntax. If Kitty starts fish, enter `bash` in that tab before executing a multi-line Bash block (and exit Bash afterward). Each section that uses `DS_REPO` defines it anew, so blocks also work in separate terminal tabs. All commands are intended for a **normal host terminal**, not inside Pi's interactive prompt.

## 1. Prerequisites and preflight

The upstream adapter requires **Node.js 20+**, **pnpm 10**, an installed **Google Chrome or Chromium**, network access to `https://chat.deepseek.com`, and a functioning graphical session for initial login. Your installed version of Pi may require a newer Node version; do not downgrade Node to satisfy this adapter.

Run:

```bash
command -v pi && pi --version
command -v git && git --version
command -v node && node --version
command -v pnpm && pnpm --version
command -v curl && curl --version | head -n 1
command -v python3 && python3 --version

# At least one should resolve to a real executable:
for browser in google-chrome google-chrome-stable chromium chromium-browser; do
  command -v "$browser" || true
done

# Check that Pi's existing config directory is present:
test -d "$HOME/.pi/agent" && echo 'Pi config directory exists'
```

Expected: `pnpm --version` starts with **`10.`**, Node is **20 or newer**, and one browser command resolves. If any is missing, install it with **your existing OS/Nix/Home Manager package setup**, then re-run this check. For pnpm, use a method that supplies pnpm 10; don't blindly upgrade/downgrade the Node environment used by Pi. An optional Nix shell for local inspection is possible, but this guide does **not** change your flake, Home Manager modules, or Pi installation.

If a browser is installed but not on `PATH`, use `DS_CHROME_PATH` with its **actual executable path** in Section 3. No macOS-only browser starter or third-party browser skill is required.

> **Safety check:** Read the repository's README, `SECURITY.md`, and source before authenticating a personal account. This is third-party software handling web-session credentials.

## 2. Install `deepseek-web-api` outside the Pi config

We keep the adapter checkout **outside `~/.pi/agent`**. That makes it less likely that `node_modules`, generated binaries, a browser profile, or secrets are accidentally committed with your Pi configuration.

```bash
DS_REPO="$HOME/.local/opt/deepseek-web-api"
mkdir -p "$HOME/.local/opt"

if [ -e "$DS_REPO" ]; then
  echo "Already exists: $DS_REPO"
  echo 'Inspect that directory; do not clone over it.'
else
  git clone https://github.com/kittors/deepseek-web-api.git "$DS_REPO"
fi

cd "$DS_REPO" || exit 1
printf 'Repository revision: '
git rev-parse --short HEAD

git status --short
pnpm install --frozen-lockfile
pnpm build
```

`--frozen-lockfile` is deliberate: it uses the repository's committed lockfile rather than silently resolving a new dependency graph. If build fails, fix the reported prerequisite/compatibility problem **before** proceeding; don't modify your Pi provider configuration yet.

**Optional upstream checks** (no login required; slower):

```bash
cd "$HOME/.local/opt/deepseek-web-api" || exit 1
pnpm typecheck
pnpm lint
pnpm test
```

The build and unit tests cannot prove that DeepSeek's live private endpoint still works. The live validation in Section 6 addresses that separately.

## 3. Authenticate with DeepSeek Web

The adapter normally manages a **dedicated Chrome/Chromium profile**, using CDP internally. Its default CDP endpoint is port **9333**, not the `9222` port used by some other browser automation tools. Do **not** point it at a regular browser containing unrelated accounts unless you intentionally accept that risk.

From your **host desktop terminal**, run:

```bash
cd "$HOME/.local/opt/deepseek-web-api" || exit 1
umask 077
pnpm login
```

Follow the visible login window and sign in to the **consumer DeepSeek Chat website**. The adapter validates the browser token and cookies and saves authentication state under `data/`. If the browser is not discovered, try the login command again with an explicit executable path, for example:

```bash
cd "$HOME/.local/opt/deepseek-web-api" || exit 1
DS_CHROME_PATH="$(command -v chromium)" pnpm login
```

Replace `chromium` with the browser command that actually exists on your host. `DS_CHROME_PATH` must point to an executable, not a directory. If you are in a terminal-only environment without a GUI, carry out the initial login in a permitted graphical session; do not assume headless first-time login will work.

**Never paste session tokens, cookies, `auth.json`, or `.api-key` into chat, issues, screenshots, or Git.**

## 4. Start the local server (separate Kitty tab)

Open a **second Kitty tab or window**. Keep the server process alive there while Pi is using DeepSeek:

```bash
cd "$HOME/.local/opt/deepseek-web-api" || exit 1
umask 077
HOST=127.0.0.1 PORT=8787 DS_TOOL_REASONING=hidden pnpm start
```

The upstream server uses a locally generated API key to guard its `/v1/*` endpoints. It is **not a DeepSeek developer API key**. The process defaults to a loopback-only HTTP listener; keep it that way.

If you skipped the explicit `pnpm login` step, first startup may initiate login in a visible browser. Wait until the server reports that it is listening before running validation commands. Do not start multiple copies on the same port.

In a different Kitty tab, verify the local process:

```bash
curl --fail --silent --show-error \
  http://127.0.0.1:8787/health | python3 -m json.tool

test -s "$HOME/.local/opt/deepseek-web-api/data/.api-key" \
  && echo 'Local API key exists (value not displayed)'
```

This should return an HTTP success response, but the precise health JSON shape is version-dependent. If the command fails, inspect the server tab first. If the port is already occupied, investigate the owner of `8787`; do not bind to `0.0.0.0` to resolve a local problem.

## 5. Add the provider to your **existing** `models.json` safely

**Do not replace your whole file** with the repository's example: that could erase providers you already use. The next script:

1. Reads the current `~/.pi/agent/models.json` (or initializes an empty one if missing).
2. Refuses to overwrite any **different** existing `deepseek-web` provider.
3. Saves a backup **outside** your Git-tracked Pi config before changing an existing file.
4. Adds just the `deepseek-web` provider.
5. Uses `!cat /absolute/path/to/data/.api-key` so the **key value never appears in Git-tracked JSON** and is read by Pi at request time.
6. Writes the JSON atomically, with restrictive permissions. Existing provider values survive, although whitespace/JSON formatting may be normalized.

Run only after the key file exists (Section 4):

```bash
python3 - <<'PY'
from datetime import datetime
from pathlib import Path
import json
import os
import shlex
import shutil
import tempfile

home = Path.home()
agent = home / '.pi' / 'agent'
path = agent / 'models.json'
keyfile = home / '.local' / 'opt' / 'deepseek-web-api' / 'data' / '.api-key'
backups = home / '.local' / 'state' / 'pi-config-backups'

if not agent.is_dir():
    raise SystemExit(f'Pi config directory not found: {agent}')
if not keyfile.is_file() or keyfile.stat().st_size == 0:
    raise SystemExit(f'Key missing or empty: {keyfile}; start the server first')
if path.is_symlink():
    raise SystemExit('models.json is a symlink; inspect its real target before editing')

config = json.loads(path.read_text(encoding='utf-8')) if path.exists() else {}
if not isinstance(config, dict):
    raise SystemExit('models.json root must be a JSON object')
providers = config.setdefault('providers', {})
if not isinstance(providers, dict):
    raise SystemExit('models.json providers must be a JSON object')

model_template = {
    'reasoning': True,
    'input': ['text'],
    'thinkingLevelMap': {
        'off': 'none', 'minimal': 'low', 'low': 'low',
        'medium': 'medium', 'high': 'high',
        'xhigh': None, 'max': None,
    },
    'cost': {'input': 0, 'output': 0, 'cacheRead': 0, 'cacheWrite': 0},
}
models = []
for model_id, model_name in (
    ('deepseek-v4-flash', 'DeepSeek Web Flash'),
    ('deepseek-v4-pro', 'DeepSeek Web Pro'),
):
    models.append({'id': model_id, 'name': model_name, **model_template})

provider = {
    'baseUrl': 'http://127.0.0.1:8787/v1',
    'api': 'openai-completions',
    'apiKey': '!cat ' + shlex.quote(str(keyfile)),
    'compat': {
        'supportsDeveloperRole': True,
        'supportsReasoningEffort': True,
        'supportsUsageInStreaming': False,
        'supportsStore': False,
    },
    'models': models,
}

if 'deepseek-web' in providers:
    if providers['deepseek-web'] == provider:
        print('Already configured: deepseek-web (no changes).')
        raise SystemExit(0)
    raise SystemExit(
        'deepseek-web already exists with different settings. '
        'Review and merge it manually; existing configuration was not changed.'
    )

if path.exists():
    backups.mkdir(parents=True, exist_ok=True, mode=0o700)
    backup = backups / (
        'models.json.' + datetime.now().strftime('%Y%m%d-%H%M%S-%f') + '.bak'
    )
    shutil.copy2(path, backup)
    backup.chmod(0o600)
    print(f'Backup written: {backup}')

providers['deepseek-web'] = provider
fd, tmp_name = tempfile.mkstemp(prefix='.models.json.', suffix='.tmp', dir=agent)
try:
    os.fchmod(fd, 0o600)
    with os.fdopen(fd, 'w', encoding='utf-8') as stream:
        json.dump(config, stream, indent=2, ensure_ascii=False)
        stream.write('\n')
        stream.flush()
        os.fsync(stream.fileno())
    os.replace(tmp_name, path)
finally:
    if os.path.exists(tmp_name):
        os.unlink(tmp_name)

print(f'Added deepseek-web provider to {path} (existing providers retained).')
PY
```

**Why `!cat` instead of exporting a key?** Pi's custom-model configuration supports a leading shell command for secret resolution. This avoids putting the key value into a tracked file and works regardless of whether Kitty starts fish, Bash, or zsh. It does execute a local `cat` command when Pi resolves the provider key; that is negligible local overhead and **does not consume LLM tokens**. The path to your local proxy key, but not its contents, is stored in `models.json`.

If you use a **different agent directory** selected by a Pi environment/config override, this guide's default `~/.pi/agent` path must be changed consistently. It does not deliberately modify your other agent configuration.

## 6. Verify HTTP first, then verify Pi

### A. Test the adapter's model listing

In a host terminal with the server running:

```bash
DS_REPO="$HOME/.local/opt/deepseek-web-api"
API_KEY="$(cat "$DS_REPO/data/.api-key")"

curl --fail --silent --show-error \
  'http://127.0.0.1:8787/v1/models' \
  -H "Authorization: Bearer $API_KEY" \
  | python3 -m json.tool

unset API_KEY
```

Expect the local adapter's public model IDs, including `deepseek-v4-flash` and `deepseek-v4-pro`. This only checks the local server and auth layer; it does not prove upstream completions.

### B. Test one **actual** DeepSeek completion

```bash
DS_REPO="$HOME/.local/opt/deepseek-web-api"
API_KEY="$(cat "$DS_REPO/data/.api-key")"

curl --fail --silent --show-error \
  'http://127.0.0.1:8787/v1/chat/completions' \
  -H "Authorization: Bearer $API_KEY" \
  -H 'Content-Type: application/json' \
  --data '{
    "model": "deepseek-v4-flash",
    "messages": [{"role":"user","content":"Reply with exactly: PI-DEEPSEEK-OK"}],
    "stream": false
  }' | python3 -m json.tool

unset API_KEY
```

A successful JSON response with a nonempty assistant message confirms that the local proxy can authenticate and call DeepSeek Web **at this moment**. Don't assume the model will return your test marker exactly. If you get a 401/502, diagnose the adapter before debugging Pi.

### C. Verify the Pi registry and selection

```bash
python3 -m json.tool "$HOME/.pi/agent/models.json" >/dev/null \
  && echo 'models.json syntax valid'

pi --list-models deepseek-web
```

Then start Pi normally from the project directory where you intend to work:

```bash
cd /path/to/your/project
pi
```

Inside **Pi's prompt**, enter `/model`, search for **DeepSeek Web Flash**, and select it. Ask a simple text-only question. Alternatively, launch a session with explicit model selection:

```bash
pi --provider deepseek-web --model deepseek-v4-flash
```

Check `pi --help` for flags supported by your installed Pi release. Use the actual project path in place of `/path/to/your/project`; this placeholder is **not** a directory you should create.

**Important acceptance test:** Start a new Pi session with DeepSeek selected; ask two related questions in sequence to check continuity. Next, test a *read-only* tool call in a disposable project (for example, ask Pi to read a test text file), and inspect whether the tool call actually executes. Do not use production files for this first tool-call test.

### D. Acceptance checklist

- [ ] `pnpm build` completes.
- [ ] First-party Chrome/Chromium login succeeds; no secrets were copied into Pi configuration.
- [ ] `http://127.0.0.1:8787/health` answers successfully.
- [ ] A local `.api-key` file exists and is **not** pasted into Git or shared.
- [ ] `GET /v1/models` responds with expected model IDs.
- [ ] A real `POST /v1/chat/completions` produces assistant text.
- [ ] `models.json` parses, and all previously configured providers remain present.
- [ ] `pi --list-models deepseek-web` shows the custom models.
- [ ] `/model` allows DeepSeek Web selection.
- [ ] A text-only Pi chat works and a second turn responds coherently.
- [ ] A read-only tool call is tested in an isolated, disposable project.
- [ ] Your existing OpenAI provider still works after switching back via `/model`.

## 7. Daily usage from Kitty

When you want to use DeepSeek through Pi:

1. Start the adapter in a host Kitty tab with the Section 4 command (unless it is already running).
2. Open Pi in your normal project directory (another tab).
3. Use `/model` to switch to `deepseek-web/deepseek-v4-flash` or `deepseek-web/deepseek-v4-pro` as shown in Pi's picker.
4. Switch back to your existing GPT/OpenAI provider with `/model` when needed.
5. Stop the local adapter with `Ctrl+C` in its terminal tab when finished, unless you intentionally want it running.

### Flash vs Pro

The names `deepseek-v4-flash` and `deepseek-v4-pro` are **adapter-specific public model IDs**, not promises about official paid API product names. The adapter currently maps them to DeepSeek Web `default` and `expert` modes, respectively. Its upstream documentation says web search is supported only for the Flash/default mapping. Start with Flash for integration testing and compare Pro separately.

### Thinking controls and usage accounting

Pi may offer `off`, `minimal`, `low`, `medium`, or `high`. The adapter maps **`off` to DeepSeek thinking off** and **all supported non-off levels to thinking on**. They are not distinct DeepSeek reasoning budgets. Reliable **input token counts are unavailable** from DeepSeek Web; apparent `0` input tokens should not be interpreted as zero computational/token consumption. Prompt caching and strict schema enforcement are not implemented.

### Existing `AGENTS.md`, skills, extensions, and MCP-backed tools

You do **not** need to add global DeepSeek instructions to `AGENTS.md`. Pi will pass its configured instructions and available tool definitions to the selected provider as appropriate. More enabled tools or verbose `AGENTS.md` content can make requests larger; the adapter does not make those instructions free or invisible to the upstream model. Tool schemas exposed through Pi/MCP bridges reach the adapter as ordinary tool definitions; the adapter itself is **not an MCP server**.

For your existing sandbox/permission stack, keep the same approvals and host protections. The local proxy runs as a **separate host process**; Pi's sandbox on `bash`/`write` tools does not automatically sandbox the proxy or the browser. Simulated tool calls must still be constrained by Pi's actual permission and sandbox enforcement. Never treat model instructions as a security boundary.

## 8. Security checks and Git hygiene

Inspect paths without revealing their contents:

```bash
DS_REPO="$HOME/.local/opt/deepseek-web-api"
cd "$DS_REPO" || exit 1

git check-ignore -v data/auth.json data/.api-key data/sessions.json \
  data/chrome-profile/ || true

# File metadata only: permissions and paths, not secrets
for f in data/auth.json data/.api-key data/sessions.json; do
  test -e "$f" && stat -c '%a %n' "$f"
done

# Verify Pi JSON has a command, not an embedded key
python3 - <<'PY'
import json
from pathlib import Path
p = Path.home() / '.pi' / 'agent' / 'models.json'
spec = json.loads(p.read_text())['providers']['deepseek-web']
print('Provider:', spec['api'])
print('Key method:', 'shell-command' if spec['apiKey'].startswith('!cat ') else 'CHECK MANUALLY')
print('Base URL:', spec['baseUrl'])
PY
```

Recommended safeguards:

- Keep `HOST=127.0.0.1`; **never** expose port `8787` to other hosts or the internet.
- Protect `data/auth.json`, `.api-key`, `sessions.json`, and `chrome-profile/`. They are secret/private state; `data/` is Git-ignored upstream, but always verify before committing anything.
- Do not check out the adapter inside your tracked Pi config. Do not create project-level copies of `models.json` with real credentials.
- Do not paste `curl -v` headers, cookies, tokens, raw request transcripts, private prompts, or `DS_DEBUG=1` logs into an issue or a chat.
- If you run Pi in an untrusted project, its project configuration/extensions may introduce executable behavior. Review your loaded Pi extensions and sandbox settings before allowing file changes.
- The official DeepSeek API and web adapter have different usage limits and policies; verify the rules governing the account you use.

## 9. Troubleshooting

| Symptom | Likely cause / action |
|---|---|
| `pnpm: command not found` | Install pnpm **10** through your existing host toolchain; rerun preflight. |
| Build fails | Check `node --version` (20+), `pnpm --version` (10.x), dependency errors, then `pnpm typecheck` and `pnpm build`. |
| Browser not found | Use `DS_CHROME_PATH="$(command -v chromium)" pnpm login` (or your actual Chrome binary). |
| Login keeps repeating | Log in within the **adapter's** managed browser profile, not your regular unrelated Chrome profile; retry `pnpm login`. |
| `/health` connection refused | Server not started, wrong port, or startup blocked waiting for login; inspect its terminal. |
| `401 Invalid API key` | Pi is reading a key from a different checkout, `DS_API_KEY` overrides the generated key, or file was regenerated. Confirm paths; never share key contents. |
| Upstream 401 | DeepSeek web session may have expired; rerun `pnpm login`. |
| `502`, PoW challenge, or empty stream | DeepSeek private endpoints/worker may have changed. Refer to upstream troubleshooting; a code update may be required. Avoid invented PoW bypasses. |
| Model absent in `/model` | `models.json` malformed, incorrect Pi config directory, unreadable/missing `.api-key`, or auth command resolution failed. Check `pi --list-models deepseek-web`. |
| Raw `<tool_call>` text or broken tool execution | Expected failure mode of **prompt-emulated** tool calls. Keep `DS_TOOL_REASONING=hidden`; try a smaller read-only test. |
| Session loses continuity | Preserve `data/sessions.json` and complete, stable Pi conversation history. Avoid concurrent turns in one DeepSeek session. |
| Pi security policy blocks an operation | Investigate your **Pi tool** sandbox/permissions separately. Do not disable protections to make the model comply. |

For diagnosis, temporarily run:

```bash
cd "$HOME/.local/opt/deepseek-web-api" || exit 1
# Only if the normal server is STOPPED:
HOST=127.0.0.1 PORT=8787 DS_DEBUG=1 DS_TOOL_REASONING=hidden pnpm start
```

Debug logs may contain private data; keep them local and turn off debug mode afterward. **Do not** commit a debug log or upload browser/session state.

## 10. Updating the adapter, safely

The user-facing Pi config does not need to change for every adapter update. Upstream does not promise stable private DeepSeek endpoints, so review code and changelogs before pulling new revisions.

1. Stop the adapter (`Ctrl+C`).
2. Keep the existing `data/` directory intact.
3. Examine repository status and changes.
4. Rebuild; log in again only if required; rerun the live checks.

```bash
cd "$HOME/.local/opt/deepseek-web-api" || exit 1

git status --short
git log -1 --format='%h %cs %s'

# Only proceed when you have reviewed changes and are ready to update:
git fetch origin
git log --oneline HEAD..origin/main
# After review:
git pull --ff-only
pnpm install --frozen-lockfile
pnpm typecheck
pnpm test
pnpm build
```

Do not run `git clean -fdx` or remove `data/`: those actions can destroy saved authentication and conversation state. Back up private runtime state securely before any destructive maintenance; never put it into a public repository or cloud sync folder without adequate encryption.

## 11. Rollback / uninstall (without damaging other Pi providers)

**Rollback only the Pi integration:** stop the server, then remove only the `deepseek-web` entry from `~/.pi/agent/models.json`. The following operation is idempotent and leaves other providers intact:

```bash
python3 - <<'PY'
from pathlib import Path
from datetime import datetime
import json, os, shutil, tempfile

home = Path.home()
p = home / '.pi' / 'agent' / 'models.json'
if not p.is_file() or p.is_symlink():
    raise SystemExit('No regular models.json to edit; inspect manually')
data = json.loads(p.read_text(encoding='utf-8'))
providers = data.get('providers')
if not isinstance(providers, dict) or 'deepseek-web' not in providers:
    print('deepseek-web not present; no changes')
    raise SystemExit(0)
backups = home / '.local' / 'state' / 'pi-config-backups'
backups.mkdir(parents=True, exist_ok=True, mode=0o700)
bak = backups / ('models.json.before-deepseek-removal.' + datetime.now().strftime('%Y%m%d-%H%M%S-%f') + '.bak')
shutil.copy2(p, bak)
bak.chmod(0o600)
del providers['deepseek-web']
fd, temp = tempfile.mkstemp(prefix='.models.json.', dir=p.parent)
try:
    os.fchmod(fd, 0o600)
    with os.fdopen(fd, 'w', encoding='utf-8') as stream:
        json.dump(data, stream, ensure_ascii=False, indent=2)
        stream.write('\n')
        stream.flush()
        os.fsync(stream.fileno())
    os.replace(temp, p)
finally:
    if os.path.exists(temp):
        os.unlink(temp)
print(f'Removed only deepseek-web. Backup: {bak}')
PY
```

Then verify `pi --list-models deepseek-web` no longer lists the custom provider and your other Pi models still appear. **Do not delete the adapter checkout** unless you intentionally want to delete or separately secure its `data/` folder: that folder contains logged-in session credentials and chat lineage. If you remove it, make sure its keys/tokens and backup copies are handled according to your security requirements.

## 12. Quick reference

| Action | Command / location |
|---|---|
| Start proxy | `cd ~/.local/opt/deepseek-web-api && HOST=127.0.0.1 PORT=8787 DS_TOOL_REASONING=hidden pnpm start` |
| Reauthenticate | `cd ~/.local/opt/deepseek-web-api && pnpm login` |
| Proxy health | `curl -fsS http://127.0.0.1:8787/health` |
| Pi model listing | `pi --list-models deepseek-web` |
| Start Pi with DeepSeek | `pi --provider deepseek-web --model deepseek-v4-flash` |
| Interactive model switch | `/model` inside Pi |
| Quit proxy | `Ctrl+C` in its Kitty tab |
| Pi provider file | `~/.pi/agent/models.json` |
| Proxy secrets | `~/.local/opt/deepseek-web-api/data/` — **do not share** |

### Primary references

- Repository: https://github.com/kittors/deepseek-web-api
- Upstream Pi guide: https://github.com/kittors/deepseek-web-api/blob/main/docs/pi.md
- Architecture: https://github.com/kittors/deepseek-web-api/blob/main/docs/architecture.md
- Troubleshooting: https://github.com/kittors/deepseek-web-api/blob/main/docs/troubleshooting.md
- Security policy: https://github.com/kittors/deepseek-web-api/blob/main/SECURITY.md
- Pi model configuration: https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/models.md
- Pi CLI: https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/cli.md

**Verification boundary:** These steps were checked against available upstream documentation and inspected for shell/JSON safety. They have **not** been executed against your local Pi installation or a live DeepSeek login. The live acceptance tests above are how you establish actual functionality on your machine.
