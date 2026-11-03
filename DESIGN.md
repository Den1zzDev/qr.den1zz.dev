---
name: QR Code Studio
description: Local, client-rendered QR studio with an AMOLED instrument-panel identity.
colors:
  background: "#000000"
  surface: "#0a0a0a"
  surface-raised: "#101010"
  surface-sunken: "#050505"
  line: "rgba(255, 255, 255, 0.1)"
  line-strong: "rgba(255, 255, 255, 0.22)"
  line-control: "rgba(255, 255, 255, 0.3)"
  text: "#ffffff"
  text-dim: "#adadad"
  text-muted: "#8a8a8a"
  text-faint: "#7a7a7a"
  accent: "#00f0ff"
  accent-bright: "#64f6ff"
  accent-dim: "#008b99"
  accent-wash: "rgba(0, 240, 255, 0.08)"
  accent-ring: "rgba(0, 240, 255, 0.35)"
  success: "#34d399"
  warning: "#fbbf24"
  danger: "#fb7185"
typography:
  display:
    fontFamily: "Maple Mono, ui-monospace, monospace"
    fontSize: "1.5rem"
    fontWeight: 700
    lineHeight: 1.25
    letterSpacing: "-0.03em"
  display-mobile:
    fontFamily: "Maple Mono, ui-monospace, monospace"
    fontSize: "1.375rem"
    fontWeight: 700
    lineHeight: 1.25
    letterSpacing: "-0.03em"
  wordmark:
    fontFamily: "Maple Mono, ui-monospace, monospace"
    fontSize: "1.125rem"
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: "-0.03em"
  body:
    fontFamily: "Maple Mono, ui-monospace, monospace"
    fontSize: "0.9375rem"
    fontWeight: 400
    lineHeight: 1.6
    letterSpacing: "0em"
  intro:
    fontFamily: "Maple Mono, ui-monospace, monospace"
    fontSize: "0.875rem"
    fontWeight: 400
    lineHeight: 1.6
    letterSpacing: "0em"
  action:
    fontFamily: "Maple Mono, ui-monospace, monospace"
    fontSize: "0.8125rem"
    fontWeight: 600
    lineHeight: 1.4
    letterSpacing: "0em"
  subhead:
    fontFamily: "Maple Mono, ui-monospace, monospace"
    fontSize: "0.875rem"
    fontWeight: 600
    lineHeight: 1.4
    letterSpacing: "0em"
  field-mobile:
    fontFamily: "Maple Mono, ui-monospace, monospace"
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "0em"
  label:
    fontFamily: "Maple Mono, ui-monospace, monospace"
    fontSize: "0.75rem"
    fontWeight: 500
    lineHeight: 1.5
    letterSpacing: "0em"
  micro:
    fontFamily: "Maple Mono, ui-monospace, monospace"
    fontSize: "0.6875rem"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "0em"
  panel-title:
    fontFamily: "Maple Mono, ui-monospace, monospace"
    fontSize: "0.8125rem"
    fontWeight: 600
    lineHeight: 1.4
    letterSpacing: "0.1em"
rounded:
  xs: "5px"
  segment: "7px"
  sm: "8px"
  md: "10px"
  stage: "14px"
  lg: "16px"
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
    rounded: "{rounded.lg}"
    padding: "20px"
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.background}"
    rounded: "{rounded.pill}"
    padding: "10px 16px"
    height: "42px"
  button-secondary:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.text}"
    rounded: "{rounded.pill}"
    padding: "10px 16px"
    height: "42px"
  input:
    backgroundColor: "{colors.surface-sunken}"
    textColor: "{colors.text}"
    rounded: "{rounded.md}"
    padding: "10px 12px"
    height: "44px"
  option-selected:
    backgroundColor: "{colors.accent-wash}"
    textColor: "{colors.text}"
    rounded: "{rounded.md}"
    padding: "0 10px"
    height: "42px"
---

# Design System: QR Code Studio

## Overview

**Creative North Star: "The Darkroom Instrument"**

The studio is a precision instrument in a dark room. The page is a black field, the controls are milled trays, and cyan appears only where the tool is measuring, selecting, or acting. Nothing glows for atmosphere. The monospace face suits a tool that reports versions, byte counts, and contrast ratios; it is not a costume for "technical".

