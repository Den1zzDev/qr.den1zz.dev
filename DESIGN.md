---
name: QR Code Studio
description: Local, client-rendered QR studio with an AMOLED glass-panel workbench identity.
colors:
  background: "#000000"
  surface: "#0a0a0a"
  surface-raised: "#141414"
  surface-selected: "#181818"
  surface-sunken: "#050505"
  paper: "#ffffff"
  paper-off: "#f2f2f2"
  checker-light: "#efefef"
  checker-dark: "#d7d7d7"
  line: "rgba(255, 255, 255, 0.12)"
  line-strong: "rgba(255, 255, 255, 0.48)"
  line-control: "rgba(255, 255, 255, 0.38)"
  text: "#ffffff"
  text-dim: "#adadad"
  text-muted: "#8a8a8a"
  text-faint: "#7a7a7a"
  accent: "#00f0ff"
  accent-bright: "#64f6ff"
  on-accent: "#000000"
  success: "#34d399"
  warning: "#fbbf24"
  danger: "#fb7185"
typography:
  wordmark-mobile:
    fontFamily: "Maple Mono, ui-monospace, SFMono-Regular, Menlo, monospace"
    fontSize: "1.75rem"
    fontWeight: 700
    lineHeight: 1.15
    letterSpacing: "-0.04em"
  wordmark:
    fontFamily: "Maple Mono, ui-monospace, SFMono-Regular, Menlo, monospace"
    fontSize: "2.25rem"
    fontWeight: 700
    lineHeight: 1.15
    letterSpacing: "-0.04em"
  tagline:
    fontFamily: "Maple Mono, ui-monospace, SFMono-Regular, Menlo, monospace"
    fontSize: "0.8125rem"
    fontWeight: 400
    lineHeight: 1.6
    letterSpacing: "0em"
  body:
    fontFamily: "Maple Mono, ui-monospace, SFMono-Regular, Menlo, monospace"
    fontSize: "0.9375rem"
    fontWeight: 400
    lineHeight: 1.6
    letterSpacing: "0em"
  panel-title:
    fontFamily: "Maple Mono, ui-monospace, SFMono-Regular, Menlo, monospace"
    fontSize: "0.6875rem"
    fontWeight: 600
    lineHeight: 1.4
    letterSpacing: "0.12em"
  subhead:
    fontFamily: "Maple Mono, ui-monospace, SFMono-Regular, Menlo, monospace"
    fontSize: "0.875rem"
    fontWeight: 600
    lineHeight: 1.4
    letterSpacing: "0em"
  action:
    fontFamily: "Maple Mono, ui-monospace, SFMono-Regular, Menlo, monospace"
    fontSize: "0.8125rem"
    fontWeight: 600
    lineHeight: 1.4
    letterSpacing: "0em"
  label:
    fontFamily: "Maple Mono, ui-monospace, SFMono-Regular, Menlo, monospace"
    fontSize: "0.75rem"
    fontWeight: 500
    lineHeight: 1.5
    letterSpacing: "0em"
  data:
    fontFamily: "Maple Mono, ui-monospace, SFMono-Regular, Menlo, monospace"
    fontSize: "0.75rem"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "0em"
  field-mobile:
    fontFamily: "Maple Mono, ui-monospace, SFMono-Regular, Menlo, monospace"
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "0em"
rounded:
  xs: "5px"
  sm: "8px"
  control: "10px"
  stage: "16px"
  panel: "16px"
  pill: "999px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "20px"
  2xl: "24px"
components:
  panel:
    backgroundColor: "{colors.surface}"
    rounded: "{rounded.panel}"
    padding: "20px"
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.on-accent}"
    rounded: "{rounded.pill}"
    padding: "0 16px"
    height: "42px"
  button-secondary:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.text}"
    rounded: "{rounded.pill}"
    padding: "0 16px"
    height: "42px"
  input:
    backgroundColor: "{colors.surface-sunken}"
    textColor: "{colors.text}"
    rounded: "{colors.control}"
    padding: "10px 12px"
    height: "44px"
  option-selected:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.on-accent}"
    rounded: "{colors.control}"
    padding: "0 10px"
    height: "42px"
  stage:
    backgroundColor: "{colors.background}"
    rounded: "{rounded.stage}"
    padding: "20px"
