# MingoSolve cheat sheet

Print this (two pages). The same content is in the app under **Help** (<kbd>⌘/</kbd> on Mac, <kbd>Ctrl+/</kbd> elsewhere).

## 1. Quick start

1. **Paste** the question anywhere in the window.
2. **Open** the first match under the problem. Its fields arrive filled in.
3. **Check** the values it took, then **copy** the answer (<kbd>⌘↵</kbd> / <kbd>Ctrl+Enter</kbd>) and paste it into the quiz.

A blank field is an unknown: the script solves for whatever you leave empty.

New here? **Help > Take the tour** (or Settings) replays a short guided tour of the real screen; **Esc** skips it.

## 2. Finding scripts

- **Paste a problem.** Each match shows how many values it filled in; the best one is first.
- **Library / Topics.** Type a word (`spring`) or a variable (`k_s`).
- **<kbd>⌘K</kbd> / <kbd>Ctrl+K</kbd>** from anywhere: `discharge`, `skidpad score`. A whole sentence is searched as a problem.
- **Worked examples** at the bottom of a script load a past question's inputs in one click.

## 3. Reading the answer

| You see | It means | What to do |
|---|---|---|
| Several roots (`Other roots: ...`) | A quadratic has two answers | Pick the one the physics allows; a negative time or current is not an answer |
| `Assumed g = 9.81.` | A constant you did not give was filled with its default | If the problem says another value, type it in |
| `v_i → v0 (initial velocity)` under the known values in Chain | Common names (`v_i`, `v_f`, `μ`, `wheelbase`...) are understood and shown as the real name | Check it is the variable you meant; for an unknown or ambiguous name, click the suggested one |
| **from the problem** tag | The value was read from the pasted text (hover to see which words) | If it is not what the problem says, fix it |
| **not used** chip under the problem | The script ignored a value found in the text | You may have the wrong script, or the question has a detail you must add |
| `rules 2027` pill, `Other rules: ...` | Scoring uses the rule year in Settings; the line shows 2026 and legacy | Old questions often keep an old key: if the options match another year, use it |
| **These values contradict each other** | The given values cannot all be true | Do not copy. Re-read for a wrong number or unit |

## 4. Checking options

Open **Check against options** and paste the choices, one per line (`a) 73.4 A`). The closest is marked with its distance in percent.

- Within 2 % counts as a match; units are converted (`0.5 kW` = `500 W`).
- **Careful** = two options are close, or one was skipped (other unit). Re-check.
- **Rule year** = another year's rules fit an option much better.

## 5. Known-wrong keys

A few official answers disagree with the physics or the rules. If the answer matches **no** option, choose by elimination. The list (question number and what the key did) is [`data/known_keys.toml`](../data/known_keys.toml). Pasting one of these past questions shows a red note under the problem.

## 6. Shortcuts

| Mac | Windows / Linux | Does |
|---|---|---|
| ⌘K | Ctrl+K | Find a script (again to close) |
| ⌘/ | Ctrl+/ | Open Help (again to go back) |
| ⌘V | Ctrl+V | Paste a problem (outside a text field) |
| ⌘↵ | Ctrl+Enter | Copy the answer, also from inside a field |
| ↵ | Enter | Run a tool script / evaluate the calculator |
| ↑ ↓, ↵, Esc | ↑ ↓, Enter, Esc | In the palette: move, open, close |
| ↑ ↓ Home End | ↑ ↓ Home End | In the library: move through topics |
| → or ↵ / ← | → or Enter / ← | Into the scripts of a topic / back |
| any letter | any letter | In the library: start a search (Esc clears, ↵ opens the first) |

**Undo clear** is a button: after **Clear** it stays at the top of the sheet for 8 seconds.

## 7. Rehearsing: the session log

Every copied answer is added to **Session log** (left rail). Type a label such as `Q7` in **Question #** at the top of the sheet. **Start a mock quiz** resets the timer and tags the entries that follow. Keep the last 500 entries on your computer.

**Export CSV** writes a UTF-8 file with a byte order mark. Choose the **semicolon** separator if your spreadsheet uses a decimal comma (Spanish Excel). If the download does not start, **Copy CSV** and paste into a sheet.

| Column | Content |
|---|---|
| `timestamp` | When it was copied, ISO 8601, UTC |
| `mock_quiz` | Mock quiz id, empty outside one |
| `question` | Your **Question #** label |
| `script_id`, `script_title` | The script used |
| `inputs` | `name=value; name=value`, as typed, units included |
| `answer` | The text that was copied |
| `rule_year` | For scoring scripts, the rules used |
| `elapsed_s` | Seconds since the problem was pasted (empty if no clock was running) |
| `method` | `button`, `shortcut` or `copy-all` |

Compare two teammates by sorting both files on `question`.

## 8. If the app fails

Do not troubleshoot during the quiz. Both fallbacks use the same data and give the same answers. From the repository folder:

```bash
# command line (needs Rust)
cargo run --release
cargo run --release -- speed s=75m t=3.5s @v_avg=km/h

# Python (needs uv)
cd legacy/python && uv sync && uv run fsq
```

Installing and updating: [INSTALL.md](INSTALL.md). Report problems afterwards at <https://github.com/isc-fs/MingoSolve/issues> with your system and what you saw.
