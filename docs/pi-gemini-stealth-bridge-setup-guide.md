# Pi Coding Agent — Gemini Stealth Bridge Setup Guide

**Scope:** Use the consumer Gemini web app through a local OpenAI-compatible proxy as a selectable model in Pi Coding Agent. This guide installs **only** [`azrinaldi/gemini-stealth-bridge`](https://github.com/azrinaldi/gemini-stealth-bridge); it does not install `web-ai-skills`, the Google Gemini developer API, or a separate Pi extension.

**Verified against documentation on:** 16 November 2026  
**Latest published release verified:** `v0.4.3` (published 13 November 2026). The repository's `main` README also describes newer unreleased changes; this guide deliberately uses the published release.  
**Target:** Linux desktop with Kitty terminal, standard Pi configuration at `~/.pi/agent`, and an existing Pi setup (including any DeepSeek Web provider).  
**Installation style:** Verified prebuilt binary; no Docker, `sudo`, Go toolchain, Nix/Home Manager, or persistent repository checkout required.  
**Primary API:** `openai-completions` at `http://127.0.0.1:20133/v1`.

> **Important:** This is an **unofficial bridge** to `gemini.google.com`, using Google web-session credentials. It is not an official Google Gemini API. Changes to Google's private web protocols, session expiration, quota limits, or account enforcement can make it fail or restrict access. Review Google's applicable terms and use only accounts you control. No live login or end-to-end Pi execution was performed while preparing this document.

---

## 0. What the completed setup does

```text
Kitty terminal → Pi Coding Agent → ~/.pi/agent/models.json
                                         │
                                Gemini Web provider
                                         │
                              127.0.0.1:20133/v1
                                         │
                            gemini-stealth-bridge (gsb)
                                         │
                     Your authenticated gemini.google.com session
```

The bridge is a **separate local daemon**, not a Pi-native extension. Pi speaks OpenAI-compatible HTTP to it; the daemon speaks to Gemini Web. It can provide streaming chat, reasoning levels, continuity cache, prompt-emulated tool calls, model discovery, and account-quota observations. Pi still executes file/shell tools locally. Function calling is **not** native Google developer-API function calling and must not be assumed safe for unsupervised editing.

This guide **does not modify** your existing GPT/OpenAI or `deepseek-web-api` providers, `AGENTS.md`, permissions, sandbox rules, extensions, or the default model. You explicitly select Gemini with Pi's `/model` picker.

### File locations

| Purpose | Location | Git-track? |
|---|---|---|
| Pi model/provider configuration | `~/.pi/agent/models.json` | Fine **if no secrets are in it** |
| Bridge executable and launcher | `~/.local/opt/gemini-stealth-bridge/` | No need |
| Bridge runtime data and Google session | `~/.local/share/gemini-stealth-bridge/` | **Never** |
| Local HTTP pool key | `~/.local/share/gemini-stealth-bridge/pool.key` | **Never** |
| Optional user systemd unit | `~/.config/systemd/user/gemini-stealth-bridge.service` | Optional, contains no secret |
| Backups of Pi model configuration | `~/.local/state/pi-provider-backups/` | **Never** |

Keeping bridge runtime data outside `~/.pi/agent` avoids accidental Git tracking and keeps the Google session separate from your Pi configuration.

---

## 1. Prerequisites and command-shell convention

Run commands in **Kitty**, not in the Pi chat prompt. These examples use **Bash syntax**; Kitty is a terminal emulator and may host Fish, Zsh, Bash, etc. If your normal shell is Fish, enter `bash` first, complete the steps in that Bash shell, and type `exit` at the end.

Required:

- Linux `x86_64` or `aarch64`; current graphical desktop session.
- A browser supported by the bridge for interactive Google sign-in (Chrome/Chromium/Brave/Edge, etc.).
- A Google account that can use <https://gemini.google.com/app>.
- `curl`, `sha256sum`, `install`, `python3`, and `openssl` or Python's standard library for secret generation.
- Pi already installed and working from `~/.pi/agent`.

Check your environment:

```bash
set -euo pipefail
uname -s
uname -m
command -v pi
command -v curl
command -v sha256sum
command -v python3
command -v chromium || command -v google-chrome || command -v brave-browser || true
```

**Expected:** `Linux`, an `x86_64` or `aarch64` architecture, and a valid executable path for Pi. If no supported browser is installed, install one using your normal system package manager. Do not install a second copy of Pi or change its configuration location.

Optional Fedora package check (not a mandatory installation step):

```bash
rpm -q python3 curl coreutils
```

---

## 2. Install and verify the published bridge binary

Use a fixed release for a reproducible first install. The asset name must be preserved **until after checksum verification**, because the published `SHA256SUMS` uses asset filenames, not simply `gsb`.

```bash
set -euo pipefail
RELEASE='v0.4.3'
REPO_URL='https://github.com/azrinaldi/gemini-stealth-bridge'
BIN_DIR="$HOME/.local/opt/gemini-stealth-bridge"

case "$(uname -m)" in
  x86_64)  ASSET='gsb_linux_amd64' ;;
  aarch64) ASSET='gsb_linux_arm64' ;;
  *) printf 'Unsupported architecture: %s\n' "$(uname -m)" >&2; exit 1 ;;
esac

umask 077
mkdir -p "$BIN_DIR"
chmod 700 "$BIN_DIR"
TMPDIR_GSB="$(mktemp -d)"
trap 'rm -rf "$TMPDIR_GSB"' EXIT

curl --fail --location --show-error --silent \
  "$REPO_URL/releases/download/$RELEASE/$ASSET" \
  --output "$TMPDIR_GSB/$ASSET"

curl --fail --location --show-error --silent \
  "$REPO_URL/releases/download/$RELEASE/SHA256SUMS" \
  --output "$TMPDIR_GSB/SHA256SUMS"

(
  cd "$TMPDIR_GSB"
  grep -F "  $ASSET" SHA256SUMS >/dev/null || {
    printf 'Expected asset was not listed in SHA256SUMS\n' >&2; exit 1;
  }
  sha256sum --check --ignore-missing SHA256SUMS
)

if [ -e "$BIN_DIR/gsb" ]; then
  cp -p "$BIN_DIR/gsb" "$BIN_DIR/gsb.before-$RELEASE-$(date +%Y%m%d%H%M%S)"
fi
install -m 0755 "$TMPDIR_GSB/$ASSET" "$BIN_DIR/gsb.next"
mv -f "$BIN_DIR/gsb.next" "$BIN_DIR/gsb"
"$BIN_DIR/gsb" --version
```

**Expected:** Checksum verification reports `OK`, followed by a `gsb --version` report. If the checksum does not match, stop: do not run the binary. This approach does **not** require Go 1.26 (that is required only for source builds).

**Why not `curl ... -o gsb` then `sha256sum -c SHA256SUMS`?** That would change the filename and could skip or fail verification. The installation block above ensures the published checksum is checked against the downloaded asset with its original name.

---

## 3. Prepare a private runtime directory and strong local key

Set the runtime directory in the current Bash session:

```bash
export BRIDGE_HOME="$HOME/.local/share/gemini-stealth-bridge"
umask 077
mkdir -p "$BRIDGE_HOME/accounts"
chmod 700 "$BRIDGE_HOME" "$BRIDGE_HOME/accounts"
```

Create a pool key **only if none exists**. This is safe to run again; it does not rotate an existing working key:

```bash
python3 - <<'PY'
from pathlib import Path
import secrets
import os

home = Path.home() / '.local/share/gemini-stealth-bridge'
key = home / 'pool.key'
home.mkdir(parents=True, exist_ok=True, mode=0o700)
os.chmod(home, 0o700)
try:
    fd = os.open(key, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
except FileExistsError:
    print('Existing pool key retained.')
else:
    with os.fdopen(fd, 'w', encoding='ascii') as f:
        f.write(secrets.token_urlsafe(48) + '\n')
    print('Generated new private pool key.')
os.chmod(key, 0o600)
PY
```

**Do not print or paste the key into chat, Git, screenshots, bug reports, or shell history.** The bridge also accepts predictable per-account aliases such as `gsb-acc1`; these are routing aliases, **not secure authentication keys**. The guide uses a generated pool key and binds only to localhost.

---

## 4. Sign in to Gemini Web and verify the account

Do this from your normal **graphical user desktop**, not from a headless/remote shell. Run:

```bash
export BRIDGE_HOME="$HOME/.local/share/gemini-stealth-bridge"
"$HOME/.local/opt/gemini-stealth-bridge/gsb" --login acc1
```

1. A dedicated, temporary browser profile should open.
2. Sign into your Google account and complete any 2FA prompts.
3. Use Gemini Web normally if prompted, and follow the terminal's capture/confirmation instructions.
4. The bridge saves the authorized session at `$BRIDGE_HOME/accounts/acc1.json` and removes the disposable browser profile.
5. **Do not manually extract Google cookies unless the recommended login method fails and you have reviewed the repository's security advice.**

Probe the stored session:

```bash
"$HOME/.local/opt/gemini-stealth-bridge/gsb" --probe
"$HOME/.local/opt/gemini-stealth-bridge/gsb" --accounts
```

**Expected:** The probe returns one or more available Gemini Web models/quota observations **without authentication errors**. Merely having an `acc1.json` file is **not** proof that the session works. If the probe fails because of authentication, run `--login acc1` again and inspect the specific error. **Do not** use `--unquarantine` to bypass an upstream account restriction.

The bridge's default runtime path is `~/.gemini-stealth-bridge`; all commands in this guide explicitly set `BRIDGE_HOME` to avoid accidentally creating a second account/session store.

---

## 5. Create a secure, reusable daemon launcher

This launcher is useful both for running the bridge in Kitty and for the optional systemd user service. It loads the private key without including it in any Git-tracked file and refuses an absent key.

```bash
BIN_DIR="$HOME/.local/opt/gemini-stealth-bridge"
cat > "$BIN_DIR/start-gsb" <<'BASH'
#!/usr/bin/env bash
set -euo pipefail
umask 077
export BRIDGE_HOME="$HOME/.local/share/gemini-stealth-bridge"
export BRIDGE_HOST='127.0.0.1'
export BRIDGE_PORT='20133'
export BRIDGE_WEB_UI='1'
export BRIDGE_TOOL_SCHEMA_COMPACT='1'
export BRIDGE_LIVE_TOOL_CALLS='0'
export BRIDGE_REJECT_DOWNGRADE='1'
KEY_FILE="$BRIDGE_HOME/pool.key"
[ -s "$KEY_FILE" ] || { echo 'Missing bridge pool key' >&2; exit 1; }
export BRIDGE_API_KEY="$(cat "$KEY_FILE")"
exec "$HOME/.local/opt/gemini-stealth-bridge/gsb" --web
BASH
chmod 700 "$BIN_DIR/start-gsb"
bash -n "$BIN_DIR/start-gsb"
```

Keep default rate limits, retry limits, session caching, and quota preflight settings; do not increase them just to force requests through. **Do not run** `gsb --web` without the launcher unless you separately set `BRIDGE_API_KEY`: the stock default `public` can permit unauthenticated requests.

### Run in the foreground first

In one Kitty window:

```bash
"$HOME/.local/opt/gemini-stealth-bridge/start-gsb"
```

The process stays in that terminal. Open a **second Kitty window** for the next steps. The daemon should bind only to `127.0.0.1:20133`. If you receive a lock/bind error, check for an already-running bridge; do not start multiple daemon instances for the same `BRIDGE_HOME`.

---

## 6. Test localhost API health, authentication, models, and generation

### 6.1 Health and bind address

In the second Kitty window:

```bash
curl --fail --silent --show-error http://127.0.0.1:20133/health
curl --silent --show-error http://127.0.0.1:20133/readyz
ss -ltn | grep ':20133'
```

**Expected:** `/health` returns a service-health JSON object; `ss` shows `127.0.0.1:20133`, **not** `0.0.0.0:20133`. A healthy local daemon is **not** proof that Gemini generation works.

### 6.2 List the models your Google account can actually access

Use Python so the key does not appear in a command-line `curl -H` argument or terminal output:

```bash
python3 - <<'PY'
from pathlib import Path
from urllib.request import Request, urlopen
import json

key = (Path.home() / '.local/share/gemini-stealth-bridge/pool.key').read_text().strip()
req = Request('http://127.0.0.1:20133/v1/models',
              headers={'Authorization': f'Bearer {key}'})
with urlopen(req, timeout=30) as response:
    models = json.load(response)['data']
for model in models:
    print(f"{model['id']:<32} {model.get('display_name') or model.get('description') or ''}")
print(f'Total available: {len(models)}')
PY
```

**Expected:** One or more live IDs. **Use these exact IDs** for Pi. Do not blindly assume old aliases such as `gemini-pro` or `gemini-flash` will always resolve: Gemini Web tier names change. As of the bridge's November 2026 documentation, names such as `gemini-4-carbon` and `gemini-3.1-pro` have appeared, but your account's live response is authoritative.

### 6.3 Verify actual generation (not just `/health`)

The following makes **one small** upstream query against the **first** available model. It does not invoke Pi or call OpenAI:

```bash
python3 - <<'PY'
from pathlib import Path
from urllib.request import Request, urlopen
import json

base = 'http://127.0.0.1:20133/v1'
key = (Path.home() / '.local/share/gemini-stealth-bridge/pool.key').read_text().strip()
headers = {'Authorization': f'Bearer {key}', 'Content-Type': 'application/json'}
with urlopen(Request(base + '/models', headers=headers), timeout=30) as r:
    ids = [m['id'] for m in json.load(r).get('data', []) if isinstance(m.get('id'), str)]
if not ids:
    raise SystemExit('No available model; complete --probe troubleshooting first')
print('Testing model:', ids[0])
body = json.dumps({'model': ids[0],
                   'messages': [{'role':'user','content':'Reply with the word READY only.'}],
                   'stream': False}).encode()
with urlopen(Request(base + '/chat/completions', data=body, headers=headers, method='POST'),
             timeout=180) as r:
    result = json.load(r)
print('Model response:', result['choices'][0]['message'].get('content'))
PY
```

**Expected:** A nonempty model response. It need not match `READY` perfectly; successful upstream generation is the acceptance criterion. A `401`, `403`, `429`, `502`, or `503` is **not** a successful test—see troubleshooting below.

---

## 7. Safely merge the bridge into your existing Pi `models.json`

**Do not replace `~/.pi/agent/models.json` with a single-provider example.** You already have other providers and may have an existing DeepSeek setup. The script below:

1. Reads the live `GET /v1/models` list using your private pool key.
2. Builds a provider named `gemini-web` from the actual available model IDs.
3. Retains all other providers and unrelated top-level fields.
4. Creates a private, timestamped backup before changing a file.
5. Uses a temporary file and atomic replacement; does not store the key's value in `models.json`.
6. Refuses to replace an existing `gemini-web` provider if it points at a different URL.

If you previously edited `models.json` manually, verify it is valid **JSON** (not JSON with comments) before proceeding.

```bash
python3 - <<'PY'
from pathlib import Path
from urllib.request import Request, urlopen
import datetime as dt
import json
import os
import shlex
import shutil
import tempfile

home = Path.home()
config = home / '.pi/agent/models.json'
keyfile = home / '.local/share/gemini-stealth-bridge/pool.key'
backup_dir = home / '.local/state/pi-provider-backups'
base = 'http://127.0.0.1:20133/v1'
key = keyfile.read_text(encoding='ascii').strip()
if not key:
    raise SystemExit('Empty pool key')

req = Request(base + '/models', headers={'Authorization': 'Bearer ' + key})
with urlopen(req, timeout=30) as response:
    data = json.load(response)
remote_models = data.get('data', [])
if not isinstance(remote_models, list) or not remote_models:
    raise SystemExit('No live models. Do not write a broken Pi provider.')

model_entries = []
for m in remote_models:
    if not isinstance(m, dict) or not isinstance(m.get('id'), str):
        continue
    if not m['id'].strip() or not m['id'].startswith('gemini-'):
        continue
    caps = m.get('capabilities') or {}
    if not isinstance(caps, dict):
        caps = {}
    think = m.get('thinking') or {}
    if not isinstance(think, dict):
        think = {}
    reasoning = bool(m.get('reasoning', caps.get('reasoning', False)))
    item = {
        'id': m['id'],
        'name': 'Gemini Web — ' + str(m.get('display_name') or m['id']),
        'reasoning': reasoning,
        'input': ['text', 'image'] if caps.get('vision') is True else ['text'],
        # Conservative Pi-side planning caps, NOT promises from Google.
        'contextWindow': min(131072, int(m.get('context_window') or m.get('context_length') or 131072)),
        'maxTokens': min(16384, int(m.get('max_output_tokens') or m.get('max_completion_tokens') or 16384)),
        'cost': {'input': 0, 'output': 0, 'cacheRead': 0, 'cacheWrite': 0},
    }
    raw_levels = caps.get('thinkingLevels') or think.get('efforts') or []
    levels = set(raw_levels) if isinstance(raw_levels, list) else set()
    if reasoning and levels:
        item['thinkingLevelMap'] = {
            'off': None,
            'minimal': 'low' if 'low' in levels else None,
            'low': 'low' if 'low' in levels else None,
            'medium': 'medium' if 'medium' in levels else None,
            'high': 'high' if 'high' in levels else None,
            'xhigh': None,
            'max': None,
        }
    model_entries.append(item)

if not model_entries:
    raise SystemExit('No recognized Gemini model IDs; inspect /v1/models first.')

if config.exists():
    raw = json.loads(config.read_text(encoding='utf-8'))
    if not isinstance(raw, dict):
        raise SystemExit('Existing models.json must contain a JSON object')
else:
    raw = {}

providers = raw.setdefault('providers', {})
if not isinstance(providers, dict):
    raise SystemExit('Existing "providers" is not a JSON object')
old_provider = providers.get('gemini-web')
if old_provider is not None and (
    not isinstance(old_provider, dict) or
    old_provider.get('baseUrl') != base
):
    raise SystemExit('Existing gemini-web provider has an unexpected URL: refusing overwrite')

providers['gemini-web'] = {
    'name': 'Gemini Web (local bridge)',
    'baseUrl': base,
    'api': 'openai-completions',
    'apiKey': '!cat ' + shlex.quote(str(keyfile)),
    'authHeader': True,
    'compat': {
        'supportsDeveloperRole': True,
        'supportsReasoningEffort': True,
        'supportsUsageInStreaming': False,
        'supportsStore': False,
        'supportsStrictMode': False,
    },
    'models': model_entries,
}

config.parent.mkdir(parents=True, exist_ok=True)
backup_dir.mkdir(parents=True, exist_ok=True, mode=0o700)
os.chmod(backup_dir, 0o700)
if config.exists():
    stamp = dt.datetime.now().strftime('%Y%m%d-%H%M%S-%f')
    backup = backup_dir / f'models.before-gemini-{stamp}.json'
    shutil.copyfile(config, backup)
    os.chmod(backup, 0o600)
    print('Private backup:', backup)

fd, tempname = tempfile.mkstemp(prefix='.models.gemini.', suffix='.json', dir=config.parent)
try:
    with os.fdopen(fd, 'w', encoding='utf-8') as f:
        json.dump(raw, f, indent=2, ensure_ascii=False)
        f.write('\n')
    os.chmod(tempname, (config.stat().st_mode & 0o777) if config.exists() else 0o600)
    os.replace(tempname, config)
except Exception:
    Path(tempname).unlink(missing_ok=True)
    raise
print('Updated:', config)
print('Gemini models:', ', '.join(m['id'] for m in model_entries))
print('Other providers retained:', ', '.join(p for p in providers if p != 'gemini-web'))
PY
```

Validate the JSON and review only the non-secret provider summary:

```bash
python3 -m json.tool "$HOME/.pi/agent/models.json" >/dev/null
python3 - <<'PY'
from pathlib import Path
import json
p = Path.home() / '.pi/agent/models.json'
cfg = json.loads(p.read_text())
for name, provider in cfg.get('providers', {}).items():
    print(name, '→', provider.get('baseUrl', '(built-in/no custom URL)'),
          ':', len(provider.get('models', [])), 'explicit models')
PY
```

**Cost note:** The `cost: 0` entry means **no metered Gemini Developer API charges are represented in this bridge configuration**. It does not mean your Google subscription has zero cost, or that requests have no quotas or resource consumption. Gemini Web token usage is estimated rather than authoritative.

**Context limits note:** This guide deliberately caps advertised Pi context at 131,072 tokens and output at 16,384, even if the bridge advertises larger theoretical maxima, because upstream Gemini Web limits and the bridge's prompt offloading may vary. This is a conservative client planning choice, not a hard limit enforced by the bridge.

---

## 8. Activate Gemini in Pi (without replacing your main model)

In a **new Kitty window**, run:

```bash
pi --list-models gemini-web
```

If models are listed, start Pi in a **disposable test directory** to avoid touching your real projects during acceptance testing:

```bash
mkdir -p "$HOME/Projects/pi-gemini-acceptance"
cd "$HOME/Projects/pi-gemini-acceptance"
pi
```

Inside Pi:

1. Open `/model`.
2. Find **Gemini Web (local bridge)** and select a model from the actual discovered IDs.
3. First send: `Reply with exactly: GEMINI_PI_OK. Do not call tools.`
4. Next send: `Explain, without using any tools, what read-only checks you would perform before editing a Git repository.`
5. Only after those succeed, ask for a read-only local tool call, such as `Show the current working directory using pwd. Do not change any files.`
6. Check that a tool request is displayed as an actual Pi tool invocation, **not** as raw tool-call markup in the text response.
7. Return to your existing GPT or DeepSeek provider using `/model`.

**Expected:** The Gemini model is selectable and answers normally, then optionally invokes Pi tools on a small read-only test. Do not assume that one successful tool invocation establishes reliable complex autonomous tool use.

**No other model provider needs to change.** Avoid changing `~/.pi/agent/settings.json` or `/model`'s saved default unless you intentionally want Gemini for every new session.

### Models missing from `/model`?

- Confirm `/v1/models` succeeds with the bridge key (Step 6).
- Confirm the added model IDs exactly match the live endpoint, not just aliases.
- Confirm `models.json` parses and `apiKey` is present.
- Confirm `pi --list-models gemini-web` recognizes the provider.
- If your Pi version is old, inspect its custom-provider syntax and update Pi deliberately; don't overwrite all its configuration.
- If the browser/CDP login worked but the daemon is stopped, Pi may still list a static provider but generation will fail. Start the daemon.

---

## 9. Optional: start the bridge automatically with user systemd

Use this only **after** a successful foreground test. Stop the foreground daemon with `Ctrl+C` before starting the service; both modes must not hold the same data-directory lock at once.

```bash
mkdir -p "$HOME/.config/systemd/user"
cat > "$HOME/.config/systemd/user/gemini-stealth-bridge.service" <<'UNIT'
[Unit]
Description=Local Gemini Web bridge for Pi Coding Agent
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=%h/.local/opt/gemini-stealth-bridge/start-gsb
Restart=on-failure
RestartSec=5
NoNewPrivileges=true
UMask=0077

[Install]
WantedBy=default.target
UNIT

systemctl --user daemon-reload
systemctl --user enable --now gemini-stealth-bridge.service
systemctl --user --no-pager --full status gemini-stealth-bridge.service
```

Common operations:

```bash
systemctl --user stop gemini-stealth-bridge.service
systemctl --user start gemini-stealth-bridge.service
systemctl --user restart gemini-stealth-bridge.service
journalctl --user -u gemini-stealth-bridge.service --no-pager -n 50
```

The service starts during your user session. Do **not** enable lingering or expose the service to the network unless you explicitly need that behavior and understand the implications. Logs can still include operational metadata; inspect before sharing them.

---

## 10. Efficient Pi usage and security guidance

### Token/usage behavior

| Item | Effect |
|---|---|
| Static `models.json` provider entry | Local configuration; **does not** add a new Pi tool schema or skill prompt to every turn |
| Bridge daemon when idle | Local memory/CPU/background session maintenance; no GPT-6 tokens merely because it is running |
| Gemini selected as Pi's active model | Prompts go to Gemini Web through `gsb`, not through your GPT model |
| DeepSeek/GPT selected instead | Gemini bridge is not called just because it is installed |
| Pi tools / `AGENTS.md` / large conversation history when Gemini is active | They **do** increase the context Gemini receives; the bridge's `BRIDGE_TOOL_SCHEMA_COMPACT=1` helps trim tool definitions |
| Usage reported by the bridge | Estimated; not comparable to official Google Developer API tokenizer/billing metrics |

Use Gemini for interactive discussions, research, code review, or bounded coding trials. For extensive repo mutation, use a model with reliable native tool calling, and keep your Pi permission/sandbox controls enforced. A permissive agent tool will still be dangerous regardless of which model requests it.

### Security checklist

- `BRIDGE_HOST=127.0.0.1` and `BRIDGE_PORT=20133`; **never** bind publicly by accident.
- Keep `BRIDGE_API_KEY` non-`public`; the launcher loads a random key from a `0600` file.
- Protect `$BRIDGE_HOME/accounts/*.json`, `session-cache.json`, and `pool.key` from Git, cloud sync, screenshots, and shared backups.
- The configuration at `~/.pi/agent/models.json` contains a **credential file path**, not the actual key.
- Account-key aliases such as `gsb-acc1` are predictable: do not use them as a public-access security boundary.
- Avoid enabling `BRIDGE_ALLOW_PRIVATE_IMAGE_URLS`; keep the default private-address SSRF protection.
- Leave `BRIDGE_DUMP_REQUEST` off; diagnostic captures can reveal sensitive metadata.
- Do not assume `pi-sandbox` or other host restrictions apply automatically to a *separately running* bridge daemon. The Pi model provider must be able to contact localhost, while Pi's local file/shell tools should retain their existing sandbox boundaries.
- Do not forward sensitive repository content to Gemini Web unless your information-sharing policy allows it.
- Gemini Web remains subject to Google quotas and account controls; a bridge does **not** grant unlimited use.

### Optional Web dashboard

With the launcher or service running, open locally:

<http://127.0.0.1:20133/ui>

It shows account/quota information and operational controls. Keep it loopback-only. It is not required for Pi's `/model` integration.

---

## 11. Troubleshooting

| Symptom | Recommended checks |
|---|---|
| `gsb --login` cannot open a browser | Run in a graphical user session; install a supported Chrome/Chromium-derived browser; check display/Wayland availability |
| `--probe` fails | Check saved account authorization and Google's sign-in challenges; run `--login acc1` again |
| `/health` fails | Start the foreground launcher or user systemd service; check port 20133 and daemon logs |
| `401 authentication_error` at `/v1/models` | Ensure caller uses the pool key from the same `$BRIDGE_HOME/pool.key` that the launcher loaded; restart daemon after rotating a key |
| `503 auth_unavailable` | Check `$BRIDGE_HOME/accounts/acc1.json` exists, and the daemon uses the same `BRIDGE_HOME` |
| `401 upstream_auth_error` / code 1016 | Reauthenticate with `--login acc1`; cookies may have expired |
| `403 account_banned` / `TOS_VIOLATION` | Stop automated retries; inspect Google's account restrictions and terms |
| `404 model_not_found` | Refresh the live list and rerun Step 7 to update provider model IDs |
| `429` quota exhausted | Inspect quota; wait for reset rather than increasing retry pressure |
| `429 model_substitution` | The bridge detected another model being used; check selected model/quota rather than disabling downgrade rejection blindly |
| Tool request appears as text | Prompt-simulated function call was malformed; do not trust it to edit files |
| Reasoning or streaming looks wrong | Compare a direct `/v1/chat/completions` response against Pi; test one model at a time |
| Pi cannot reach `127.0.0.1` | Check whether Pi is in a container or a restricted network namespace/sandbox; do not weaken filesystem restrictions to fix a networking issue |
| Port `20133` already in use | `ss -ltnp | grep :20133`; stop conflicting process instead of spawning duplicate daemon |
| Bridge complains about `.lock` | Verify only one instance uses this `BRIDGE_HOME`; don't remove a live lock or session state blindly |

After a model rename or quota change, the JSON provider list is **not self-updating**. Repeat **Step 7** to refresh the discovered model entries after stopping the existing Pi session or returning to `/model`.

---

## 12. Updating without losing credentials

1. Keep `$BRIDGE_HOME` intact: it contains Google session credentials and continuation state.
2. Check <https://github.com/azrinaldi/gemini-stealth-bridge/releases> for a **published** release and its changelog. Do not assume an unreleased README section is in your binary.
3. Stop the user service or foreground daemon.
4. Repeat Step 2 with `RELEASE='vX.Y.Z'` changed to the verified published version. **Verify its checksum.** The installation block saves a timestamped copy of the old executable.
5. Keep the same `start-gsb` launcher, key file, and account data.
6. Start the daemon; run `--version`, `/health`, `--probe`, the generation test, and the Pi acceptance check again.
7. If the list of Gemini models changed, rerun Step 7; it only refreshes your `gemini-web` provider.
8. If the new binary fails, stop the daemon and restore the older verified executable from the timestamped backup, then retest.

No extra Pi extension or prompt bundle needs to be installed for updates. Do not automatically track unreviewed GitHub `main` builds.

---

## 13. Removing or rolling back the integration

### 13.1 Remove only the Gemini provider from Pi

This preserves your other providers:

```bash
python3 - <<'PY'
from pathlib import Path
import json, os, tempfile
p = Path.home() / '.pi/agent/models.json'
raw = json.loads(p.read_text(encoding='utf-8'))
providers = raw.get('providers', {})
if 'gemini-web' not in providers:
    print('No Gemini provider found; nothing to remove')
else:
    del providers['gemini-web']
    fd, tmp = tempfile.mkstemp(prefix='.models.rollback.', suffix='.json', dir=p.parent)
    try:
        with os.fdopen(fd, 'w', encoding='utf-8') as f:
            json.dump(raw, f, indent=2, ensure_ascii=False)
            f.write('\n')
        os.chmod(tmp, p.stat().st_mode & 0o777)
        os.replace(tmp, p)
    except Exception:
        Path(tmp).unlink(missing_ok=True)
        raise
    print('Removed only gemini-web')
PY
```

Alternatively, restore the matching private pre-change backup from `~/.local/state/pi-provider-backups/` **only if you are sure that doing so will not discard later edits to other providers**.

### 13.2 Stop the daemon and optionally remove it

```bash
systemctl --user disable --now gemini-stealth-bridge.service 2>/dev/null || true
rm -f "$HOME/.config/systemd/user/gemini-stealth-bridge.service"
systemctl --user daemon-reload
# Optional: remove just the binary/launcher directory after confirming it is stopped:
# rm -rf "$HOME/.local/opt/gemini-stealth-bridge"
```

**Credential retention:** `$HOME/.local/share/gemini-stealth-bridge` holds Google session material and the local pool key. Keep it if you plan to reconnect later; if you intentionally decommission the bridge, stop it first and securely remove that directory through your preferred data-removal process. Ordinary file deletion does not guarantee secure erasure on SSDs or synced backups.

---

## 14. Acceptance checklist

Check each item on your machine:

- [ ] The downloaded binary's matching SHA-256 release checksum verified successfully.
- [ ] `gsb --version` reports the expected published release.
- [ ] `BRIDGE_HOME` is consistent for login, probe, launcher, and daemon.
- [ ] `--probe` succeeds using the Gemini account (no auth error).
- [ ] The daemon listens **only** on `127.0.0.1:20133`.
- [ ] `/health` returns service health, and `/v1/models` lists live account-accessible IDs.
- [ ] A direct Chat Completions request produces a nonempty Gemini answer.
- [ ] `models.json` remains valid JSON and still contains all existing GPT/DeepSeek providers.
- [ ] Pi lists `gemini-web` models and `/model` can select one.
- [ ] A basic Pi chat turn succeeds and a separately tested read-only tool call behaves correctly.
- [ ] No Google cookies or local pool key appear in Git-tracked files, shell history, or screenshots.
- [ ] If enabled, the systemd user service starts/stops reliably, with no second conflicting daemon.
- [ ] Switching back to your previous Pi model works.

If all checks pass, Gemini Web is integrated as **an optional, selectable Pi model provider**. The outcome is still experimental; it does **not** certify the reliability of autonomous coding-agent tool calls.

---

## 15. Primary references

1. [`azrinaldi/gemini-stealth-bridge` repository and README](https://github.com/azrinaldi/gemini-stealth-bridge) — binary releases, login, service, API, security and compatibility.
2. [Published GitHub releases](https://github.com/azrinaldi/gemini-stealth-bridge/releases) — verify the current stable published version and `SHA256SUMS` before updating.
3. [Repository `AGENTS.md`](https://github.com/azrinaldi/gemini-stealth-bridge/blob/main/AGENTS.md) — operational security, keys, account statuses, and coding-agent integration.
4. [Pi custom model configuration reference](https://fossies.org/linux/pi/packages/coding-agent/docs/models-configuration.md) — `models.json`, provider fields, `apiKey`, models, compatibility and thinking controls.
5. [Pi provider documentation](https://github.com/badlogic/pi-mono) — canonical Pi project; confirm your installed Pi version's custom provider schema if it differs.

*Prepared for an existing `~/.pi/agent` configuration, with a separate DeepSeek provider retained. No live browser credentials, Google login, or sandbox acceptance tests were performed during authoring.*
