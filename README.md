# fsq-solver

Solvers for Formula Student registration-quiz calculations. Built from the ~430 calculation questions
in the FS-Quiz bank (mirror in `../IFS-Tests/data/fsquiz`), grouped into archetypes in [docs/archetypes](docs/archetypes).

Being turned into **ISC MingoSolve**, a desktop app (macOS, Windows, Linux) in the MingoCAN family. The engine is
now Rust (`fsq` crate at the root) and reproduces every solved past question; the Python engine in
`legacy/python` stays as the quiz-day fallback until after the January 2027 quizzes.

```bash
cargo run --release            # interactive: load once, answer fast
cargo run --release -- ex skidpad
cargo test                     # past questions, golden corpus, units vs pint, rules 2027

cd legacy/python && uv run fsq # fallback: same commands, same data
```

Formulas, rules, units and the solved past questions are data in [data/](data), shared by both engines.

## Desktop app (apps/mingosolve)

Tauri 2 + Svelte 5, same structure and design system as MingoCAN's can-studio. The Rust commands call the `fsq`
crate directly.

```bash
cd apps/mingosolve
npm ci
npx tauri icon src-tauri/icons/icon.png   # once: platform icons are generated, not committed
npm run tauri:dev                         # run
npm run check                             # svelte-check (strict TS)
npx tauri build --bundles app             # local macOS bundle; CI builds all platforms (phase 3)
```

Views: **Solve** (paste the question → ranked formulas/tools/past questions → pre-filled form, live solve, option
matching, copy in quiz format, session log), **Chain**, **Tools**, **Past questions**, **Rules**, **Settings**;
calculator docked on the right; Cmd/Ctrl+K jumps to the formula search. The updater is wired to
`isc-fs/iskapps/mingosolve/latest.json` but update artifacts stay off until the signing key exists (phase 3).

## Commands

| Command | Does |
|---|---|
| `find <words>` | search formulas (`find spring`, `find discharge`) |
| `show <formula>` | equations, variables, units, gotchas |
| `<formula> k=v ...` | solve for every unknown the givens determine: `speed s=75m t=3.5s @v_avg=km/h` |
| `chain <target> k=v ...` | chain formulas automatically: `chain v h_cg=0.205m R_c=14.5m t_tr=1.24m` |
| `calc <expr> -> unit` | unit-aware calculator: `calc 0.5*300kg*(100km/h)**2 -> kJ` |
| `tools [words]` | procedural tools: scoring (legacy and 2026), rule tables, nodal circuit solver, E-series, CAN/UART... |
| `ex <words>` | past questions solved with fsq, with the exact command |
| `vars <words>` | variable names and units |

Values take units (`100km/h`, `8000rpm`, `1.5g0`, `20Ah`, `60degC`, `2bar`, decimal commas ok). Bare numbers
are SI. `@var=unit` picks the display unit. Defaults (g = 9.81, rho_air = 1.225, ...) are printed when used.

## How much to trust it

- Every row in [data/examples.toml](data/examples.toml) is an official answer the tool reproduces (pytest).
- `chain` prints any formula the final values violate (`! inconsistent`): that means two formulas used a
  variable with different meanings or the inputs contradict each other. Don't submit before reading it.
- Rules are year-keyed: `rules=2027` (default, FS-Rules 2027 v1.0), `2026`, and `legacy` (reproduces most
  old quiz keys). What changed for quiz answers: [docs/rules-2027-changes.md](docs/rules-2027-changes.md).
- Known quiz keys that don't match the physics or the rules (pick by option elimination):
  Q167, Q169, Q202, Q584, Q911, Q125, Q596, Q727, Q602, Q115, Q29, Q451, Q408, Q410, Q114, Q863 parts 2-3, Q1008.

Data: FS-Quiz (fs-quiz.eu), ODbL. Don't publish question text or derived databases without the ODbL attribution.
