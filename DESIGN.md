---
name: ISC MingoSolve
description: Night glass and Paper glass, a pit-wall instrument for FS quiz answers.
colors:
  racing-green: "#064229"
  racing-green-600: "#0a5334"
  racing-green-500: "#0d6b43"
  pit-gold: "#ffb81d"
  pit-gold-400: "#ffc94f"
  pit-gold-600: "#e6a61a"
  night-ink: "#07130d"
  paddock-paper: "#f7f5f1"
  sand: "#dfdbd2"
  graphite: "#404040"
  on-gold: "#1a1406"
  night-glass-solid: "#11241a"
  night-text: "#f4f2ec"
  night-muted: "rgba(255, 255, 255, 0.75)"
  night-good: "#8cf0bf"
  night-warn: "#ffcf66"
  night-bad: "#ffb4ab"
  paper-text: "#1c211e"
  paper-muted: "#5d5d5d"
  paper-warn: "#7a5700"
  paper-bad: "#b3261e"
  hue-teal-night: "#6fd3d6"
  hue-teal-paper: "#0b6e74"
typography:
  display:
    fontFamily: "Jost, Futura, Century Gothic, sans-serif"
    fontSize: "1.625rem"
    fontWeight: 600
    letterSpacing: "0.01em"
  answer:
    fontFamily: "Jost, Futura, Century Gothic, sans-serif"
    fontSize: "2rem"
    fontWeight: 600
    letterSpacing: "0.01em"
  headline:
    fontFamily: "Jost, Futura, Century Gothic, sans-serif"
    fontSize: "1.25rem"
    fontWeight: 600
    letterSpacing: "0.01em"
  title:
    fontFamily: "Jost, Futura, Century Gothic, sans-serif"
    fontSize: "1rem"
    fontWeight: 600
    letterSpacing: "0.01em"
  body:
    fontFamily: "IBM Plex Sans, Segoe UI, system-ui, sans-serif"
    fontSize: "0.90625rem"
    fontWeight: 400
    lineHeight: 1.5
  body-sm:
    fontFamily: "IBM Plex Sans, Segoe UI, system-ui, sans-serif"
    fontSize: "0.8125rem"
    fontWeight: 400
    lineHeight: 1.5
  data:
    fontFamily: "IBM Plex Mono, ui-monospace, SF Mono, Menlo, monospace"
    fontSize: "0.8125rem"
    fontWeight: 500
  label:
    fontFamily: "IBM Plex Mono, ui-monospace, SF Mono, Menlo, monospace"
    fontSize: "0.6875rem"
    fontWeight: 400
    letterSpacing: "0.08em"
rounded:
  sm: "10px"
  md: "13px"
  lg: "18px"
  xl: "22px"
  pill: "999px"
spacing:
  "1": "4px"
  "2": "8px"
  "3": "12px"
  "4": "16px"
  "5": "20px"
  "6": "24px"
  "8": "32px"
