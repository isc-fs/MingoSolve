<!--
  Fill in every section. PRs that omit "Closes #N" will not auto-close
  their tracking issue when merged into dev.
-->

## Summary

<!-- 1–3 bullets on what changed and why. -->

-

## Linked issue

Closes #

## Type of change

- [ ] New functionality (`feat/*`)
- [ ] Bug fix (`fix/*`)
- [ ] New formulas, tools or past questions (data)
- [ ] Refactor / no behaviour change
- [ ] Documentation only
- [ ] Build / tooling / CI

## Testing

<!-- What did you run? Tick all that apply. -->

- [ ] `cargo test` (engine, past questions, Python parity, dimensions)
- [ ] `cargo test -p mingosolve` (app commands)
- [ ] `npm test` and `npm run e2e` in `apps/mingosolve`
- [ ] `uv run pytest` in `legacy/python` (if data or the fallback changed)
- [ ] Tried in the app (`npm run tauri:dev`): what you checked
- [ ] Not testable in this PR, because:

## Checklist

- [ ] Branch is `feat/<n>-...` or `fix/<n>-...` with the next number for its type
- [ ] PR targets `dev` (never `main` directly)
- [ ] `cargo fmt`, `cargo clippy -- -D warnings` and `npm run check` are clean
- [ ] Every new formula or tool has a past question with its official answer, or a test against an independent reference
- [ ] Nothing copied from FS-Quiz question text; derived data keeps the ODbL attribution