The layout reads as a workbench. The payload and its styling controls form the left bench; the generated code and export actions form the right inspection tray. On a phone the preview follows the payload and a compact quick-action bar appears once the preview scrolls away, so the code and its export stay one tap from the controls. Every panel has the same hairline border and the same 16-pixel radius. Density is deliberate: labels are compact, values are tabular, and the largest type belongs to the page title, not to decoration.

This is an Operate surface. Familiar affordances are kept familiar: native radio semantics, native colour pickers, native file input, standard download buttons. The character comes from restraint, measurement, and the way failure turns amber instead of shouting in red.

**Key Characteristics:**
- AMOLED black canvas with no drop shadows and no glass blur.
- One cyan signal colour, used for primary action, selection, and live state.
- A single monospace family with a fixed, compact role scale.
- Hairline borders for panels; 3:1 control borders for inputs and buttons.
- Every styling decision reports a measurable consequence.

## Colors

The palette is a dark instrument panel: near-black surfaces separated by hairlines, one electric cyan, and three semantic colours that appear only when they mean something.

### Primary
- **Signal Cyan** (#00f0ff): primary actions, selected options, active controls, links, focus rings, and the `qr.` wordmark. It is the only saturated colour at rest.
- **Signal Bright** (#64f6ff): hover state for the primary action.
- **Signal Ring** (rgba(0, 240, 255, 0.35)): the border around a selected option, segmented control, or URL note. It is a tint of the signal colour, never a fill.
- **Signal Dim** (#008b99): the default finder dot colour and quiet supporting marks. It is a value, not a UI accent.

### Neutral
- **AMOLED Black** (#000000): page and header background.
- **Tray Black** (#0a0a0a): panel surface.
- **Raised Black** (#101010): idle option rows and secondary buttons.
- **Sunken Black** (#050505): inputs and the QR stage, so fields read as recessed.
- **Hairline** (rgba(255, 255, 255, 0.1)): panel borders and dividers. Decorative separation only.
- **Strong Line** (rgba(255, 255, 255, 0.22)): the hover border for fields, options, and secondary buttons. It sits between the hairline and the control line.
- **Control Line** (rgba(255, 255, 255, 0.3)): borders of inputs, buttons, segmented controls, option rows, swatches, and status pills. This is the 3:1 boundary token.
- **Primary Text** (#ffffff): values and headings.
- **Secondary Text** (#adadad): field labels and body copy.
- **Muted Text** (#8a8a8a): panel titles and secondary metadata.
- **Faint Text** (#7a7a7a): hints, placeholders, and tertiary metadata; still at least 4.5:1 on the panel.

### Semantic
- **Measured Green** (#34d399): a passing contrast check, a copied confirmation.
- **Measured Amber** (#fbbf24): a failing contrast measurement, a payload that cannot be encoded. Amber is a measurement, not an alarm.
- **Measured Rose** (#fb7185): a hard error such as a failed file read. It never fills a large surface.

### Named Rules

**The One Signal Rule.** Cyan marks a control that is selected, focused, or primary. It never becomes body copy, a decorative border, or a static background wash larger than a chip. The transient scan pass is the one exception because it reads as a measurement, not decoration.

**The Measurement Rule.** A colour failure is amber, not red, because the user is tuning a value and can fix it. Rose is reserved for something that did not run at all.

## Typography

**Display Font:** Maple Mono (with ui-monospace, SFMono-Regular, Menlo, monospace)
**Body Font:** Maple Mono (same stack)
**Label/Mono Font:** Maple Mono, self-hosted variable WOFF2 at `public/fonts/maple-mono.woff2`

**Character:** One family, used honestly. The same face carries the wordmark, the buttons, the fields, and the ratios, which keeps the studio feeling like one instrument rather than a page of borrowed components. Numerals are tabular wherever a value can change.

### Hierarchy
- **Display** (700, 1.5rem on desktop / 1.375rem on mobile, 1.25 line-height, -0.03em): the page title only.
- **Wordmark** (700, 1.125rem, 1.2 line-height, -0.03em): the header identity, not a heading in the document outline.
- **Title** (600, 0.8125rem, 0.1em tracking, uppercase, #b8b8b8): panel titles. It is one step brighter than a field label so the region is unmistakable without competing with the content.
- **Body** (400, 0.9375rem, 1.6 line-height, max 66ch): explanatory copy.
- **Intro** (400, 0.875rem, 1.6 line-height, max 66ch): the line under the page title.
- **Action** (600, 0.8125rem, 1.4 line-height): buttons and compact control labels.
- **Subhead** (600, 0.875rem, 1.4 line-height): empty-state and encoding-error titles.
- **Field** (400, 1rem on mobile / 0.9375rem on desktop, 1.4 line-height): input values. The 16px mobile floor prevents iOS zoom.
- **Label** (500, 0.75rem, 1.5 line-height): field labels, option labels, and control names.
- **Data** (400–600, 0.75–0.8125rem, tabular numerals): ratios, versions, byte counts, hex values, status text, and hints.

### Named Rules

**The Readout Rule.** Any number that can change while the user works is set in tabular figures: contrast ratios, byte counts, matrix size, and version.

**The No-Costume Rule.** Monospace is used because the interface is full of measured values, not to signal that the product was made by developers. Prose still gets a 66ch measure and 1.6 leading.

## Layout

The page is capped at 70rem including 1.5rem side padding. Above 1024px the workspace is a two-column grid: a fluid left bench and a fixed 27.5rem preview tray. The preview becomes sticky only when the viewport is at least 850px tall, because a shorter viewport cannot show the full tray and would trap the export buttons below the fold.

Below 1024px the grid collapses to one column in this order: payload, preview, styling panels. Between 640px and 1023px the preview card is capped at 27.5rem and centred, so the QR does not grow into a poster while the controls stay full width. The export grid is one column under 400px and two columns above it.

Spacing is a 4px base: 4, 8, 12, 16, 20, 24. Panels are separated by 20px on mobile and 24px on desktop. Panel interiors use 16–20px. Labels sit 6px above their field; hints sit 5px below. A panel title sits above its content with more space than separates the title from the panel edge.

Touch targets: 44px fields, 42px option rows and buttons, 32px swatches, 32px segmented controls, 36px header icons, and 24px palette copy icons and hex readouts. The colour picker and presets sit in a single row that wraps rather than shrinking below 32px. Below 1024px, a fixed quick-action bar holds a 40px code thumbnail, the version and byte count, and the PNG export; it hides while the preview tray is on screen so it never covers the code.

## Elevation & Depth

There are no drop shadows. On pure black they would be invisible, and a coloured halo would read as decoration. Depth comes from three flat tones (black page, #0a0a0a panel, #050505 recessed field) separated by one-pixel borders. The QR stage adds a 1px inner highlight only through its border; it is not lifted.

Motion stays in the controls: the payload menu glides between options, notices fade and lift into place, and selected pills move by transform. The preview has one authored moment, a cyan scan pass that crosses the stage once when the page takes over and is then removed from the DOM. It does not animate on every keystroke, because a code that flickers while typing is noise rather than feedback.

### Named Rules

**The Flat-By-Default Rule.** Surfaces are flat at rest. A shadow appears only if it explains a physical layer, and on this palette that means it does not appear at all.

## Shapes

Panels use a 16px radius. The preview stage uses 14px, inputs and option rows use 10px, the segmented control uses a 7px inner radius, small chips use 8px, ratio badges use 5px, and swatches use a full circle. Buttons, status pills, and the wordmark's identity are pills. The QR itself is the only dense geometry on the page and is always presented inside a square stage with a 4-module quiet zone.

Borders are one pixel at every size. There are no thick accent edges, no corner brackets, and no notched panels. The square is the recurring form: the preview stage, the logo thumbnail, the checkbox, and the matrix all share it.

## Components

### Buttons
- **Shape:** pill, 42px minimum height, 13px semibold label, inline 15–16px icon with a 7px gap.
- **Primary:** Signal Cyan fill with black text; hover shifts to Signal Bright. Used once per preview panel: Download PNG.
- **Secondary:** raised black fill with a control-line border; used for Copy PNG.
- **Ghost:** transparent fill with a control-line border; used for the two SVG actions.
- **Hover / Focus:** 120ms colour and border transitions; a 2px cyan outline with 2px offset on keyboard focus; active state scales to 0.985.
- **Disabled:** 45% opacity, not-allowed cursor.

### Inputs / Fields
- **Shape:** 44px minimum height, 10px radius, 16px text on mobile (prevents iOS zoom) and 15px on desktop, 12px horizontal padding.
- **Default:** #050505 fill, 30% white border, white value, #7a7a7a placeholder.
- **Focus:** cyan border plus a 3px translucent cyan ring. The native outline is removed only because this ring replaces it.

### Segmented Control
- **Shape:** 10px container radius, 7px segment radius, 3px inset padding, 32px segment height.
- **Selected:** cyan wash at 8%, cyan text, 1px inset cyan ring. The selected state is a tint, not a solid fill, because it is a control state rather than a primary action.
- **Semantics:** native radio inputs inside labels. Arrow keys move the selection; focus is visible through `:has(input:focus-visible)`.

### Option Rows
- **Shape:** 42px minimum height, 10px radius, icon, label, trailing check.
- **Selected:** cyan wash, cyan icon, white label, visible check.
- **Purpose:** module shape, eye frame shape, eye dot shape. The icon is a miniature of the real geometry, not a generic square.

### Swatches
- **Shape:** 32px circles; the custom picker is a 32px circle with a small pipette badge.
- **Selected:** a 3.5px cyan ring with a 2px surface gap. The check mark uses `mix-blend-mode: difference` so it reads on black and white.
- **Disabled:** 40% opacity; the background row dims while transparency is on.

### Palette
- **Rows:** each colour role has a name, a contrast badge where the role is measured, a copyable hex value, a 32px custom picker, and the role's preset swatches.
- **Hex readout:** the value is a button. Activating it copies the hex and announces the result through the global live region. The background value is disabled while transparency is on.
- **Match action:** the eye frame and eye dot rows carry a 24px copy icon that applies the previous role's colour. Its accessible label names the source role, and it is disabled when the colours already match.

### Panels
- **Corner Style:** 16px.
- **Background:** #0a0a0a.
- **Border:** 1px rgba(255,255,255,0.1).
- **Shadow Strategy:** none.
- **Internal Padding:** 16px mobile, 20px desktop; headings tighten to 12px of top padding.

### Preview Stage
- **Shape:** square, 14px radius, 18px padding, #050505 fill, control-line border.
- **Update rhythm:** the code re-renders once, 120ms after the last keystroke. Style changes render immediately. The input never waits for the matrix.
- **Transparent:** a light checkerboard replaces the fill so black modules remain visible and the export's real transparency is obvious.
- **Motion:** a single 1050ms cyan scan pass after hydration. It is decorative, pointer-transparent, and removed after 1400ms. The code is static while the user types or tunes a colour. Under `prefers-reduced-motion` the pass is hidden.

### Status and Notices
- **Status pill:** 12px text with a matching icon; green for a passing contrast check, amber for a failure or an encoding error.
- **Notice:** a reserved 20px line under the export grid, 12px text, green for success and rose for failure, announced through the global live region. It fades and lifts 3px into place. Reserving the line prevents layout shift.
- **Quick-action bar:** fixed below 1024px, 61px tall plus the safe-area inset, with a 40px code thumbnail, version and byte count, and the current PNG resolution. It disappears while the preview tray is in view. The thumbnail is mounted only while the bar is visible, so the initial document carries one copy of the SVG.

## Do's and Don'ts

### Do:
- **Do** keep cyan for selection, focus, links, and the primary action. A screen should read as black, grey, and white until the user acts.
- **Do** show the measured contrast ratio next to each colour control, not only in the preview.
- **Do** keep the four-module quiet zone in the preview and every export. It is part of the rendered code, not padding around it.
- **Do** show the physical print size under the export size so a pixel count becomes a decision.
- **Do** use 30% white for control borders so inputs and buttons clear the 3:1 non-text contrast floor.
- **Do** use tabular numerals for any value that updates while the user types.
- **Do** keep the sensitive-payload summary free of Wi-Fi passwords and email bodies.
- **Do** let a user copy a hex readout by activating it, and confirm the copy through the existing notice line.
- **Do** keep an invalid coordinate pair or crypto amount out of the code. The preview stays empty until the form can encode the real value.

### Don't:
- **Don't** add glass, backdrop blur, gradients, or glow to panels; depth is tonal here.
- **Don't** fill inactive options or secondary buttons with cyan; it erases the primary-action hierarchy.
- **Don't** use Unicode glyphs or emoji as icons; the icon set is drawn at a 1.6px stroke on a 24-unit grid.
- **Don't** claim a code is scan-verified when only contrast was checked; say "Contrast passes" and ask the user to test-scan before printing.
- **Don't** make the preview sticky on short viewports; the export controls would be unreachable.
