# Style guide

The site belongs to the same family as the author's homepage, [mooofna.github.io](https://mooofna.github.io):
the same tokens, the same grid and the same restraint. This file collects the rules for design,
components and copy. CI enforces the character rules with `scripts/check_style.py`.

## Grid

- `.page` is `min(76rem, 100% - 2 gutters)` wide and centred. `.row` is a grid of 12 columns.
- A section is a row under a hairline (`--rule`). Its name sits in the label column (`.label`,
  columns 1 to 3); its content in the content column (`.content`, columns 4 to 12). Below 48rem
  both become one column.
- Running text is at most 38rem wide (`--measure`). Tables, figures and lists of links may use
  the full content column.
- Spacing comes from the scale `--space-1` to `--space-6`; no other values for gaps between blocks.

## Type

- Schibsted Grotesk for text; IBM Plex Mono (`.mono`) for tape cells, machine strings, standard
  format, configurations, numbers in tables and every live number. Numbers use tabular figures.
- Headings are plain and describe their content ("Configurations", not "Where the magic
  happens"). One h1 per page.
- Sizes come from `--text-xs` to `--text-display`. The display size is for the landing page only.

## Colour

- Ink, muted grey and hairlines carry the page. `--accent` (ETH blue, lighter in the dark theme)
  marks only the head of a machine, the active state and edge, the primary button, links on
  hover, the current navigation item and the focus ring.
- Every colour is a `light-dark()` token in `app.css`. The system setting picks the theme; the
  toggle in the header stores a choice in `data-theme` and local storage.
- Space-time diagrams are greyscale in the theme's background and ink, with the head in the
  accent. `lib/engine/palette.ts` reads the tokens, and diagrams recolour on a theme change
  without simulating again.

## Forbidden

Border radius, shadows, gradients, illustrations, emoji, glow, decorative icons, animated hero
effects and rounded cards. Use hairlines and square buttons (`.button`, `.button.primary`).

## Components (`web/src/lib/components`)

| Component                   | Use                                                                                       |
| --------------------------- | ----------------------------------------------------------------------------------------- |
| `PageHeader`                | h1, optional label (for example "Chapter 1") and lede at the top of a page                |
| `Section`                   | label column with the name, content column with the children                              |
| `Figure`                    | numbered figure; caption in the label column on wide screens, below on phones             |
| `ChapterNav`                | numbered list of the chapters (`variant="list"`) or previous and next (`variant="pager"`) |
| `Fact`                      | value from `content/facts.ts` in mono with its reference numbers                          |
| `Cite`                      | superscript reference numbers linking to the bibliography on the About page               |
| `Bibliography`              | numbered list of `content/sources.ts`                                                     |
| `Live`                      | number computed in the browser, muted while it is still changing                          |
| `Details`                   | collapsible block for definitions and proofs                                              |
| `Math`                      | MathML generated at build time with Temml                                                 |
| `SpaceTime` (`lib/machine`) | space-time diagram rendered by the engine worker                                          |

Place `Figure` between sections at page level. `sources.ts` exports `sources: Source[]`, and a
reference is numbered by its position in that list on every page. A citation of an unknown source
fails the build, because its link has no target.

The layout writes `<title>` and the meta description: chapter titles and summaries come from
`content/chapters.ts`. A page changes them by returning `title` and `description` from its load
function, not with its own `<title>`, which would add a second one.

Scrolling regions must be reachable by keyboard: wrap long lines in `pre` on phones or make the
region focusable (`Math` does this for wide formulas).

## Interaction

- One primary action per screen; everything else is a link or a secondary button.
- Definitions, proofs and long tables go into `Details`.
- Diagrams start when they scroll into view. Nothing moves under `prefers-reduced-motion`.
- Every control works with the keyboard and shows the focus ring. The skip link leads to `main`.
- At 375 px width no page scrolls sideways.

## Copy

- English, plain, specific and academic. Write what a thing is and does.
- No slogans, no tricolons for rhythm, no "not X, but Y", no rhetorical questions or one-line
  punchlines, no exclamation marks.
- No em dash, en dash or middle dot anywhere. Write ranges as "1 to 5" and use commas,
  semicolons or a new sentence instead of dashes.
- Every number is computed live in the browser or comes from the facts ledger with a source.
  Open problems are called open; "not reached in 10^5 iterations" is not "never halts".
- Numbers are grouped with commas: 47,176,870. Durations name their unit and the place they were
  measured ("41 ms in this browser").

## Privacy

No cookies, no analytics, no requests to other origins. Fonts are self-hosted with Fontsource.
The end-to-end tests fail on any request to another origin.
