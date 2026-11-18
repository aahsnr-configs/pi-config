# Visualization parity register — P04 foundations only

**2026-11-16:** P04 adds tested static chat Markdown, code-fence presentation and workflow cards. These are partial foundations for families03/11/12, not full interactive parity. All23 full-family acceptance results remain missing; no real Kitty proof exists.

This is the semantic parity target for the future Rust UI. None of these components should be marked complete based on the existence of this document or a static mockup.

| Family | Current status |
|---|---|
| Stack | missing |
| Text | missing |
| Card | partial: static event-derived chat cards, tested-ci; family interactions missing |
| Callout | missing |
| Metric | missing |
| Table | missing |
| Tabs | missing |
| Button | missing |
| Choice | missing |
| Faq | missing |
| Markdown | partial: local parser/stream wrapping, tested-ci; full family acceptance missing |
| CodeBlock | partial: fence labels/styling/wrap, tested-ci; lexer/copy/pan missing |
| Image | missing |
| BarChart | missing |
| LineChart | missing |
| AreaChart | missing |
| PieChart | missing |
| RadarChart | missing |
| GeoMap | missing |
| Dag | missing |
| Erd | missing |
| InfraMap | missing |
| SequenceDiagram | missing |

Additional mandatory later capabilities: reactive local expressions, forms, tab/tooltip navigation, pinned annotations, keyboard parity, numeric simulation integrators, pendulum and other physics demos, offline images, choice answer staging, safe inline HTML/JS Canvas, terminal focus and scroll-aware graphics.

Default UI-only interactions must have **zero incremental LLM calls and schema tokens**. Explicit new model-generated visualization authoring is outside that zero-incremental-UI contract.

P02 foundation is locally tested (G0 PASS); it adds no widget family or GUI acceptance. P03.1 revised G1 passed under its approved scope.

P03.1 revised G1 permission-interception scope passes. P04 automated/PTY frontend proof exists; real Kitty remains NOT RUN. Original sandbox-inclusive G1 evidence is archived.
