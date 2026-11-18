# ADR 0001 — Own the Rust TUI and official Pi RPC adapter

- **Decision:** Independent Ratatui/Crossterm Rust workspace; unmodified Pi subprocess via its public JSONL RPC contract.
- **No runtime dependency:** Pitago, pi-genui, pi-openui.
- **Reason:** Full control of component geometry, local graphics, dependency upgrades, non-model UI rendering and future browser isolation, without patching Pi or coupling to a small upstream application's development workflow.
- **Trade-offs:** The team must implement and maintain RPC compatibility, editor/completions, theme adapter, interactive widget semantics and extension UI dialogs itself.
- **Security:** No frontend shell bypass; browser execution opt-in and independently sandboxed; no credentials in frontend.
- **Status:** Accepted architecture; implementation of the backend transport NOT STARTED.