---

# Design System: QR Code Studio

## Overview

**Creative North Star: "The Glass-Panel Workbench"**

The studio is a dark room with a single instrument on the bench. The page is one centred column of black trays, each with a 16-pixel radius and a hairline edge, stacked in the order the work happens: choose the payload, watch the code, then tune its shape, colour, and logo. Cyan is the instrument's signal. It marks the wordmark, the active choice, the primary export, the links, and keyboard focus. A white QR code sits on a black mat inside its tray, framed like a viewfinder, with a status pill above it reading the measured contrast.

The identity comes from commit `cadd240` and this document describes that world as it now ships. It is not a generic dark theme: the centred composition, the full-cyan selection bars, the pill exports, the large wordmark, and the matted code are the signature. Later work added debounced rendering, a mobile quick-action bar, WCAG contrast tokens, and stronger type; those changes are kept because they make the instrument easier to read, not because they change what it is.

This is an Operate surface. The controls are native where a native control exists, every measurement is visible, and the code is always one glance away from the control that changes it.

**Key Characteristics:**
- AMOLED black page with opaque near-black trays; no blur, no translucency, no coloured halo.
- One centred column, 45rem at its widest, with 20 to 24-pixel gaps between trays.
- A single cyan signal used for the wordmark, selection, the primary export, links, and focus.
- Pill buttons, 10-pixel control corners, 16-pixel trays, circular swatches.
- A black matted viewfinder around the code, with a four-module quiet zone inside it.
- Every styling decision reports a measurable consequence: contrast ratio, version, matrix size, error level, byte count.

## Colors

The palette is black on black with one electric cyan. Surfaces are separated by tone and a one-pixel line, never by shadow.

### Primary
- **Signal Cyan** (`#00f0ff`): the `qr.` wordmark, the selected option row, the selected export size, the Download PNG button, links, the skip link, and the focus ring. It is the only saturated UI colour at rest.
- **Signal Bright** (`#64f6ff`): hover state for cyan fills.
- **On Accent** (`#000000`): text and icons on a cyan fill.
- **Sheet Black** (`#000000`): the page and the viewfinder mat.
- **Tray Black** (`#0a0a0a`): every panel.
- **Raised Black** (`#141414`): idle option rows, secondary buttons, the payload menu surface.
- **Selected Black** (`#181818`): the open payload menu and its glide highlight.
- **Sunken Black** (`#050505`): fields and the status pill.
- **Paper** (`#ffffff`): the code's quiet-zone background and the dock thumbnail. A QR code needs a light ground; this is a code surface, not a UI surface.
- **Paper Off** (`#f2f2f2`): the second background preset, one step below paper.
- **Checker Light** (`#efefef`) and **Checker Dark** (`#d7d7d7`): the transparency checkerboard. It is light on purpose so black modules stay visible when the export is transparent.

### Lines and text
- **Hairline** (`rgba(255,255,255,0.12)`): panel edges, dividers, the viewfinder ring.
- **Strong Line** (`rgba(255,255,255,0.48)`): hover edges. It is brighter than the resting border so hover increases contrast instead of reducing it.
- **Control Line** (`rgba(255,255,255,0.38)`): field, button, option, swatch, and status borders. On the page, the tray, the raised row, and the sunken field it measures 3.34:1 to 3.55:1.
- **Primary Text** (`#ffffff`): values, headings, the wordmark.
- **Secondary Text** (`#adadad`): field labels, the tagline, body copy on panels.
- **Muted Text** (`#8a8a8a`): panel titles, hints, captions.
- **Faint Text** (`#7a7a7a`): placeholders and tertiary metadata. It still clears 4.5:1 on the tray and the sunken field.

