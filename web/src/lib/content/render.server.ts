// The content pipeline: turns a chapter written in Markdown into segments of static HTML.
//
// It runs only in `+page.server.ts` at prerender time, so neither marked nor Temml ship to the
// browser. A chapter file has this form:
//
//   # Title
//
//   Lede paragraphs.
//
//   ## Section heading
//
//   Paragraphs with $inline$ and $$display$$ mathematics, values from the ledger {{fact:ID}},
//   numbers computed in the browser {{live:ID: description}} and citations [@source].
//
//   <<figure:fig1.1 | Fig. 1.1. Caption.>>
//
//   <details>
//   <summary>Proof</summary>
//
//   Markdown.
//
//   </details>
//
// The result is a title, a lede, a list of segments (a section per `##`, a figure per
// `<<figure:...>>`, and a continuation section for text that follows a figure) and the list of
// references cited on the page, numbered by first appearance. The build fails on unknown fact,
// source, live or figure ids, on unconfirmed facts used without their qualifying wording, on
// figure numbers out of sequence and on characters the house style forbids.

import { Marked } from 'marked';
import temml from 'temml';
import { chapters } from './chapters.ts';
import { factById, tableById, type LedgerFact, type LedgerTable } from './facts.ts';
import { liveIds } from './live.svelte.ts';
import { sourceById, type LedgerSource } from './sources.ts';
import type { Chapter } from './types.ts';

/** A section of a chapter: a `##` heading and its text, or text that continues after a figure. */
export interface SectionSegment {
  kind: 'section';
  /** The heading as plain text; null for the text that follows a figure within a section. */
  heading: string | null;
  /** Anchor id of the section; empty for a continuation. */
  id: string;
  html: string;
}

/** A figure: the component comes from `lib/figures/registry.ts`, the caption from the text. */
export interface FigureSegment {
  kind: 'figure';
  /** Registry id, for example `fig1.1`. */
  id: string;
  /** Figure number, for example `1.1`; null for an unnumbered figure. */
  number: string | null;
  /** Anchor id, for example `fig-1-1`. */
  anchor: string;
  /** Caption as HTML, without the leading "Fig. 1.1.". */
  caption: string;
}

export type Segment = SectionSegment | FigureSegment;

/** An entry of the list of sources at the end of a page. */
export interface Reference {
  n: number;
  id: string;
  /** The formatted entry as HTML. */
  html: string;
}

export interface RenderedChapter {
  title: string;
  /** The paragraphs before the first section, as HTML. */
  lede: string;
  segments: Segment[];
  references: Reference[];
}

export interface RenderOptions {
  /** Chapter number; figure captions must be numbered `Fig. <chapter>.<k>.` in order. */
  chapter?: number;
  /** Name of the file, for error messages. */
  file?: string;
  /** Link to a chapter, by number; when given, "chapter N" in the text becomes a link. */
  chapterHref?: (n: number) => string | undefined;
}

/** A content error. The message names the file and, where possible, the line. */
export class ContentError extends Error {
  override name = 'ContentError';
}

/**
 * Unconfirmed facts that may be shown, each with the reason and the wording that must appear in
 * the same paragraph or table row (see `src/content/research/REPORT.md`).
 */
export const ALLOWED_UNCONFIRMED: Readonly<Record<string, { reason: string; qualifier: RegExp }>> =
  {
    'zf-tm-254': {
      reason:
        'Announced on 13 October 2026 with a Coq formalisation that the fact check could not ' +
        'compile; the wiki still states 432 as the established bound. Report it as announced ' +
        'and awaiting review.',
      qualifier: /announced[\s\S]*awaiting review/i
    }
  };

/** Characters the house style forbids anywhere in prose. */
const FORBIDDEN: ReadonlyArray<[string, string]> = [
  ['\u2014', 'em dash'],
  ['\u2013', 'en dash'],
  ['\u00B7', 'middle dot']
];

// Placeholders survive marked unchanged: a private-use character, a kind, an index, another one.
const OPEN = '\uE000';
const CLOSE = '\uE001';
const PLACEHOLDER = /\uE000([A-Z])(\d+)\uE001/g;

const marked = new Marked({ gfm: true, breaks: false });

// ---------------------------------------------------------------------------------------------
// Small helpers

const escapeHtml = (text: string) =>
  text
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');

