---
name: visual-recap
description: Explicit-command access to pi-feature-dev's visual-recap skill.
disable-model-invocation: true
---

# visual-recap

The user explicitly invoked this skill. Resolve the Pi package directory as `$PI_PACKAGE_DIR` if set; otherwise use `$PI_CODING_AGENT_DIR/npm` if that variable is set, or `~/.pi/agent/npm`. Expand the selected directory to a real path, then read `node_modules/pi-feature-dev/skills/visual-recap/SKILL.md` under it and follow the instructions for the user's request. Resolve relative references against that installed skill directory. If that file is missing, report the missing package resource and stop.

Global security, approval, and no-agent-Git rules still apply. Do not load another explicit-command-only skill unless the user invokes its `/skill:name` command.