### Semantic
- **Measured Green** (`#34d399`): a passing contrast check and a copied confirmation.
- **Measured Amber** (`#fbbf24`): a failing contrast measurement or an invalid payload.
- **Measured Rose** (`#fb7185`): a hard failure such as a file read error. It never fills a surface.

### Named Rules

**The One Signal Rule.** Cyan marks the wordmark, the active choice, the primary export, links, and focus. Selection is a cyan fill with black text, not a tint. Cyan never becomes body copy, a decorative border, or a background wash.

**The Measurement Rule.** Amber means a value the user can tune. Rose means something did not run. A contrast miss is never an alarm colour.

**The Opaque Glass Rule.** The trays carry the `cadd240` glass-panel silhouette, but the material is opaque. No backdrop blur, no translucent white overlay, no gradient sheen, and no coloured glow. Depth comes from the step from `#000000` to `#0a0a0a`, one hairline, and a one-pixel top bevel.

**The Paper Rule.** White appears only where a QR code needs a light ground: the code's quiet zone, its background presets, the transparency checker, and the dock thumbnail. White is never a UI surface.

## Typography

**Display Font:** Maple Mono, self-hosted variable WOFF2 at `public/fonts/maple-mono.woff2`
**Body Font:** Maple Mono (with ui-monospace, SFMono-Regular, Menlo, monospace)
**Label/Mono Font:** Maple Mono

**Character:** One family, used honestly. The same face carries the wordmark, the buttons, the fields, and the ratios, which keeps the studio feeling like one instrument. Numerals are tabular wherever a value can change.

### Hierarchy
- **Wordmark** (700, 2.25rem desktop / 1.75rem mobile, 1.15 line-height, -0.04em): the page's only h1, centred, `qr.` in cyan and the domain in white.
- **Tagline** (400, 0.8125rem, 1.6 line-height, 34rem measure): the line under the wordmark. It is the page's introduction; there is no separate hero.
- **Panel title** (600, 0.6875rem, 0.12em tracking, uppercase, `#8a8a8a`): the name of each tray.
- **Subhead** (600, 0.875rem, 1.4 line-height): empty-state and encoding-error titles.
- **Body** (400, 0.9375rem, 1.6 line-height, 66ch measure): explanatory copy and hints.
- **Action** (600, 0.8125rem, 1.4 line-height): buttons and control labels.
- **Label** (500, 0.75rem, 1.5 line-height): field labels and option names.
- **Field** (400, 1rem mobile / 0.9375rem desktop, 1.4 line-height): input values. The 16-pixel mobile floor prevents iOS zoom.
- **Data** (400, 0.75rem, tabular): ratios, versions, byte counts, hex values, and status text.

### Named Rules

**The Readout Rule.** Any number that can change while the user works is set in tabular figures: contrast ratios, byte counts, matrix size, version, and error level.

**The No-Costume Rule.** Monospace is used because the interface is full of measured values. Prose still gets a 66ch measure and 1.6 leading.

## Layout

The page is one centred column, 45rem (720px) including 1rem side padding on phones and 1.5rem above 640px. The masthead is centred. The workbench stacks the payload tray, the preview tray, the shapes tray, the palette tray, and the logo tray with a 20-pixel gap on phones and 24 pixels above 640px. There is no desktop two-column split; the centred stack is the `cadd240` composition and it keeps the code and its controls in one reading line.

Inside the preview tray, the viewfinder is capped at 25rem and centred, and the action block below it matches that measure, so the code and its controls sit on one axis. The status pill sits above the stage, centred, and may run wider than it on desktop. The palette uses an intrinsic two-column grid above 16rem per row. The shapes tray uses `repeat(auto-fit, minmax(11rem, 1fr))`, so large text collapses the columns instead of truncating labels.