components:
  button:
    backgroundColor: "rgba(255, 255, 255, 0.085)"
    textColor: "{colors.night-text}"
    typography: "{typography.body-sm}"
    rounded: "{rounded.pill}"
    padding: "0 16px"
    height: "34px"
  button-primary-night:
    backgroundColor: "{colors.pit-gold}"
    textColor: "{colors.on-gold}"
    rounded: "{rounded.pill}"
    padding: "0 16px"
    height: "34px"
  button-primary-paper:
    backgroundColor: "{colors.racing-green-500}"
    textColor: "#ffffff"
    rounded: "{rounded.pill}"
    padding: "0 16px"
    height: "34px"
  button-copy:
    backgroundColor: "{colors.pit-gold}"
    textColor: "{colors.on-gold}"
    rounded: "{rounded.pill}"
    padding: "0 16px"
    height: "34px"
  button-sm:
    typography: "{typography.data}"
    rounded: "{rounded.pill}"
    padding: "0 12px"
    height: "28px"
  input:
    backgroundColor: "rgba(0, 0, 0, 0.3)"
    textColor: "{colors.night-text}"
    typography: "{typography.body}"
    rounded: "{rounded.sm}"
    padding: "8px 12px"
    height: "38px"
  input-paper:
    backgroundColor: "#ffffff"
    textColor: "{colors.paper-text}"
    rounded: "{rounded.sm}"
    padding: "8px 12px"
    height: "38px"
  chip:
    backgroundColor: "rgba(255, 184, 29, 0.14)"
    textColor: "{colors.pit-gold-400}"
    typography: "{typography.data}"
    rounded: "{rounded.pill}"
    padding: "0 10px"
    height: "24px"
  sheet:
    backgroundColor: "rgba(255, 255, 255, 0.055)"
    rounded: "{rounded.xl}"
    padding: "20px"
  answer-slab-paper:
    backgroundColor: "{colors.racing-green}"
    textColor: "#ffffff"
    typography: "{typography.answer}"
    rounded: "{rounded.lg}"
    padding: "16px 20px"
  rail:
    backgroundColor: "rgba(255, 255, 255, 0.055)"
    width: "236px"
    padding: "12px"
  rail-item:
    textColor: "rgba(255, 255, 255, 0.82)"
    typography: "{typography.body}"
    rounded: "{rounded.md}"
    padding: "9px 12px"
---

# Design System: ISC MingoSolve

## Overview

**Creative North Star: "The Pit-Wall Instrument"**

MingoSolve is read the way a timing screen is read from the pit wall: in a glance, under pressure, with one number
that matters. Everything is frosted glass panes floating over a softly lit, grainy ground: Racing Green light pooling
from the top-left corner, a faint Pit Gold glow from the bottom-right. Panes are calm and low-contrast so that the one
loud surface, the answer slab, wins every time.

Controls are tactile and confident. Buttons are pills that press in, fields are clear-edged wells, and gold means
"act on this". There are two themes over one structure: **Night glass** (dark, the default when the OS is dark) and
**Paper glass** (light). On macOS and Windows 11 the window itself is translucent and the ground becomes a tint over
the desktop. Every pane has a solid fallback, because glass is the look but never a requirement.

Built on the ISC design system tokens (green, gold, graphite; Jost, IBM Plex Sans, IBM Plex Mono). The look is MingoSolve's
own and does not borrow MingoCAN's.

ISC is the default and the product's identity, and everything above describes it. Settings > Look also offers six
opt-in **styles**: Prontuario, Plano, Salidas, Datasheet, Grafito and Rosa, each a world the team reads daily, each
with a light and a dark mode. They are skins of the same shell, one instrument in seven bindings. Layout, components,
copy and behaviour never change per style. A style lends only type, palette, material and density, plus one signature
move, and in every style the answer slab and its Copy button stay the loudest thing on screen. The style
is set by `data-style` on `<html>`. ISC's tokens live in `app.css`, and each other style overrides them in
`src/styles/<id>.css`.

**Key Characteristics:**
- Frosted, rounded glass panes (18–22px corners) over a lit, grainy ground.
- One loud surface per screen: the gold-edged answer slab, which sticks to the bottom while the fields scroll.
- Jost for headings and the answer, Plex Sans for prose, Plex Mono for every value, unit, variable and label.
- Pill-shaped controls, gold (night) or green (paper) as the single action accent.
- Dense, desktop-first, keyboard-first; shortcuts shown inline as `kbd` hints.
- Seven switchable styles over one unchanged shell; ISC is the default, the others are opt-in skins.

## Colors

Racing Green and Pit Gold over near-black ink or warm paper. The accent is rare, and the two themes swap which brand
colour carries it.

### Primary
- **Racing Green** (racing-green, with -600 and -500 steps): the brand ground. In Paper glass it is the action
  accent (the 500 step) and the fill of the answer slab. In Night glass it is the glow pooling behind the glass.
- **Pit Gold** (pit-gold, -400, -600): the action and answer colour. In Night glass it is the accent, focus ring,
  variable names and answer label (the 400 step for text). In both themes it fills the Copy button, with On-Gold text.