/** A slug for anchors: lower case, letters and digits, words joined by hyphens. */
export function slugify(text: string): string {
  return text
    .normalize('NFKD')
    .replace(/[\u0300-\u036F'\u2019]/g, '')
    .toLowerCase()
    .replace(/[\u03c3\u03c2]/g, 'sigma')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

/** Text without tags, for headings and labels. */
const stripTags = (html: string) => html.replace(/<[^>]*>/g, '');

/** Straight quotes to typographic quotes, in prose only (code and maths are protected). */
export function smartQuotes(text: string): string {
  return text
    .replace(/(^|[\s([{\u2018])"/gu, '$1\u201C')
    .replace(/"/g, '\u201D')
    .replace(/(\p{L}|\p{N}|[)\]])'(?=\p{L}|\s|$|[.,;:])/gu, '$1\u2019')
    .replace(/(^|[\s([{])'/gu, '$1\u2018')
    .replace(/'/g, '\u2019');
}

/** Superscripts for powers in ledger values: `10^42` becomes 10<sup>42</sup>. */
const powers = (escaped: string) => escaped.replace(/\^(\u2212?[\d,]+)/g, '<sup>$1</sup>');

/** A value is set in mono unless it is a date or a phrase. */
const isDate = (display: string) =>
  /^\d{1,2} (January|February|March|April|May|June|July|August|September|October|November|December) \d{4}$/.test(
    display
  );
const isPhrase = (display: string) => /\b[a-z]{3,}\b/.test(display);

function formatAuthors(authors: string): string {
  // "A, B, C and D": more than three names are shortened to the first and "et al.".
  const names = authors.split(/, | and /);
  return names.length > 3 ? `${names[0]} et al.` : authors;
}

/** Short form of a source for labels, for example "Radó 1962" or "Lin and Radó 1965". */
export function shortCitation(source: LedgerSource): string {
  const names = source.authors.split(/, | and /).map((n) => n.trim());
  const family = (name: string) =>
    name.startsWith('The ') || /contributors$/.test(name)
      ? name
      : (name
          .replace(/\s*\(.*\)$/, '')
          .split(/\s+/)
          .at(-1) ?? name);
  const who =
    names.length === 1
      ? family(names[0] ?? '')
      : names.length === 2
        ? `${family(names[0] ?? '')} and ${family(names[1] ?? '')}`
        : `${family(names[0] ?? '')} et al.`;
  return `${who} ${source.year}`;
}

/** The formatted bibliography entry of a source, as HTML. */
export function formatSource(source: LedgerSource): string {
  const link = source.url ?? (source.doi ? `https://doi.org/${source.doi}` : undefined);
  const title = escapeHtml(source.title);
  const parts: string[] = [];
  const authors = escapeHtml(formatAuthors(source.authors));
  parts.push(authors.endsWith('.') ? authors : `${authors}.`);
  parts.push(`${link ? `<a href="${escapeHtml(link)}">${title}</a>` : title}.`);
  let venue = escapeHtml(source.venue);
  if (source.volume) venue += ` ${escapeHtml(source.volume)}`;
  if (source.issue) venue += `(${escapeHtml(source.issue)})`;
  if (source.pages && /^\d+-\d+$/.test(source.pages)) venue += `, ${source.pages}`;
  parts.push(venue.includes(String(source.year)) ? `${venue}.` : `${venue}, ${source.year}.`);
  const ids: string[] = [];
  if (source.doi) ids.push(`doi:${escapeHtml(source.doi)}`);
  if (source.arxiv && !source.venue.includes(`arXiv:${source.arxiv}`)) {
    ids.push(`arXiv:${escapeHtml(source.arxiv)}`);
  }
  if (ids.length > 0) parts.push(`<span class="ref-id mono">${ids.join(' ')}</span>`);
  if (!source.confirmed) {
    parts.push('<span class="ref-note">Details not checked against the original.</span>');
  }
  return parts.join(' ');
}

// ---------------------------------------------------------------------------------------------
// The renderer

interface Context {
  file: string;
  options: RenderOptions;
  /** Rendered HTML of each placeholder, by kind and index. */
  stash: string[];
  /** Source id to reference number, in order of first appearance. */
  numbers: Map<string, number>;
  /** Figure ids on this page, to link "Fig. 1.2" in the text. */
  figures: Set<string>;
}

function fail(ctx: Context, message: string, line?: number): never {
  throw new ContentError(`${ctx.file}${line ? `:${line}` : ''}: ${message}`);
}

function stash(ctx: Context, kind: string, html: string): string {
  ctx.stash.push(html);
  return `${OPEN}${kind}${ctx.stash.length - 1}${CLOSE}`;
}

function cite(ctx: Context, ids: readonly string[], line?: number): string {
  const links = ids.map((id) => {
    const source = sourceById(id);
    if (!source) fail(ctx, `unknown source [@${id}]`, line);
    let n = ctx.numbers.get(id);
    if (n === undefined) {
      n = ctx.numbers.size + 1;
      ctx.numbers.set(id, n);
    }
    const label = `Source ${n}: ${shortCitation(source)}`;
    return `<a href="#ref-${n}" aria-label="${escapeHtml(label)}" title="${escapeHtml(label)}">${n}</a>`;
  });
  return `<sup class="cite">${links.join('<span class="sep">,</span>')}</sup>`;
}

function renderMath(ctx: Context, tex: string, display: boolean, line?: number): string {
  try {
    const mathml = temml.renderToString(tex, { displayMode: display, throwOnError: true });
    return display ? `<div class="math-display">${mathml}</div>` : mathml;
  } catch (error) {
    fail(ctx, `cannot render $${tex}$: ${(error as Error).message}`, line);
  }
}

function factHtml(fact: LedgerFact): string {
  // Values in up-arrow notation are set as mathematics, like the arrows in the text around them.
  if (/^[0-9\u2191 ]+$/.test(fact.display) && fact.display.includes('\u2191')) {
    const tex = fact.display.replaceAll('\u2191', ' \\uparrow ');
    const mathml = temml.renderToString(tex, { throwOnError: true });
    return `<span class="fact short" data-fact="${escapeHtml(fact.id)}">${mathml}</span>`;
  }
  const text = powers(escapeHtml(fact.display));
  const mono = !isDate(fact.display) && !isPhrase(fact.display);
  // Short values stay on one line; long ones (lists, phrases) may wrap.
  const classes = [
    'fact',
    ...(mono ? ['mono'] : []),
    ...(fact.display.length <= 28 ? ['short'] : [])
  ];
  return `<span class="${classes.join(' ')}" data-fact="${escapeHtml(fact.id)}">${text}</span>`;
}

function tableHtml(ctx: Context, table: LedgerTable, line?: number): string {
  const head = table.columns
    .map((c, i) => `<th scope="col"${i === 0 ? ' class="num"' : ''}>${escapeHtml(c)}</th>`)
    .join('');
  const rows = table.rows.map((row) => {
    const fact = factById(row.fact);
    if (!fact) fail(ctx, `table ${table.id} uses unknown fact ${row.fact}`, line);
    checkConfirmed(ctx, fact, row.cells.join(' '), line);
    // The citation goes into the last cell: directly after the number it would read as a power.
    const source = fact.source[0] ?? '';
    const cells = row.cells
      .map((c, i) => {
        const text = escapeHtml(smartQuotes(c));
        return `<td>${i === row.cells.length - 1 ? text + cite(ctx, [source], line) : text}</td>`;
      })
      .join('');
    return `<tr><td class="num">${factHtml(fact)}</td>${cells}</tr>`;
  });
  return (
    `<div class="table-wrap"><table class="table">` +
    `<caption>${escapeHtml(smartQuotes(table.caption))}</caption>` +
    `<thead><tr>${head}</tr></thead><tbody>${rows.join('')}</tbody></table></div>`
  );
}

function checkConfirmed(ctx: Context, fact: LedgerFact, block: string, line?: number) {
  if (fact.confirmed) return;
  const allowed = ALLOWED_UNCONFIRMED[fact.id];
  if (!allowed) {
    fail(ctx, `fact ${fact.id} is not confirmed and not in ALLOWED_UNCONFIRMED`, line);
  }
  if (!allowed.qualifier.test(block)) {
    fail(
      ctx,
      `fact ${fact.id} is not confirmed; its paragraph must match ${allowed.qualifier}`,
      line
    );
  }
}

/** Line number of `offset` within `text`, counting from `first`. */
const lineAt = (text: string, offset: number, first: number) =>
  first + (text.slice(0, offset).match(/\n/g)?.length ?? 0);

/**
 * Replaces code, maths, facts, live values and citations in one block of Markdown by
 * placeholders, in document order, so that citations are numbered by first appearance.
 */
/**
 * Where the citation of a fact goes: after the clause that contains the value, that is before
 * the next comma, semicolon, colon or full stop outside maths and code, before the end of a
 * table cell, or at the end of the block. A citation directly after a number would read as an
 * exponent.
 */
function citeTarget(text: string, from: number): number {
  let i = from;
  const skip = (open: string, close: string) => {
    if (!text.startsWith(open, i)) return false;
    const end = text.indexOf(close, i + open.length);
    i = end < 0 ? text.length : end + close.length;
    return true;
  };
  while (i < text.length) {
    if (
      skip('$$', '$$') ||
      skip('$', '$') ||
      skip('`', '`') ||
      skip('{{', '}}') ||
      skip('[@', ']')
    ) {
      continue;
    }
    const char = text[i] ?? '';
    const next = text[i + 1] ?? '';
    if (char === '|' || char === '\n') break;
    if (/[.,;:]/.test(char) && (next === '' || /[ \n]/.test(next))) break;
    i += 1;
  }
  while (i > from && /[ ]/.test(text[i - 1] ?? '')) i -= 1;
  return i;
}

/**
 * In a table row, the citation of a fact joins the last citation of the row, which names the
 * sources of the row, or else goes to the end of the last cell. Undefined outside tables.
 */
function rowCiteTarget(text: string): number | undefined {
  if (!text.trimStart().startsWith('|')) return undefined;
  const last = text.lastIndexOf('[@');
  if (last >= 0) return text.indexOf(']', last) + 1;
  let end = text.trimEnd().lastIndexOf('|');
  while (end > 0 && text[end - 1] === ' ') end -= 1;
  return end;
}

/**
 * Gives every fact a citation of its first source, unless the block already cites one of its
 * sources: the citation is inserted as `[@id]` after the clause that contains the fact, or in
 * the last cell of a table row.
 */
function citeFacts(text: string): string {
  const cited = new Set([...text.matchAll(/\[@([^\]\s]+)\]/g)].map((m) => m[1] ?? ''));
  const insertions: { at: number; id: string }[] = [];
  for (const match of text.matchAll(/\{\{fact:([^}\s]+)\}\}/g)) {
    const fact = factById(match[1] ?? '');
    if (!fact || fact.source.some((s) => cited.has(s))) continue;
    const source = fact.source[0] ?? '';
    cited.add(source);
    const at = rowCiteTarget(text) ?? citeTarget(text, (match.index ?? 0) + match[0].length);
    insertions.push({ at, id: source });
  }
  let result = text;
  for (const { at, id } of insertions.reverse()) {
    result = `${result.slice(0, at)} [@${id}]${result.slice(at)}`;
  }
  return result;
}

/**
 * Replaces code, maths, facts, live values and citations in one block of Markdown by
 * placeholders, in document order, so that citations are numbered by first appearance.
 * Citations follow a comma, semicolon, colon or full stop that comes directly after them.
 */
function protect(ctx: Context, raw: string, firstLine: number): string {
  const text = citeFacts(raw);
  const pattern =
    /(`+)([\s\S]*?)\1|\$\$([\s\S]+?)\$\$|\$((?:\\\$|[^$\n])+?)\$|\{\{fact:([^}\s]+)\}\}|\{\{live:([a-z0-9-]+)(?::\s*([^}]*))?\}\}|\{\{([^}]*)\}\}|\s*((?:\[@[^\]\s]+\])(?:\s*\[@[^\]\s]+\])*)([.,;:](?=\s|$))?/g;
  return text.replace(
    pattern,
    (
      match,
      ticks,
      code,
      display,
      inline,
      factId,
      liveId,
      liveText,
      other,
      cites,
      punct,
      offset
    ) => {
      const line = lineAt(text, offset as number, firstLine);
      if (ticks !== undefined) return stash(ctx, 'C', match);
      if (display !== undefined) return stash(ctx, 'D', renderMath(ctx, display, true, line));
      if (inline !== undefined) return stash(ctx, 'M', renderMath(ctx, inline, false, line));
      if (factId !== undefined) {
        const table = tableById(factId);
        if (table) return stash(ctx, 'T', tableHtml(ctx, table, line));
        const fact = factById(factId);
        if (!fact) fail(ctx, `unknown fact {{fact:${factId}}}`, line);
        checkConfirmed(ctx, fact, raw, line);
        for (const [char, name] of FORBIDDEN) {
          if (fact.display.includes(char)) fail(ctx, `fact ${fact.id} contains an ${name}`, line);
        }
        return stash(ctx, 'F', factHtml(fact));
      }
      if (liveId !== undefined) {
        if (!Object.hasOwn(liveIds, liveId)) fail(ctx, `unknown live value ${liveId}`, line);
        const description =
          (liveText as string | undefined)?.trim() || liveIds[liveId as keyof typeof liveIds];
        const label = `${description}, computed in your browser`;
        return stash(
          ctx,
          'L',
          `<span class="live mono" data-live="${liveId}" data-state="pending" title="${escapeHtml(label)}">` +
            `<span class="live-pending" aria-hidden="true">\u2026</span>` +
            `<span class="visually-hidden">${escapeHtml(label)}</span></span>`
        );
      }
      if (other !== undefined) fail(ctx, `unknown directive {{${other}}}`, line);
      const ids = [...(cites as string).matchAll(/\[@([^\]\s]+)\]/g)].map((m) => m[1] ?? '');
      return (punct ?? '') + stash(ctx, 'R', cite(ctx, ids, line));
    }
  );
}

/** Puts the rendered HTML back in place of the placeholders. Code is restored before marked. */
function restore(ctx: Context, html: string): string {
  // Display maths and tables are blocks: a paragraph around them is split.
  const block = /(\uE000[DT]\d+\uE001)/;
  let result = html.replace(/<p>([\s\S]*?)<\/p>/g, (match, inner: string) => {
    if (!block.test(inner)) return match;
    return inner
      .split(block)
      .map((part) => part.trim())
      .filter((part) => part !== '')
      .map((part) => (block.test(part) ? part : `<p>${part}</p>`))
      .join('\n');
  });
  // Placeholders can contain placeholders (a table contains facts and citations).
  for (let i = 0; i < 3 && result.includes(OPEN); i++) {
    result = result.replace(PLACEHOLDER, (_, _kind, index) => ctx.stash[Number(index)] ?? '');
  }
  return result;
}

/** Restores code spans as Markdown so that marked renders them. */
const restoreCode = (ctx: Context, text: string) =>
  text.replace(PLACEHOLDER, (match, kind, index) =>
    kind === 'C' ? (ctx.stash[Number(index)] ?? '') : match
  );

/** Typographic touches on prose: quotes, and no line break inside "Fig. 1.1" or "chapter 4". */
function typeset(text: string): string {
  return smartQuotes(text)
    .replace(/\bFig\. (\d)/g, 'Fig.\u00A0$1')
    .replace(/\b([Cc]hapters?) (\d)/g, '$1\u00A0$2');
}

/** Links "Fig. 1.2" to the figure on this page and "chapter 4" to that chapter. */
function crossLinks(ctx: Context, html: string): string {
  // Only in text between tags, never inside an attribute or an existing link.
  return html.replace(/(<a\b[\s\S]*?<\/a>)|(<[^>]+>)|([^<]+)/g, (match, link, tag, text) => {
    if (link || tag) return match;
    let out = (text as string).replace(/\bFig\.\u00A0(\d+)\.(\d+)\b/g, (m, a, b) =>
      ctx.figures.has(`fig${a}.${b}`) ? `<a href="#fig-${a}-${b}">${m}</a>` : m
    );
    const href = ctx.options.chapterHref;
    if (href) {
      out = out.replace(/\b([Cc]hapter)\u00A0(\d)\b/g, (m, _word, n) => {
        const number = Number(n);
        const target = number === ctx.options.chapter ? undefined : href(number);
        return target ? `<a href="${escapeHtml(target)}">${m}</a>` : m;
      });
    }
    return out;
  });
}

/** Tables from Markdown: the site's table style, a scroll container, numeric cells in mono. */
function styleTables(html: string): string {
  return html
    .replace(/<table>/g, '<div class="table-wrap"><table class="table">')
    .replace(/<\/table>/g, '</table></div>')
    .replace(/<(td|th)( align="\w+")?>([\s\S]*?)<\/\1>/g, (match, tag, _align, body: string) => {
      const bare = body.replace(/\uE000[FMR]\d+\uE001/g, '').replace(/<[^>]*>/g, '');
      const numeric = /^[\s\d,.]*$/.test(bare) && body.trim() !== '';
      const scope = tag === 'th' ? ' scope="col"' : '';
      return `<${tag}${scope}${numeric ? ' class="num"' : ''}>${body}</${tag}>`;
    });
}

/** Renders one block of Markdown (a section body or the lede) to HTML. */
function renderBlock(ctx: Context, text: string, firstLine: number): string {
  // Split into paragraphs, list items and table rows, which are the scope of a citation.
  const blocks = text.split(/(\n{2,}|\n(?=\s*(?:[-*] |\d+\. |\|)))/);
  let offsetLine = firstLine;
  const parts = blocks.map((block) => {
    const protectedText = protect(ctx, block, offsetLine);
    offsetLine += block.match(/\n/g)?.length ?? 0;
    return protectedText;
  });
  const markdown = restoreCode(ctx, typeset(parts.join('')));
  const html = styleTables(marked.parse(markdown, { async: false }));
  return finish(crossLinks(ctx, restore(ctx, html))).trim();
}

/**
 * Last touches on the HTML: a table header that is only a formula gets a text equivalent, an
 * inline formula stays on one line with the punctuation that follows it, and a machine in the
 * text format may break after each underscore.
 */
function finish(html: string): string {
  return html
    .replace(
      /<th([^>]*)><math>((?:(?!<\/math>)[\s\S])*)<\/math><\/th>/g,
      (_, attrs: string, inner: string) =>
        `<th${attrs}><math aria-hidden="true">${inner}</math>` +
        `<span class="visually-hidden">${stripTags(inner)}</span></th>`
    )
    .replace(/(<math>(?:(?!<\/math>)[\s\S])*<\/math>)([.,;:)]+)/g, '<span class="nobr">$1$2</span>')
    .replace(/<code>([^<]*_[^<]*)<\/code>/g, (_, code: string) =>
      code
        .split(/(?<=_)/)
        .map((part) => `<code>${part}</code>`)
        .join('<wbr>')
    );
}

function renderInline(ctx: Context, text: string, line: number): string {
  const protectedText = restoreCode(ctx, typeset(protect(ctx, text, line)));
  return finish(crossLinks(ctx, restore(ctx, marked.parseInline(protectedText, { async: false }))));
}

/** Checks the house style on the raw text: forbidden characters, "!" and "?" in prose. */
function checkStyle(ctx: Context, markdown: string) {
  const lines = markdown.split('\n');
  let fence = false;
  lines.forEach((raw, i) => {
    if (/^\s*```/.test(raw)) fence = !fence;
    for (const [char, name] of FORBIDDEN) {
      if (raw.includes(char))
        fail(
          ctx,
          `${name} (U+${char.charCodeAt(0).toString(16).toUpperCase().padStart(4, '0')})`,
          i + 1
        );
    }
    if (fence) return;
    const prose = raw
      .replace(/`[^`]*`/g, '')
      .replace(/\$\$[\s\S]*?\$\$|\$[^$]*\$/g, '')
      .replace(/\{\{[^}]*\}\}/g, '')
      .replace(/<!--[\s\S]*?-->/g, '')
      .replace(/\]\([^)]*\)/g, ']');
    if (/[!?]/.test(prose)) fail(ctx, 'exclamation or question mark in prose', i + 1);
  });
}

/** Renders a chapter file. Throws a ContentError on any problem. */
export function renderChapter(markdown: string, options: RenderOptions = {}): RenderedChapter {
  const ctx: Context = {
    file: options.file ?? 'chapter',
    options,
    stash: [],
    numbers: new Map(),
    figures: new Set()
  };
  const source = markdown.replace(/\r\n?/g, '\n');
  checkStyle(ctx, source);

  const titleMatch = /^# (.+)\n/.exec(source);
  if (!titleMatch) fail(ctx, 'the file must start with "# Title"', 1);
  const title = titleMatch[1]?.trim() ?? '';

  // Split the rest into the lede, sections and figures, keeping line numbers.
  const lines = source.split('\n');
  type Chunk =
    | { kind: 'text'; heading: string | null; lines: string[]; first: number }
    | { kind: 'figure'; id: string; caption: string; line: number };
  const chunks: Chunk[] = [];
  let current: Extract<Chunk, { kind: 'text' }> = {
    kind: 'text',
    heading: null,
    lines: [],
    first: 2
  };
  let lede: Extract<Chunk, { kind: 'text' }> | null = current;
  let fence = false;
  for (let i = 1; i < lines.length; i++) {
    const line = lines[i] ?? '';
    if (/^\s*```/.test(line)) fence = !fence;
    const heading = fence ? null : /^## (.+)$/.exec(line);
    const figure = fence ? null : /^<<figure:([\w.-]+)\s*\|\s*([\s\S]+)>>\s*$/.exec(line);
    if (heading) {
      chunks.push(current);
      current = { kind: 'text', heading: heading[1]?.trim() ?? '', lines: [], first: i + 2 };
    } else if (figure) {
      chunks.push(current);
      chunks.push({
        kind: 'figure',
        id: figure[1] ?? '',
        caption: figure[2]?.trim() ?? '',
        line: i + 1
      });
      current = { kind: 'text', heading: null, lines: [], first: i + 2 };
    } else if (/^#{1,2} /.test(line) && !fence) {
      fail(ctx, 'only one "#" title and "##" sections are allowed', i + 1);
    } else if (/<<figure:/.test(line)) {
      fail(ctx, 'a figure must stand on a line of its own: <<figure:ID | caption>>', i + 1);
    } else {
      current.lines.push(line);
    }
  }
  chunks.push(current);

  for (const chunk of chunks) if (chunk.kind === 'figure') ctx.figures.add(chunk.id);

  let ledeHtml = '';
  const segments: Segment[] = [];
  const anchors = new Set<string>();
  let figureCount = 0;
  for (const chunk of chunks) {
    if (chunk.kind === 'figure') {
      let caption = chunk.caption;
      let number: string | null = null;
      const numbered = /^Fig\. (\d+)\.(\d+)\.\s+/.exec(caption);
      if (numbered) {
        figureCount += 1;
        number = `${numbered[1]}.${numbered[2]}`;
        if (options.chapter !== undefined) {
          const expected = `${options.chapter}.${figureCount}`;
          if (number !== expected) {
            fail(ctx, `figure ${chunk.id} is numbered ${number}, expected ${expected}`, chunk.line);
          }
        }
        if (chunk.id !== `fig${number}`) {
          fail(ctx, `figure id ${chunk.id} does not match its number ${number}`, chunk.line);
        }
        caption = caption.slice(numbered[0].length);
      }
      segments.push({
        kind: 'figure',
        id: chunk.id,
        number,
        anchor: number ? `fig-${number.replace('.', '-')}` : `fig-${slugify(chunk.id)}`,
        caption: renderInline(ctx, caption, chunk.line)
      });
      continue;
    }
    const body = chunk.lines.join('\n').trim();
    if (chunk === lede) {
      ledeHtml = body ? renderBlock(ctx, body, chunk.first) : '';
      continue;
    }
    if (chunk.heading === null && body === '') continue;
    const html = renderBlock(ctx, body, chunk.first);
    if (chunk.heading === null) {
      segments.push({ kind: 'section', heading: null, id: '', html });
      continue;
    }
    const heading = stripTags(renderInline(ctx, chunk.heading, chunk.first - 2));
    let id = slugify(heading) || 'section';
    while (anchors.has(id)) id += '-2';
    anchors.add(id);
    segments.push({ kind: 'section', heading, id, html });
  }
  lede = null;

  const references: Reference[] = [...ctx.numbers.entries()].map(([id, n]) => {
    const source = sourceById(id);
    if (!source) fail(ctx, `unknown source ${id}`);
    return { n, id, html: formatSource(source) };
  });

  return { title, lede: ledeHtml, segments, references };
}

// ---------------------------------------------------------------------------------------------
// Loading the chapter files

const files = import.meta.glob<string>('/src/content/0[1-6]-*.md', {
  query: '?raw',
  import: 'default',
  eager: true
});

/** The data of a chapter page: its entry in the chapter list and its rendered content. */
export function loadChapter(slug: string, href: (chapter: Chapter) => string) {
  const chapter = chapters.find((c) => c.slug === slug);
  if (!chapter) throw new ContentError(`no chapter with the slug ${slug}`);
  const name = `0${chapter.n}-${slug}.md`;
  const markdown = files[`/src/content/${name}`];
  if (markdown === undefined) throw new ContentError(`missing file src/content/${name}`);
  const content = renderChapter(markdown, {
    chapter: chapter.n,
    file: name,
    chapterHref: (n) => {
      const target = chapters.find((c) => c.n === n);
      return target ? href(target) : undefined;
    }
  });
  if (content.title !== chapter.title) {
    throw new ContentError(`${name}: title "${content.title}" differs from chapters.ts`);
  }
  return { chapter, content };
}