Spacing is a 4-pixel base: 4, 8, 12, 16, 20, 24. Tray interiors use 16 to 20 pixels. Labels sit 6 pixels above their field and hints 5 pixels below. The export grid is one column below 360 pixels and two columns above it.

Touch targets: 44-pixel fields, 42-pixel option rows and buttons, 32-pixel segments, 32-pixel swatches, 28-pixel hex readouts and 24-pixel match links, 36-pixel header controls, and a 44-pixel dock thumbnail. Below 1024 pixels, a fixed quick-action bar holds the thumbnail, the version and byte count, and the PNG export; it hides while the preview tray is on screen.

## Elevation & Depth

There are no drop shadows. On pure black they would be invisible and a coloured halo would read as decoration. The trays use one inset top highlight (`rgba(255,255,255,0.045)`) as a milled edge, plus the one-pixel hairline border. The viewfinder adds an inset one-pixel ring and an inset black vignette so the code reads as a window cut into the tray.

Motion lives in the controls: the payload menu glides between options, notices fade and lift into place, and buttons lift one pixel on hover. The preview has one authored moment, a cyan scan pass that crosses the stage once when the page takes over and is then removed from the DOM. The code does not animate while the user types.

### Named Rules

**The Flat-By-Default Rule.** Surfaces are flat. A shadow appears only when it explains a physical layer; on this palette that means an inset hairline or a vignette, never a cast shadow or a glow.

## Shapes

Trays and the viewfinder use 16 pixels. Fields and option rows use 10 pixels, and swatches and the custom picker are circles or 8-pixel squares. The payload menu uses 12 pixels with 8-pixel rows. Small chips use 5 to 8 pixels. Buttons, segments, status pill, and selection bars are pills. Borders are one pixel at every size. There are no thick accent edges, corner brackets, or notched panels. The square is the recurring form: the viewfinder, the logo thumbnail, the checkbox, and the matrix all share it.

## Components

### Masthead
- **Wordmark:** centred h1 with a home link. `qr.` is Signal Cyan, the domain is white. It is not a link row and it carries no badge, LED, or header icon.
- **Tagline:** `Local, client-side QR studio built in pure Rust & WebAssembly.` It carries the privacy claim without a status chip.

### Footer
- **Rule:** none. The footer is centred text on the page background: the MIT line, then a cyan `GitHub repository` link.

### Panels
- **Shape:** 16-pixel radius, `#0a0a0a`, one-pixel hairline, inset top bevel.
- **Padding:** 16 pixels on phones, 20 pixels above 640px. The head sits above the body with more space than separates it from the tray edge.
- **Head:** panel title on the left, one optional text action on the right. The payload tray's action is the content-type menu.

### Buttons
- **Shape:** pill, 42-pixel minimum height, 13-pixel semibold label, inline 15 to 16-pixel icon with a 7-pixel gap.
- **Primary:** Signal Cyan fill with black text; hover shifts to Signal Bright and lifts one pixel. Used once in the preview: Download PNG.
- **Secondary:** Raised Black fill with a Control Line border; used for Download SVG and for Upload.
- **Ghost:** transparent fill with a Control Line border; used for the two copy actions.
- **Focus:** 2-pixel cyan outline with a 2-pixel offset. Active returns to the resting position. Disabled uses 45% opacity and a not-allowed cursor.

### Inputs
- **Shape:** 44-pixel minimum height, 10-pixel radius, 1rem text on mobile and 0.9375rem above 640px, 12-pixel horizontal padding.
- **Default:** Sunken Black fill, Control Line border, white value, faint placeholder.
- **Focus:** a cyan border plus the shared 2-pixel cyan outline at a 2-pixel offset.
- **Clear action:** a 32-pixel icon button inside the URL field, labelled "Clear the destination".

### Segmented export size
- **Shape:** three free-standing 32-pixel pills with a 4-pixel gap. Native radios sit inside labels.
- **Selected:** cyan fill, black text, cyan border.
- **Idle:** Raised Black fill, Control Line border, secondary text.
- **Keyboard:** arrow keys move the radio selection; `:has(input:focus-visible)` draws the ring.