### Secondary
- **Pit-Lane Teal** (hue-teal-night / hue-teal-paper): only a category dot in the palette, alongside green and gold. It
  never carries actions.

### Neutral
- **Night Ink** (night-ink): the Night glass ground. Panes are white at 5.5–8.5% opacity over it, and the solid
  fallback is Night Glass Solid (night-glass-solid).
- **Paddock Paper** (paddock-paper): the Paper glass ground. Panes are white at 60–78%, edged in green at 11%.
- **Sand** (sand): field borders in Paper glass.
- **Graphite** (graphite): secondary text in Paper glass. Night glass uses white at 82%.
- **Night Text / Paper Text** (night-text, paper-text): primary text. Muted text (Night Muted, white at 75%, or
  paper-muted) is only for metadata. Night Muted was raised from 62% so muted text still reads on a selected row.
- **Status** (night-good/warn/bad, paper-warn/bad; good on paper is Racing Green 500): text colour for strong labels
  inside notes, paired with the same hue at 8–12% as the note's background.

### Named Rules
**The Gold-on-Paper Rule.** Gold is a fill or a dark-theme accent, never text on a light ground. On paper, accent text
is Racing Green 500 (`--ink-accent`).

**The 4.5 Rule.** Every text token meets 4.5:1 on its own surface; `contrast.test.ts` enforces it. A new text colour
needs a row there.

**The One Readout Rule.** Pit Gold carries one meaning per screen: the answer and the act of copying it. Do not use it
for decoration, section headers or secondary buttons.

### Style palettes
Every style defines every colour token (ground, glass, text, accent, answer, status, category hues) for both modes.
The values are in `src/styles/<id>.css` and the sidecar; the character of each:
- **Prontuario** (Printed page / Bookcloth): a formula handbook. Black ink on white pages over a warm grey desk, and
  black bookcloth with cream type at night. Ink is the only accent, and vermilion is kept for errors, like an errata
  slip. The answer slab is the page's one reversed plate: black with white figures on the page, cream with black
  figures on the bookcloth.
- **Plano** (Drawing sheet / Blueprint): an ISO drawing sheet in black line weights with a drafting-blue accent, and a
  cyanotype blueprint at night with white lines and a pale yellow accent. The slab is always the paper of the title
  block, ruled in the drawing's ink.
- **Salidas** (Boarding pass / Departures): a pale grey boarding pass with navy ink, and a black departures hall at
  night. Board yellow is held for the answer alone (the figures and the Copy fill), and orange is the alert, used only
  for warnings.
- **Datasheet** (Page / Reader): an IC datasheet. White pages, steel-blue heads and accent, tinted header cells, and a
  dark inverted reader mode with sky-blue heads.
- **Grafito** (Light / Dark): the category standard played straight, with a fine neutral grey scale, hairlines and one
  cobalt accent (a lighter cobalt at night).
- **Rosa** (Newsprint / Wine): the maglia rosa. Claret ink on rose newsprint by day and wine-black with rose by night.
  The answer slab wears the leader's pink.

**The Loudest Slab Rule.** In every style and mode, the answer slab and its Copy button are the loudest element. Text
meets 4.5:1 and focus rings 3:1, the Copy fill meets 3:1 against the slab, and the figures meet 4.5:1 on their plate
(the bib, the flap) or on the slab. `contrast.test.ts` checks every style × mode from the real stylesheets, so a new
style is tested the moment it is registered.

**The Held Colour Rule.** Each style keeps its loudest colour for one job: Pit Gold for the answer in ISC, board yellow
for the answer in Salidas, and vermilion for errors in Prontuario. A style's alert colour is never its answer colour.

## Typography

**Display Font:** Jost (with Futura, Century Gothic)
**Body Font:** IBM Plex Sans (with Segoe UI, system-ui)
**Label/Mono Font:** IBM Plex Mono (with ui-monospace, SF Mono, Menlo)

