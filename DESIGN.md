# Vela Workbench design system

## Register and scene

This is a product interface. A researcher uses it on a wide desktop display in a well-lit office or laboratory, moving between source code and exact scientific state for long sessions. That scene calls for a light, cool-neutral canvas with high-contrast ink, compact controls, and restrained semantic color. The interface should feel native and quiet enough to remain open beside an editor.

## Visual direction

Use a restrained Vela product palette. The cool blue neutral family is copied and documented from the public Vela product tokens inspected on 2026-08-17. It is a design reference, not a runtime package dependency.

- Background: `oklch(0.973 0.006 245)`
- Raised surface: `oklch(0.985 0.005 245)`
- Primary ink: `oklch(0.21 0.032 265)`
- Primary control: `oklch(0.184 0.04 261)`
- Secondary surface: `oklch(0.945 0.01 245)`
- Muted ink: `oklch(0.522 0.021 258)`
- Rule: `oklch(0.891 0.006 264)`
- Focus: `oklch(0.235 0.082 258)`
- Destructive/refusal: `oklch(0.496 0.124 16)`

Semantic colors only communicate owned states such as clean/dirty source, supported/refused schema, or local/unavailable tool. They never imply scientific truth or acceptance by color alone.

## Typography

Use the system sans stack for controls and prose: `-apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif`. Use `SFMono-Regular, ui-monospace, monospace` for hashes, roots, paths, commands, and schema names. Keep the UI scale compact and fixed: 12, 13, 14, 16, 20, and 28 pixels. Data labels are short; explanatory prose stays below 72 characters per line where practical.

## Layout

The desktop frame has three stable regions:

1. A 248-pixel repository rail that owns selection and recents.
2. A compact source header that names the selected repository, commit, branch, dirty state, and Vela binary identity.
3. A main task surface with two tabs, Orient and Execute / Source.

At narrow widths the rail becomes an inline repository selector above the content. Do not turn every fact into a card. Use ruled sections, definition grids, tables, and list rows. A raised panel is reserved for a boundary-changing handoff or an actionable refusal.

## Components

- Buttons use one 7-pixel radius, visible focus, and clear default, hover, active, disabled, and loading states.
- Repository rows use a leading classification mark, name, path tail, branch, and dirty label. Selection uses a neutral fill plus weight, not a colored stripe.
- Status badges always include text and use restrained tints.
- Hashes and paths truncate in the middle or end visually but expose the full value through accessible text and copy affordances where appropriate.
- Loading uses row skeletons. Empty states explain the next safe action. Errors state the exact command boundary and keep diagnostics selectable.
- Tabs follow standard keyboard and focus behavior through app-owned Base UI compositions.

## Motion and accessibility

Transitions last 160 to 200 milliseconds with an ease-out curve and only clarify hover, selection, disclosure, or loading state. Respect reduced motion. All interactive targets are keyboard reachable, focus remains visible in forced colors, and color is never the sole status cue. Minimum target size is 32 pixels for dense desktop controls and 40 pixels for primary actions.

## Prohibited patterns

No gradient text, glass panels, oversized hero metrics, decorative charts, nested cards, colored side stripes, custom scrollbars, remote WebView content, or animated page-load sequence. The product should resemble a trusted native work surface, not a marketing dashboard.
