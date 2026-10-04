<img width="470.235" height="179.4" alt="isc-full-primary" src="https://github.com/user-attachments/assets/31365569-11bf-427e-ae3e-8d81ca87d765" />

# ISC MingoSolve

A toolbox of engineering **scripts** for Formula Student problems, built for the registration quizzes. A formula
script solves for whatever you leave blank, with units; a tool script runs a procedure (event scoring, nodal
circuits, CAN timing...). Desktop app for macOS, Windows and Linux, plus a command-line interface.

108 formulas and 34 tools, filed under 12 topics. 145 past FS-Quiz questions are reproduced to their official answer
and run as tests. All of it is data in [data/](data).

---

## The app (apps/mingosolve)

- **Paste a problem** anywhere in the window. The scripts that fit appear with the problem's values already filled
  in, and the one the question asks for is the answer.
- Or browse **Topics**, or press **⌘K / Ctrl K** and type (`spring rate`, `discharge`, `skidpad score`).
- Every root is listed. The answer copies in quiz format (significant figures or decimals, decimal comma), and can be
  checked against pasted multiple-choice options. Each script lists the past questions it solves, one click each.
- **Chain** links several formulas to one target. The **calculator** is unit-aware (`0.5*280kg*(100km/h)**2 -> kJ`).
- Look: **Night glass** (dark) and **Paper glass** (light), following the OS, on the ISC design system. The window is
  translucent on macOS and Windows 11; solid surfaces are one setting away.

```bash
cd apps/mingosolve
npm ci
npx tauri icon src-tauri/icons/icon.png   # once: platform icons are generated, not committed
npm run tauri:dev
```

Installers will be published through [isc-fs/iskapps](https://github.com/isc-fs/iskapps) once the release pipeline
lands (see the [roadmap](ROADMAP.md)).

## The command line

```bash
cargo run --release                                   # interactive: load once, answer fast
cargo run --release -- speed s=75m t=3.5s @v_avg=km/h
cargo run --release -- chain v h_cg=0.205m R_c=14.5m t_tr=1.24m
cargo run --release -- ex skidpad                     # past questions with the exact command
```

| Command | Does |
|---|---|
| `find <words>` | search formulas (`find spring`, `find discharge`) |
| `show <formula>` | equations, variables, units, gotchas |
| `<formula> k=v ...` | solve for every unknown the givens determine |
| `chain <target> k=v ...` | chain formulas automatically to a target |
| `calc <expr> -> unit` | unit-aware calculator: `calc 0.5*300kg*(100km/h)**2 -> kJ` |
| `tools [words]` | procedural tools: scoring (legacy, 2026, 2027), rule tables, nodal circuits, E-series, CAN/UART... |
| `ex <words>` | past questions solved with fsq |
| `vars <words>` | variable names and units |

Values take units (`100km/h`, `8000rpm`, `1.5g0`, `20Ah`, `60degC`, `2bar`; decimal commas are fine). Bare numbers
are SI. `@var=unit` picks the display unit. Defaults (g = 9.81, rho_air = 1.225...) are printed when used.

`legacy/python` is the original sympy engine reading the same data (`cd legacy/python && uv run fsq`). It is the
quiz-day fallback and the parity oracle until after the January 2027 quizzes, then it goes.

## How much to trust it

- Every row in [data/examples.toml](data/examples.toml) is an official answer that both the engine and the app
  reproduce.
- Rules are year-keyed: `rules=2027` (default, FS-Rules 2027 v1.0), `2026`, and `legacy` (reproduces most old quiz
  keys). What changed for quiz answers: [docs/rules-2027-changes.md](docs/rules-2027-changes.md).
- When values contradict each other, the app and `chain` say so instead of answering. Read that before submitting.
- Some quiz keys don't match the physics or the rules. They are listed, with what the key did, in
  [data/known_keys.toml](data/known_keys.toml); pick by option elimination there.

## Tests

| Layer | What it proves | Run |
|---|---|---|
| Engine | past questions, Python parity per formula and unknown, every formula dimensionally homogeneous, property tests, units vs pint, 2027 rules | `cargo test` |
| App commands | the Tauri IPC contract, every worked example through the commands | `cargo test -p mingosolve` |
| Frontend | the sheet, finder, settings and session logic against a fake engine, incl. out-of-order replies | `cd apps/mingosolve && npm test` |
| End to end | real UI on the real engine in Chromium and WebKit: quiz journeys, every worked example clicked, accessibility (axe) | `cd apps/mingosolve && npm run e2e` |
| Fallback | the Python engine on the same data | `cd legacy/python && uv run pytest` |

CI runs all of it on every pull request.

---

## How we work with this repository

### Branches

**`main`** holds released versions only. **`dev`** is where work comes together. Never commit to either directly:
all changes arrive through a feature branch and a Pull Request to `dev`.

```
main  ──────────────────●──────────────────────●──▶  (releases only)
                        ↑                      ↑
dev   ──────●───●───●───●───●───●───●───●───●──●──▶  (continuous integration)
            ↑   ↑       ↑   ↑   ↑       ↑   ↑
          feat/1 fix/1 feat/2 fix/2   feat/3 fix/3
```

Branches are `feat/<n>-short-title` (new functionality) or `fix/<n>-short-title` (bug fix), each type with its own
counter. Every branch gets a tracking issue automatically when it is pushed (`[feat/3-...]`, labelled `feat` or `fix`),
filled with the first commit message; it closes when the branch merges into `dev`. The next number of a type is the
last issue of that type plus one; the issue warns if the number is wrong.

### Step by step

```bash
git checkout dev && git pull origin dev
git checkout -b feat/5-short-title     # next number for its type
git push origin feat/5-short-title     # the tracking issue opens within seconds
# work, commit with clear imperative messages, push
```

Open a Pull Request to `dev` with `Closes #<issue>` in the description. Before asking for review, the tests above
pass locally and the PR targets `dev`, not `main`. When `dev` holds a set of changes ready to ship, a release PR
goes from `dev` into `main`.

The [roadmap](ROADMAP.md) is generated from [.github/roadmap.yaml](.github/roadmap.yaml) and the tracking issues.
Instructions for coding agents are in [AGENTS.md](AGENTS.md).

---

## Data and licence

The questions, answers and archetype analysis come from the [FS-Quiz](https://fs-quiz.eu) question bank, available
under the [Open Database License (ODbL) 1.0](https://opendatacommons.org/licenses/odbl/1-0/). Everything in this
repository derived from it is published under the ODbL too, with this attribution; see [DATA_LICENSE.md](DATA_LICENSE.md).
Rule constants follow the published [FS-Rules](https://www.formulastudent.de/fs/rules/).

---

*ISC Racing Team*