These are ISC's faces. Other styles replace the three families and the display and answer treatments through
`--font-display/-sans/-mono`, `--display-weight/-tracking/-case` and `--answer-font/-weight/-tracking/-numeric`.
Sizes and the hierarchy never change.

**Character:** Jost's geometric, motorsport-poster shapes for names and the answer; Plex Sans for engineering prose
that stays readable when small; Plex Mono for anything you might type, compare or copy.

### Hierarchy
- **Answer** (600, 2rem): the answer value in the slab. It is the largest text on any screen, and a multi-line tool
  result switches to Mono at 1rem.
- **Display** (600, 1.625rem, +0.01em): `h1`, the view titles.
- **Headline** (600, 1.25rem): `h2`, script names on the sheet and empty-state titles.
- **Title** (600, 1rem): `h3`, section heads inside a sheet and the brand name in the rail (1.0625rem).
- **Body** (400, 0.90625rem, 1.5): prose, descriptions and rail items. Long help text is capped at about 52ch with
  1.55–1.6 line height.
- **Body small** (0.8125rem): buttons, field labels, notes.
- **Data** (Mono 500–600, 0.75–0.8125rem): values, units, variable names (`.var`, in the accent colour, 600), chips
  and `kbd`.
- **Label** (Mono, 0.6875rem, +0.08em, uppercase, muted): group labels in the rail and palette, and field-group
  labels in forms.

