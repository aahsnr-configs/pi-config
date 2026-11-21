# Install the project-only Codex Pi-maintenance workflow

This package contains **proposed files**. It has not installed or modified anything in your real home directory. Back up existing files and review differences first. The main task prompt does **not** contain Codex launch/setup commands; those live here.

## Intended destination map

| File in this ZIP | Destination on your machine |
|---|---|
| `pi/AGENTS.md` | `~/.pi/AGENTS.md` — explicitly read as **Codex** instructions |
| `pi/pi-codex-reorganization-audit-token-efficient.md` | `~/.pi/pi-codex-reorganization-audit-token-efficient.md` |
| `pi/agent/.codex/config.toml` | `~/.pi/agent/.codex/config.toml` — **merge**, don't overwrite |
| `pi/agent/.codex/agents/pi_astra_advisor.toml` | `~/.pi/agent/.codex/agents/pi_astra_advisor.toml` |
| `pi/agent/.codex/advisor-workflow.md` | `~/.pi/agent/.codex/advisor-workflow.md` |
| `pi/agent/.codex/tests/ADVISOR_DISCOVERY.md` | `~/.pi/agent/.codex/tests/ADVISOR_DISCOVERY.md` |
| `sources.md` | Research evidence supplied separately; keep wherever you prefer |

**Never place the Pi-specific advisor in `~/.codex/agents/`**, and do not touch `~/.pi/agent/AGENTS.md` during installation. The latter is Pi's own file and is a distinct artifact from `~/.pi/AGENTS.md`.

## Deployment (you run this; no live files were changed here)

Extract the ZIP to a convenient private temporary directory. The following commands assume you have changed into the extracted bundle root, containing `pi/`:

```bash
set -euo pipefail
BACKUP="$HOME/.pi/codex-setup-backup-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$BACKUP" "$HOME/.pi/agent/.codex/agents" "$HOME/.pi/agent/.codex/tests"

for rel in \
  "AGENTS.md" \
  "pi-codex-reorganization-audit-token-efficient.md" \
  "agent/.codex/config.toml" \
  "agent/.codex/agents/pi_astra_advisor.toml" \
  "agent/.codex/advisor-workflow.md" \
  "agent/.codex/tests/ADVISOR_DISCOVERY.md"
do
  if [ -f "$HOME/.pi/$rel" ]; then
    mkdir -p "$BACKUP/$(dirname "$rel")"
    cp -a "$HOME/.pi/$rel" "$BACKUP/$rel"
  fi
done
printf 'Existing-file backups (where applicable): %s\n' "$BACKUP"

# Review changed instructions BEFORE replacing them:
diff -u "$HOME/.pi/AGENTS.md" pi/AGENTS.md || true
# If satisfied, copy only the files you have reviewed:
cp -a pi/AGENTS.md "$HOME/.pi/AGENTS.md"
cp -a pi/pi-codex-reorganization-audit-token-efficient.md "$HOME/.pi/"
cp -a pi/agent/.codex/agents/pi_astra_advisor.toml "$HOME/.pi/agent/.codex/agents/"
cp -a pi/agent/.codex/advisor-workflow.md "$HOME/.pi/agent/.codex/"
cp -a pi/agent/.codex/tests/ADVISOR_DISCOVERY.md "$HOME/.pi/agent/.codex/tests/"
```

**Project `config.toml` needs special care:** If `$HOME/.pi/agent/.codex/config.toml` already exists, merge these two settings into it without discarding any other keys:

```toml
model = "gpt-6.1-sol"

[agents]
enabled = true
```

If it does **not** exist, you can use:

```bash
cp -a pi/agent/.codex/config.toml "$HOME/.pi/agent/.codex/config.toml"
```

Do not create or edit `~/.codex/config.toml` for this workflow. Codex may protect its project `.codex/` directory from normal workspace writes; create these project integration files using your terminal before starting Codex, not by asking Codex to override a protection.

## Correct Codex launch command

```bash
codex -C "$HOME/.pi/agent" -m gpt-6.1-sol -c project_doc_max_bytes=0
```

This is an **appropriate** launch for the requested instruction separation: `-C` sets the workspace directory, `-m` selects Sol 6.1 for that session, and `-c project_doc_max_bytes=0` prevents automatic **project `AGENTS.md` instruction injection**. It does **not** prevent Codex from listing/reading ordinary files, including `~/.pi/agent/AGENTS.md`, subject to filesystem/sandbox permissions. It also does not erase project `.codex/` configurations. Your global Codex guidance from `~/.codex/AGENTS.md` still applies. The project-specific `~/.pi/AGENTS.md` must be **explicitly read**, as instructed at the beginning of the task prompt. Avoid adding the project instruction limit to your global Codex config.

If the directory is not a Git project, Codex may require a project trust decision or start with read-only permissions. Review and approve only the intended project-local `.codex/` files. Check `/status` and `/debug-config` (if available in your installed CLI) rather than assuming those layers loaded.

When ready, give Codex the content of `~/.pi/pi-codex-reorganization-audit-token-efficient.md`. Do **not** copy the launch command into the implementation prompt; its job is to describe the task, not configure your shell.

## Advisor discovery acceptance

Before running the large reorganization task, follow `~/.pi/agent/.codex/tests/ADVISOR_DISCOVERY.md`. That document checks *discovery of the project-only registered Astra advisor*, **not** Astra activation or an actual model-selection test. A TOML file being present is necessary but not sufficient proof that Codex loaded the custom agent. There is no verified universal non-spawning `codex agents list` command that enumerates custom-role registrations across all versions; the test is explicit about evidence requirements and possible `NOT VERIFIED` outcomes.

If Codex cannot select the custom `pi_astra_advisor` role on your installed version, do not silently spawn a generic fallback agent. Fix project discovery/configuration or report the limitation. When Sol later becomes **stuck on the task**, its Codex `AGENTS.md` instructions require consulting the correctly registered Astra advisor.

## Boundaries

- Neither this installation guide nor the task prompt asks you to run `bwrap`.
- The task-specific user-run full Pi acceptance tests are separate and will be generated under `~/.config/pi/Tests/` when Codex implements the main task.
- Pi can also read ancestor `AGENTS.md` files when launched **inside `~/.pi/`**. Avoid launching Pi itself with `$HOME/.pi` or `$HOME/.pi/agent` as its working project directory if you do not want Pi to see the Codex-specific `$HOME/.pi/AGENTS.md` as contextual material.