### Option rows
- **Shape:** 42-pixel row, 10-pixel radius, icon, label, trailing check.
- **Selected:** cyan fill, black icon and label, black check.
- **Idle:** Raised Black fill with a Control Line border and secondary text.
- **Purpose:** module shape, eye frame shape, and eye dot shape. The icon is a miniature of the real geometry.

### Swatches
- **Shape:** 32-pixel circles; the custom picker is a 32-pixel rounded square with a small pipette badge, so it reads as a picker rather than a second preset.
- **Selected:** a 3.5-pixel white ring with a 2-pixel tray gap. The ring carries the selection on its own; no check glyph sits on the colour.
- **Disabled:** 40% opacity. The background row dims while transparency is on.

### Palette
- **Rows:** name, a measured contrast badge, a copyable hex value, a 32-pixel picker, and the role's presets.
- **Badge:** green check and ratio when the role clears 3:1, amber warning and ratio when it does not. It is text and an icon, not a tinted chip.
- **Hex readout:** a 28-pixel button. Activating it copies the hex and announces the result through the global live region. The background value is disabled while transparency is on.
- **Match action:** a 24-pixel cyan text link, `Match Code` or `Match Frame`, that applies the previous role's colour. Its accessible label names the source role and it is disabled when the values already match.
- **Background presets:** paper, paper off, and black. The off-white and black are distinct surfaces, not near-duplicate swatches.

### Preview
- **Status pill:** above the viewfinder, centred, Sunken Black with a Control Line border, 11-pixel tabular text. It carries a 6-pixel LED, the state (`Contrast passes` or `Low contrast`), the three ratios, and the version, matrix size, and error level. Below 480px the ratios and separators hide and the pill keeps the state and metadata, wrapping to two centred lines when the width cannot hold both; the ratios stay available in the palette.
- **Viewfinder:** a 16-pixel stage, black mat, 12-pixel inner padding, a one-pixel inset ring, and an inset vignette. The code is square and can grow to 25rem. The white margin inside the SVG is the four-module quiet zone, not CSS padding.
- **Transparent:** the mat becomes the light checkerboard so black modules stay visible and the exported transparency is obvious.
- **Caveat:** one line under the stage. It says `Contrast checked. Test-scan before printing.`, switches to `Contrast checked against white.` when the background is transparent, and turns amber with `Contrast is below 3:1. Adjust the palette before exporting.` when a role fails. A transparent failure keeps the white surface in the sentence.
- **Export size:** the segmented pills, the physical print size at 300 dpi, then a two-column grid: Download PNG and Download SVG, Copy PNG and Copy SVG.
- **Notice:** a 20-pixel line under the grid, rendered only while a message exists. 12-pixel text, green for success and rose for failure, announced through the global live region.

### Payload menu
- **Trigger:** a 32-pixel chip with the content icon, the type name, and a chevron.
- **Menu:** a 12-pixel surface with 36-pixel rows, a transform-glide highlight, and the current row checked in cyan.
- **Keyboard:** the trigger is a button; the menu supports arrow keys, Enter, Escape, and outside-click dismissal.

### Quick-action bar
- Fixed below 1024 pixels, 67 pixels tall plus the safe-area inset, with a 44-pixel code thumbnail on white, the version and matrix size, the byte count and current PNG size, and a cyan PNG pill. It disappears while the preview tray is in view, and the thumbnail mounts only while the bar is visible. Copy confirmations reuse this caption instead of adding a toast.

## Motion

- **Durations:** 100 to 180ms for colour and border changes, 220ms for the menu glide, 1050ms for the one scan pass, 160ms for the notice.
- **Easing:** `cubic-bezier(0.23, 1, 0.32, 1)` for movement; linear or ease for colour.
- **Lift:** buttons translate up one pixel on hover and return to rest on press.
- **Reduced motion:** `prefers-reduced-motion` removes the scan pass, the label swap blur, the notice lift, and all transforms.
- **No loop:** nothing pulses, blinks, marquees, or counts on its own. The green LED is static.