### Style faces
- **Prontuario:** Source Serif 4 for display and the answer (700, lining tabular figures), Source Sans 3, Source Code Pro.
- **Plano:** Barlow Semi Condensed in uppercase at +0.06em for headings (the drawing's lettering) and for the answer,
  Barlow for text, IBM Plex Mono.
- **Salidas:** Red Hat Display 700 for headings, Red Hat Text, and Red Hat Mono for data and for the answer (tabular,
  slashed zero, +0.04em), as on a board.
- **Datasheet:** Arimo throughout (700 heads in steel blue, through `--heading`), Cousine for data.
- **Grafito:** Geist Sans with tight tracking (−0.015em heads, −0.02em answer), Geist Mono.
- **Rosa:** Schibsted Grotesk at 800 for heads and the answer, IBM Plex Mono for data.

### Named Rules
**The Mono-Means-Data Rule.** If a reader could type it, copy it or check it against an option (a number, a unit, a
variable, a key), it is set in the style's mono (`--font-mono`; Plex Mono in ISC). Prose is never set in Mono.

**The Scale-Follows-Settings Rule.** Every size is in rem, and `html` scales by `--text-scale` (Settings > Text size).
Never set text in px.

## Layout

A two-column desktop shell: a full-height glass **rail** (236px, flush with the window edge and its corners) and a
scrolling **view**. The view's content is centred and capped at 1280px so the fields and the Copy button never drift
apart. Its padding is `max(20px, (100% - 1280px) / 2)` inline.

Below 1100px wide the rail collapses to 64px of icons only. Labels stay in the DOM as accessible names, and section
headers become 1px rules. On macOS the rail leaves 48px at the top for the traffic lights, plus a drag region.

The Solve view is a vertical stack with 16px gaps: problem pane, past-question pane, matches, then the script sheet.
Inside a sheet, fields flow in an auto-fill grid (`minmax(210px, 1fr)`, 12px gap), and the answer slab sticks to the
bottom of the scroll view while its place in the sheet is below the fold.

Spacing is a 4px-based scale (4, 8, 12, 16, 20, 24, 32). Panes pad at 16–20px, and dense lists at 7–10px vertically.

The layout is the same in every style: no style moves, adds or removes a region. Settings > Look opens with the style
gallery, a grid of cards (`minmax(220px, 1fr)`), each showing a live light and dark sample of the style side by side,
with its name, its two mode names and a one-line description. Below the gallery are the Theme choices: Follow the OS,
Dark or Light.

## Elevation & Depth

In ISC, depth is layered glass, not lift. A pane is a translucent fill with `backdrop-filter: blur(22px) saturate(140%)`,
a 1px edge, a 1px inner top highlight and one soft ambient drop shadow. Nothing floats higher on hover. The only panes
that sit above others are the sticky answer slab, which adds an upward shadow so fields can scroll under it, and the
⌘K palette over a 32% black scrim.

### Shadow Vocabulary
- **Pane** (`box-shadow: inset 0 1px 0 var(--glass-hi), 0 18px 40px -22px rgba(0,0,0,.75)`; on paper the drop is
  `0 18px 40px -24px rgba(6,66,41,.35)`): every glass pane.
- **Sticky slab** (`0 -10px 18px -10px rgba(0,0,0,.35)` plus the pane shadow): the answer slab while it is pinned to the
  bottom.
- **Focus halo** (`0 0 0 3px var(--accent-soft)`): fields on focus, together with an accent border.

The slab's shadow is `var(--slab-ring), var(--slab-shadow), var(--shadow)`; ISC leaves the ring empty and uses the
upward `--slab-shadow` above. The palette scrim is `--scrim`.

### Style materials
Glass belongs to ISC. Every other style sets `--glass-filter: none` and uses opaque panes, so solid mode looks the same,
and with an opaque `--ground-tint` window vibrancy shows nothing through.
- **Prontuario:** paper on a desk. Soft, short drop shadows, grain only on the bookcloth (22%).
- **Plano:** no shadows at all (`--shadow` is empty); line weights do the work. A 2px slab border with an inset double
  rule (`--slab-ring`), and a soft glow under the blueprint's grain at night.
- **Salidas:** flat card stock with a faint drop, no texture.
- **Datasheet:** a 1px print shadow, no texture.
- **Grafito:** hairline panes with a whisper of shadow; the answer slab alone is raised, by a long drop tinted with
  cobalt.
- **Rosa:** newsprint grain (30% by day, 20% by night) and a rose glow behind the wine ground.

### Named Rules
**The Solid Fallback Rule.** Every glass surface has an opaque twin. `[data-solid]` (Settings) and
`prefers-reduced-transparency` swap the glass for `--glass-solid`, remove the blur and turn off the grain. A new pane is
not done until it reads correctly in solid mode.

**The Ground Is Lit Rule.** In ISC the ground is never flat: radial glows of green (and gold) plus an SVG fractal-noise grain
blended with overlay (32% night, 42% paper). In window-vibrancy mode the same glow sits over a tint instead of an opaque
colour.

**The Image-Layer Slab Rule.** The slab paints `var(--slab-ornament), var(--answer-bg), var(--glass-solid)`, so
`--answer-bg` and `--slab-ornament` must be image layers (a gradient, even for a flat colour:
`linear-gradient(#151515, #151515)`). A plain colour in a non-final background layer voids the whole declaration.
The build writes `--slab-cell` and `--value-bg` as gradients too. The reverse holds for shadows:
`--slab-ring`, `--slab-shadow` and `--shadow` are colours only, because an image in `box-shadow` voids the shadow. The
contrast test checks both.

## Shapes

The shapes are soft and rounded, sized by how much a surface holds: fields 10px, list rows and wells 13px, panes and the
answer slab 18px, top-level sheets, the problem pane and the palette 22px. Every button and chip is a full pill
(999px), and `kbd` hints are 6px. Category markers are 8px dots. Edges are always 1px hairlines at 10–14% opacity, never
heavy strokes. The rail alone is square: it is part of the window, not a pane.

Each style owns its corners through `--r-sm/md/lg/xl/pill` and `--slab-radius`, and components only ever use those
tokens:
- **Prontuario:** 3–6px, with controls at 4px (no pills) and a 4px slab.
- **Plano:** square everywhere (0), with line weights instead of radii.
- **Salidas:** 6–14px with pill buttons, and a 10px slab.
- **Datasheet:** near-square, 2–3px, with 3px controls and a 2px slab.
- **Grafito:** 6–12px with 8px controls, and a 10px slab.
- **Rosa:** 6–14px with pill buttons, and a 12px slab.

## Components

### Buttons
Tactile pills that press in. They are physical but not shouty.
- **Shape:** full pill (999px), 34px tall (28px for the small size).
- **Default:** glass-strong fill, hairline edge, body-small text at 500 weight.
- **Primary:** the theme accent fill (Pit Gold with On-Gold text at night, Racing Green 500 with white text on paper),
  no edge, 600 weight.
- **Ghost:** transparent. A hover tint appears on hover.
- **Hover / Active:** the edge turns accent at 45%, primary brightens to 1.06, and every button scales to 0.98 when
  pressed. Transitions take 140ms `cubic-bezier(0.2, 0.8, 0.2, 1)`.
- **Key hints:** a `kbd` inside a button inherits the button's text colour and edge, so it is never muted on a fill.

### Chips
- **Style:** pill, 24px tall, Mono xs at 500. The default is accent text on a 14% (night) or 9% (paper) accent tint.
- **Variants:** quiet (hover tint with secondary text) for past-question links and "not used" values; warn (warn tint
  with warn text).

### Cards / Containers
- **Corner Style:** 18px panes, 22px for top-level sheets and the problem pane.
- **Background:** glass (white at 5.5% night, 60% paper), or glass-strong for the palette.
- **Shadow Strategy:** the Pane shadow (see Elevation & Depth).
- **Border:** 1px glass edge.
- **Internal Padding:** 20px for sheets, 12–16px for smaller panes.
- **Wells:** recessed sub-surfaces (black at 24% night, white at 72% paper) with a 13px radius, for grouped read-only
  content. Equation blocks use the `code` tint and render KaTeX left-aligned at 1.15em.

### Inputs / Fields
- **Style:** a 38px well with a 10px radius. The fill is dark at 30% at night and pure white on paper, the edge is
  white at 14% (night) or Sand (paper), and units sit outside the field in muted Mono.
- **Focus:** the border becomes the focus colour (gold at night, green on paper) with a 3px accent-soft halo. The global
  `:focus-visible` ring is 2px in the same colour with a 2px offset.
- **Provenance:** a value read from the pasted problem gets a "from the problem" tag. Solved values appear in the
  accent ink at 600.

### Navigation
- **Rail:** glass, flush to the window. At the top are the brand mark (the ISC logo masked to the accent ink) and a
  search button that opens ⌘K. Below are icon and label items (9px × 12px padding, 13px radius, secondary text). Hover
  is a 6% tint, and the selected item is a 10% tint with primary text at 600, its icon in the accent.
- **Palette (⌘K):** a 640px glass-strong sheet at 12vh over a scrim. The input is 1rem with no box, and focus shows as
  a 2px underline. Rows are 13px-radius and the selected row uses the selected tint. Groups have Mono uppercase labels.

### Answer Slab (signature)
The pit-board. In ISC it is a gold-edged slab (18px radius) with a 135° gradient: Pit Gold at 17%→4% over solid glass
at night, and Racing Green → Racing Green 600 on paper. Inside are a Mono label in Pit Gold 400, the value in Jost 600
at 2rem in white, a rule-year pill, and the **Copy** button: a Pit Gold fill with On-Gold text and an inline ⌘↵ hint.
Focus rings inside the slab turn white, because a gold ring disappears on gold. Notes and other roots appear in white
at 72–80%. The slab is the only surface that is always visible.

Every part of the slab is a token, so a style restyles it without touching the component: shape (`--slab-radius`,
`--slab-border`, `--slab-ring`, `--slab-ornament`), fill (`--answer-bg`), label cell (`--slab-cell`), figures
(`--answer-font/-weight/-tracking/-numeric`, `--value-text` on an optional `--value-bg` plate), Copy (`--copy-bg`,
`--copy-text`; Pit Gold and On-Gold in ISC) and its one unit field (`--slab-field`, `-edge`, `-text`, `-muted`), which
has its own colours because the slab may be a reversed plate. A multi-line tool result drops the plate and stays plain
Mono. The chain view's result slab uses the same tokens.

### Style signatures
Each style adds one signature move, scoped under `[data-style='<id>']` in its own file:
- **Prontuario:** the answer is the page's one reversed plate, and the selected rail item carries a thumb-index tab, a
  7px block in the accent cut flush to the rail's outer edge.
- **Plano:** the slab is the title block: a 2px border with an inner double rule, and the actions in their own ruled
  cell. The ground carries the drawing frame, a 1px border 8px in from the window with zone ticks along every edge.
- **Salidas:** the slab is a dark board plate. Its yellow figures sit on a two-tone split-flap tile, with the hinge
  notched only at the ends so no line crosses the figures. A dashed perforation separates the Copy stub, as on a
  boarding pass.
- **Datasheet:** the slab is a characteristics row: a tinted, ruled symbol cell for the label, the value, then the
  actions in their own ruled cell. Heads are steel blue.
- **Grafito:** a raised neutral slab (white by day, graphite at night) with a cobalt Copy.
- **Rosa:** the slab wears the maglia rosa pink, and the figures are printed in claret on a white race-number bib. Copy
  is wine-black with pink text.

### Style Gallery (Settings > Look)
The style picker is a radio group of cards (arrow keys move between them). Each card is a pair of live previews (a pane
with a heading and two lines, and a slab with the value and Copy) drawn by nesting `data-style` and `data-theme`, so
the previews use the style's real tokens even when the window is in another style. The selected card has an accent
border and a 3px accent-soft halo.

### Notes
Inline status blocks (13px radius, 10px × 12px padding, body-small). The background is a hover tint, or the status soft
tint for bad and warn notes. Only the `strong` lead-in takes the status colour; the rest stays readable text.

## Do's and Don'ts

### Do:
- **Do** keep one loud surface per screen: the answer slab with its Copy button (gold in ISC).
- **Do** set every value, unit, variable and shortcut in the style's mono (IBM Plex Mono in ISC).
- **Do** use `--ink-accent` (Racing Green 500 on paper, Pit Gold 400 at night) for accent text.
- **Do** give every new glass pane a `[data-solid]` and reduced-transparency twin, and check it in both themes.
- **Do** keep motion at or under 140–200ms and let `prefers-reduced-motion` remove it.
- **Do** size text in rem so Settings > Text size scales it.
- **Do** add a row to `contrast.test.ts` for any new text-on-surface pairing.
- **Do** ship a new style as a full set: a `src/styles/<id>.css` file that defines every colour token for light and
  dark plus at most a small scoped signature block, an entry in `src/lib/styles.ts` (id, name, light and dark mode names,
  a one-line description), and its fonts imported in `main.ts`, with the style file imported after `app.css`.
- **Do** keep a style's signature to one move that touches the slab, the ground or the rail selection.
- **Do** read every colour, radius and shadow from tokens (`--scrim`, `--heading`, `--copy-bg`...) so that every style
  reaches it.

### Don't:
- **Don't** put gold text on a light ground. On paper, gold is only a fill (the Copy button, the slab label on green).
- **Don't** copy MingoCAN's app.css or colours. MingoSolve shares the ISC tokens, not that app's look.
- **Don't** lift panes on hover. Depth is the glass layer (or, in other styles, the line and the page), and only the
  answer slab is raised.
- **Don't** use muted text for anything a user must read to answer correctly. It is for metadata only.
- **Don't** use a gold focus ring on gold or gold-tinted surfaces. Inside the answer slab, focus is white.
- **Don't** use square buttons or heavy borders in ISC. Controls are pills and edges are hairlines; other styles set their
  own corners through the radius tokens, never with literal values.
- **Don't** change layout, components, copy or behaviour per style. A style lends only type, palette, material and
  density, plus one signature move.
- **Don't** put a plain colour in `--answer-bg` or `--slab-ornament`, or an image in any shadow token.
- **Don't** make a style's answer quieter than its chrome, or give its answer colour a second job.
