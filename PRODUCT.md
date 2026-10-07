# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

Desktop app (Tauri 2 + Svelte 5) for macOS, Windows and Linux; the UI is a web view, not a native design language.
There is also the `fsq` command-line interface, which has no visual design surface.

## Users

The ISC Racing Team members who sit the Formula Student registration quizzes, plus the maintainers who add formulas
and tools. Other teams may find the public repo, but they are not an audience to design for.

On quiz day the questions come one at a time with very little time each. Several members work in parallel, each on
their own laptop with the quiz open in a browser and MingoSolve beside it. The job is: read the problem, get the
number in the exact format the quiz wants, and not submit a wrong answer. Before the quizzes, the same people rehearse
with timed mock quizzes and the session log.

## Product Purpose

A toolbox of engineering scripts for FS registration-quiz problems. A formula script solves for whatever is left
blank, with units; a tool script runs a procedure (event scoring, nodal circuits, CAN timing...). Pasting a problem
surfaces the scripts that fit, pre-filled from the text, and the answer copies in quiz format.

Success means the team answers fast and correctly in the January 2027 quizzes: every past question reproduces its
official answer, and when the app cannot be trusted (contradictory values, known-wrong keys, another rule year) it says
so instead of answering.

## Positioning

Built from the FS-Quiz question bank itself: 145 past questions reproduce their official answers and run as tests,
rules are year-keyed (2027, 2026, legacy), and quiz quirks (rounding hints, decimal commas, keys that contradict the
physics) are modelled explicitly. It is a quiz instrument, not a general calculator or CAS.

## Operating Context

- Time pressure, one question at a time, app side by side with the quiz in a browser.
- Main path: paste the problem anywhere → open the best match (fields pre-filled, each tagged with where it came from)
  → check → copy the answer (⌘↵ / Ctrl+Enter). Alternatives: Topics library, ⌘K palette, Chain, unit-aware calculator,
  check against pasted multiple-choice options, rulebook search from a locally loaded PDF.
- Help (⌘/), printable cheat sheet (`docs/cheat-sheet.md`), first-launch guided tour, session log exported as CSV.
- Installed by non-programmer teammates from isc-fs/iskapps releases, unsigned on macOS, with in-app updates.
- The Python engine in `legacy/python` is the quiz-day fallback until after the January 2027 quizzes.

## Capabilities and Constraints

- 108 formulas and 34 tools in 12 topics, all defined as data in `data/`.
- Feature freeze 15 December 2026; only fixes after 1 January 2027. Pinned version for the quizzes (v1.x).
- Keyboard-first: every main action has a shortcut; the answer must always be visible.
- Repo is public: no secrets, personal paths or FS-Quiz question text in the code or UI fixtures.
- FS-Quiz-derived data is ODbL and keeps its attribution.
- Undecided / later: dropping the Python fallback, a full REPL, a contributor guide (v2.0.0, after the quizzes).

## Brand Commitments

- Name: ISC MingoSolve, from the ISC Racing Team.
- Built on the ISC design system tokens (RAL green #064229, gold #FFB81D, graphite; Jost display, IBM Plex Sans body,
  IBM Plex Mono data), with its own look recorded in `apps/mingosolve/src/app.css`. Not a copy of MingoCAN's look.
- Gold is never text on a light ground.
- Voice: plain, direct English instructions ("Do not copy. Re-read for a wrong number or unit"); no hype.

## Evidence on Hand

- `data/examples.toml`: past questions with official answers, reproduced by engine and app.
- `data/known_keys.toml`: official keys that contradict the physics or rules.
- `docs/cheat-sheet.md`, `docs/INSTALL.md`, `docs/rules-2027-changes.md`, `docs/design/directions.html`.
- No user testimonials or quiz-result data yet; mock-quiz findings are planned (`feat/13-mock-quiz-fixes`).

## Product Principles

1. Speed to a correct, copyable answer beats everything else on screen.
2. Never let a doubtful answer look certain: show provenance, assumptions, contradictions and other rule years.
3. Everything is data with a test behind it; the UI reflects what the engine can prove.
4. Usable cold by any teammate under pressure: obvious first step, shortcuts for the practised, help one key away.

## Accessibility & Inclusion

WCAG 2.1 AA, as already enforced: axe in the e2e suite, 4.5:1 text contrast on every surface, an opaque fallback for
every glass surface (`[data-solid]`, also on the OS "reduce transparency"), motion under 200 ms and dropped with
"reduce motion", usable at zoom.