## Accessibility

- **Text contrast:** every text token clears 4.5:1 on the tray, the sunken field, and the page. `#7a7a7a` is the faintest usable text and sits at 4.7:1 on the sunken field.
- **Control contrast:** Control Line is 38% white, which measures 3.34:1 to 3.55:1 across the page, tray, raised row, and sunken field. Strong Line is 48%, so hover states clear the same floor.
- **Focus:** every interactive element shows a 2-pixel cyan outline with a 2-pixel offset through `:focus-visible`.
- **Targets:** 24 by 24 CSS pixels minimum; the smallest controls are the 24-pixel match link and the 28-pixel hex readout.
- **Semantics:** one h1, an h2 per tray (the preview's is visually hidden), fieldsets with legends for the three shape groups, native radios, native colour inputs, native file input, labelled icon buttons, `role="img"` on the stage, and a polite live region for state and copy confirmations.
- **Motion and zoom:** the layout survives 200% text without clipping or horizontal scroll; the shapes grid and palette stack by intrinsic size.

## Do's and Don'ts

### Do:
- **Do** keep the centred single-column stack. It is the `cadd240` composition.
- **Do** use cyan for selection, the wordmark, links, focus, and the one primary export.
- **Do** show the measured contrast ratio next to each colour control and in the status pill.
- **Do** keep the four-module quiet zone in the preview and every export.
- **Do** show the physical print size under the export size.
- **Do** keep the code's ground white or light, and let a transparent export show the checkerboard.
- **Do** keep sensitive payloads summarised without Wi-Fi passwords or email bodies.
- **Do** let a hex readout copy on activation and confirm through the notice and live region.
- **Do** collapse grids by intrinsic size (`auto-fit` with a rem minimum) so 200% text never truncates a label.

### Don't:
- **Don't** add blur, translucent white washes, gradients, cast shadows, or coloured glow. The glass-panel silhouette is opaque here.
- **Don't** split the workbench into two desktop columns. The single centred stack is the identity.
- **Don't** turn cyan into body copy, a decorative border, or a background wash.
- **Don't** use a rounded chip for a control that reads better as a field, a row, or a circle.
- **Don't** claim a code was scan-verified when only contrast was measured.
- **Don't** hide the export actions behind a sticky panel on short viewports; the dock covers that case below 1024 pixels.

## Deliberate Exceptions

The detector reports zero findings on this build. Three choices are intentional and stay:

1. **Full cyan selection bars.** They are the `cadd240` signature and the strongest readable state. White-on-raised selection was the regression this work removes.
2. **Opaque "glass" panels.** The silhouette and name come from `cadd240`; the material is honest AMOLED black. No blur is claimed or shipped.
3. **A light checkerboard under a dark page.** It is the only honest way to preview a transparent export whose modules are black.
4. **No cyan halo on the primary or the LED.** `cadd240` carried a 16-pixel cyan glow on the primary button and the status LED. It is the one piece of decorative glow on the reference, and it is deliberately removed: the flat fill carries the signal without the haze. The detector's `dark-glow` rule therefore reports nothing.

## Detector Record

- `cadd240` reference build: 7 findings with `--no-config` (2 low-contrast, 3 dark-glow, 1 nested-cards, 1 skipped-heading).
- Previous HEAD: 7 findings with `--no-config` (7 cramped-padding).
- This build: 0 findings at 1440 with `detect crates/qr-frontend/dist/index.html --no-config --json`.
- With design context, the same scan is 0. Without the three scoped Preflight ignores in `.impeccable/config.json`, it reports only Tailwind Preflight's vendor defaults (`#e5e7eb`, `#9ca3af`, Apple Color Emoji). SFMono-Regular is part of the declared type stack.
