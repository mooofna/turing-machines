import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { chapters } from './chapters.ts';
import { facts, tables } from './facts.ts';
import {
  ALLOWED_UNCONFIRMED,
  ContentError,
  formatSource,
  renderChapter,
  shortCitation,
  slugify,
  smartQuotes,
  type FigureSegment,
  type SectionSegment
} from './render.server.ts';
import { sourceById, sources } from './sources.ts';
import { figures } from '../figures/registry.ts';

const render = (body: string, options = {}) => renderChapter(`# Title\n\n${body}\n`, options);
const sections = (body: string) =>
  render(body).segments.filter((s): s is SectionSegment => s.kind === 'section');

describe('structure', () => {
  it('reads the title, the lede and one section per ##', () => {
    const result = render('Lede text.\n\n## First part\n\nOne.\n\n## Second part\n\nTwo.');
    expect(result.title).toBe('Title');
    expect(result.lede).toBe('<p>Lede text.</p>');
    expect(result.segments).toEqual([
      { kind: 'section', heading: 'First part', id: 'first-part', html: '<p>One.</p>' },
      { kind: 'section', heading: 'Second part', id: 'second-part', html: '<p>Two.</p>' }
    ]);
  });

  it('requires a title', () => {
    expect(() => renderChapter('No title\n')).toThrow(ContentError);
  });

  it('turns a figure line into a figure segment and continues the section after it', () => {
    const result = render(
      '## Part\n\nBefore.\n\n<<figure:fig1.1 | Fig. 1.1. A caption with $x$.>>\n\nAfter.',
      { chapter: 1 }
    );
    const [first, figure, after] = result.segments;
    expect(first).toMatchObject({ kind: 'section', heading: 'Part', html: '<p>Before.</p>' });
    expect(figure).toMatchObject({
      kind: 'figure',
      id: 'fig1.1',
      number: '1.1',
      anchor: 'fig-1-1'
    });
    expect((figure as FigureSegment).caption).toBe(
      'A caption with <span class="nobr"><math><mi>x</mi></math>.</span>'
    );
    expect(after).toEqual({ kind: 'section', heading: null, id: '', html: '<p>After.</p>' });
  });

  it('rejects figures numbered out of sequence or with a mismatched id', () => {
    expect(() => render('<<figure:fig1.2 | Fig. 1.2. Caption.>>', { chapter: 1 })).toThrow(
      /expected 1.1/
    );
    expect(() => render('<<figure:fig1.2 | Fig. 1.1. Caption.>>', { chapter: 1 })).toThrow(
      /does not match/
    );
  });

  it('keeps details blocks and renders Markdown inside them', () => {
    const [section] = sections(
      '## Part\n\n<details>\n<summary>Proof</summary>\n\nSome *text* and $x$.\n\n</details>'
    );
    expect(section?.html).toMatch(/^<details>\n<summary>Proof<\/summary><p>Some <em>text<\/em>/);
    expect(section?.html).toMatch(/<\/details>$/);
  });

  it('gives sections unique, readable anchors', () => {
    const result = sections("## Rice's theorem\n\nA.\n\n## Rice's theorem\n\nB.");
    expect(result.map((s) => s.id)).toEqual(['rices-theorem', 'rices-theorem-2']);
    expect(result[0]?.heading).toBe('Rice\u2019s theorem');
  });
});

describe('mathematics', () => {
  it('renders inline maths as MathML on the text line', () => {
    const [section] = sections('## M\n\nLet $q_0$ be the start state.');
    expect(section?.html).toMatch(/^<p>Let <math><msub><mi>q<\/mi><mn>0<\/mn><\/msub><\/math> be/);
  });

  it('renders display maths in a scroll container', () => {
    const [section] = sections('## M\n\n$$a \\vdash b$$');
    expect(section?.html).toMatch(/^<div class="math-display"><math display="block"/);
  });

  it('supports every command the chapters use', () => {
    const commands = [
      '\\text{\u2423}',
      '\\rightharpoonup',
      '\\tfrac12',
      'n \\bmod 2',
      '\\square',
      '\\vdash^*',
      '\\mathrm{enc}(M)\\#\\#\\mathrm{enc}(w)',
      '\\mathbf{P}_{\\mathrm{Halt}}',
      '\\lfloor 3b/2 \\rfloor',
      'a \\uparrow^{k} b',
      '\\bigl(4(n+1)\\bigr)^{2n}',
      '\\le_m',
      '\\langle Q, \\Sigma \\rangle'
    ];
    for (const tex of commands) {
      expect(() => sections(`## M\n\n$${tex}$`), tex).not.toThrow();
    }
  });

  it('keeps a formula on one line with the punctuation after it', () => {
    const [section] = sections('## M\n\nLet $a$ and $b$. Then $c$, done.');
    expect(section?.html).toBe(
      '<p>Let <math><mi>a</mi></math> and <span class="nobr"><math><mi>b</mi></math>.</span> ' +
        'Then <span class="nobr"><math><mi>c</mi></math>,</span> done.</p>'
    );
  });

  it('fails on unknown commands', () => {
    expect(() => sections('## M\n\n$\\nosuchcommand$')).toThrow(/cannot render/);
  });

  it('leaves code alone', () => {
    const [section] = sections('## C\n\nThe string `1RB$1` and `{{fact:x}}` stay literal.');
    expect(section?.html).toContain('<code>1RB$1</code>');
    expect(section?.html).toContain('<code>{{fact:x}}</code>');
  });
});

describe('facts', () => {
  it('replaces a fact by its display value and cites its first source after the clause', () => {
    const [section] = sections('## F\n\nIt halts after {{fact:bb5-steps}} steps, which is a lot.');
    expect(section?.html).toBe(
      '<p>It halts after <span class="fact mono short" data-fact="bb5-steps">47,176,870</span> steps,' +
        '<sup class="cite"><a href="#ref-1" aria-label="Source 1: bbchallenge wiki contributors 2026" ' +
        'title="Source 1: bbchallenge wiki contributors 2026">1</a></sup> which is a lot.</p>'
    );
  });

  it('cites a fact in a table row in the last cell of the row', () => {
    const [section] = sections(
      '## F\n\n| $n$ | $S(n)$ | By |\n|---|---|---|\n| 5 | {{fact:bb5-steps}} | Them |'
    );
    expect(section?.html).toMatch(/<td>Them<sup class="cite">/);
  });

  it('cites a fact in a table at the end of its cell', () => {
    const [section] = sections('## F\n\n| $n$ | $S(n)$ |\n|---|---|\n| 5 | {{fact:bb5-steps}} |');
    expect(section?.html).toMatch(
      /<td class="num"><span class="fact mono short" data-fact="bb5-steps">47,176,870<\/span><sup class="cite">/
    );
  });

  it('does not cite a fact again when its paragraph cites one of its sources', () => {
    const result = render('## F\n\nIt halts after {{fact:bb5-steps}} steps [@coqbb5].');
    expect(result.references.map((r) => r.id)).toEqual(['coqbb5']);
    expect(result.segments[0]).toMatchObject({
      html: expect.stringContaining('</span> steps.<sup class="cite">')
    });
  });

  it('sets dates in the text face and powers as superscripts', () => {
    const [section] = sections(
      '## F\n\nOn {{fact:bb5-announcement-date}}, below {{fact:antihydra-halting-probability}}.'
    );
    expect(section?.html).toContain('<span class="fact short" data-fact="bb5-announcement-date">');
    expect(section?.html).toContain('less than 3 \u00d7 10<sup>\u221228,723,042,565</sup>');
  });

  it('fails on unknown facts and unknown directives', () => {
    expect(() => sections('## F\n\n{{fact:no-such-fact}}')).toThrow(/unknown fact/);
    expect(() => sections('## F\n\n{{bibliography}}')).toThrow(/unknown directive/);
  });

  it('allows an unconfirmed fact only with its qualifying wording', () => {
    expect(ALLOWED_UNCONFIRMED['zf-tm-254']).toBeDefined();
    expect(() => sections('## Z\n\nA machine with {{fact:zf-tm-254}} states exists.')).toThrow(
      /not confirmed/
    );
    expect(() =>
      sections(
        '## Z\n\nA machine with {{fact:zf-tm-254}} states was announced and is awaiting review.'
      )
    ).not.toThrow();
    expect(() => sections('## Z\n\nThe {{fact:tnf-origin}}.')).toThrow(/ALLOWED_UNCONFIRMED/);
  });

  it('renders a ledger table with a citation per row', () => {
    const [section] = sections('## Z\n\n{{fact:zf-history}}');
    const table = tables['zf-history'];
    expect(section?.html).toContain('<table class="table">');
    expect(section?.html.match(/<tr>/g)).toHaveLength(table.rows.length + 1);
    expect(section?.html).toContain('awaiting review');
  });
});

describe('citations', () => {
  it('numbers sources by first appearance and merges adjacent citations', () => {
    const result = render(
      '## C\n\nFirst [@turing1936] [@rado1962]. Again [@rado1962].\n\n## D\n\nThird [@rice1953].'
    );
    expect(result.references.map((r) => [r.n, r.id])).toEqual([
      [1, 'turing1936'],
      [2, 'rado1962'],
      [3, 'rice1953']
    ]);
    const html = (result.segments[0] as SectionSegment).html;
    expect(html).toMatch(
      /^<p>First\.<sup class="cite"><a href="#ref-1"[^>]*>1<\/a><span class="sep">,<\/span><a href="#ref-2"/
    );
    expect(html).toMatch(/Again\.<sup class="cite"><a href="#ref-2"/);
  });

  it('fails on unknown sources', () => {
    expect(() => sections('## C\n\nText [@nobody2099].')).toThrow(/unknown source/);
  });

  it('formats references with authors, linked title, venue and identifiers', () => {
    const rado = sourceById('rado1962');
    expect(rado).toBeDefined();
    if (!rado) return;
    expect(formatSource(rado)).toBe(
      'Tibor Radó. <a href="https://archive.org/details/bstj41-3-877">On Non-Computable Functions</a>. ' +
        'Bell System Technical Journal 41(3), 877-884, 1962. ' +
        '<span class="ref-id mono">doi:10.1002/j.1538-7305.1962.tb00480.x</span>'
    );
    expect(shortCitation(rado)).toBe('Radó 1962');
    const lin = sourceById('linrado1965');
    if (lin) expect(shortCitation(lin)).toBe('Lin and Radó 1965');
  });

  it('marks sources whose details the fact check could not confirm', () => {
    const brady = sourceById('brady1964thesis');
    expect(brady && formatSource(brady)).toContain('not checked against the original');
  });
});

describe('live values', () => {
  it('leaves an accessible placeholder for the browser to fill', () => {
    const [section] = sections('## L\n\nIt is {{live:tower-4: value of 2\u2191\u21914}}.');
    expect(section?.html).toContain('data-live="tower-4" data-state="pending"');
    expect(section?.html).toContain('<span class="live-pending" aria-hidden="true">\u2026</span>');
    expect(section?.html).toContain(
      '<span class="visually-hidden">value of 2\u2191\u21914, computed in your browser</span>'
    );
  });

  it('fails on unknown live ids', () => {
    expect(() => sections('## L\n\n{{live:nothing}}')).toThrow(/unknown live value/);
  });
});

describe('house style', () => {
  it('fails on em dashes, en dashes and middle dots', () => {
    for (const char of ['\u2014', '\u2013', '\u00b7']) {
      expect(() => sections(`## S\n\nA ${char} B.`)).toThrow(ContentError);
    }
  });

  it('fails on exclamation and question marks in prose, not in maths or code', () => {
    expect(() => sections('## S\n\nReally?')).toThrow(/question mark/);
    expect(() => sections('## S\n\nWow!')).toThrow(/exclamation/);
    expect(() => sections('## S\n\n$n!$ and `a?`')).not.toThrow();
  });

  it('sets typographic quotes and keeps figure and chapter numbers on one line', () => {
    expect(smartQuotes('"still running" and Turing\'s and O\'Rear')).toBe(
      '\u201cstill running\u201d and Turing\u2019s and O\u2019Rear'
    );
    const [section] = sections('## S\n\nSee Fig. 2.1 and chapter 3.');
    expect(section?.html).toBe('<p>See Fig.\u00a02.1 and chapter\u00a03.</p>');
  });

  it('links figures on the page and other chapters', () => {
    const result = render(
      '## S\n\nSee Fig. 1.1 and chapter 3, but not chapter 1.\n\n<<figure:fig1.1 | Fig. 1.1. C.>>',
      { chapter: 1, chapterHref: (n: number) => `/c${n}` }
    );
    const html = (result.segments[0] as SectionSegment).html;
    expect(html).toContain('<a href="#fig-1-1">Fig.\u00a01.1</a>');
    expect(html).toContain('<a href="/c3">chapter\u00a03</a>');
    expect(html).toContain('not chapter\u00a01.');
  });

  it('lets machines in the text format break after underscores', () => {
    const [section] = sections('## S\n\nThe machine `1RB1LB_1LA1RZ`.');
    expect(section?.html).toContain('<code>1RB1LB_</code><wbr><code>1LA1RZ</code>');
  });

  it('marks numeric table cells', () => {
    const [section] = sections('## T\n\n| $n$ | name |\n|---|---|\n| 12 | x |');
    expect(section?.html).toMatch(/<th scope="col" class="num"><math>/);
    expect(section?.html).toContain('<td class="num">12</td>');
    expect(section?.html).toContain('<div class="table-wrap"><table class="table">');
  });
});

describe('ledger', () => {
  it('cites only known sources in every fact', () => {
    for (const fact of Object.values(facts)) {
      expect(fact.source.length, fact.id).toBeGreaterThan(0);
      for (const id of fact.source) expect(sourceById(id), `${fact.id}: ${id}`).toBeDefined();
    }
  });

  it('has unique source ids', () => {
    expect(new Set(sources.map((s) => s.id)).size).toBe(sources.length);
  });
});

describe('the chapters', () => {
  const files = [
    '01-machine.md',
    '02-programs-as-data.md',
    '03-limits.md',
    '04-busy-beaver.md',
    '05-proving-bb5.md',
    '06-frontier.md'
  ];

  files.forEach((file, i) => {
    it(`renders ${file}`, () => {
      const markdown = readFileSync(new URL(`../../content/${file}`, import.meta.url), 'utf8');
      const result = renderChapter(markdown, { chapter: i + 1, file });
      expect(result.title).toBe(chapters[i]?.title);
      expect(result.lede.length).toBeGreaterThan(0);
      for (const segment of result.segments) {
        if (segment.kind === 'figure') expect(figures[segment.id], segment.id).toBeDefined();
      }
      expect(result.references.length).toBeGreaterThan(0);
    });
  });
});
