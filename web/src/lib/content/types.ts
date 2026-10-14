// Types of the facts ledger and the bibliography.
//
// Every number in the prose that is not computed live comes from the ledger (`facts.ts`), and
// every ledger entry names at least one source of the bibliography (`sources.ts`). Components
// render a fact with `<Fact>` and a reference with `<Cite>`.

/** A value stated on the site, with the sources that support it. */
export interface Fact {
  /** Stable key, for example `bb5-steps`. */
  id: string;
  /** The value in machine-readable form, for comparisons with live results. */
  value: number | string;
  /** The value as printed, for example `47,176,870`. */
  display: string;
  /** Ids of the supporting sources, in `sources.ts`. At least one. */
  source: string[];
  /** A direct link to the place in the source, if it is more specific than the source's link. */
  url?: string;
  /** The date the value was last checked against its sources, as `YYYY-MM-DD`. */
  verified: string;
  /** Qualifications, for example the exact definition used by the source. */
  note?: string;
}

/** An entry of the bibliography. */
export interface Source {
  /** Stable key, for example `turing1936`. */
  id: string;
  /** Authors as printed, for example `A. M. Turing` or `The bbchallenge Collaboration`. */
  authors: string;
  /** Title of the work. */
  title: string;
  /** Journal, proceedings, publisher, website or course, as printed. */
  venue: string;
  /** Year of publication. */
  year: number;
  /** Link to the work. */
  url?: string;
  /** DOI without the resolver prefix, for example `10.1112/plms/s2-42.1.230`. */
  doi?: string;
  /** arXiv identifier, for example `2509.12337`. */
  arxiv?: string;
}

/** A chapter of the site. */
export interface Chapter {
  /** Position, from 1. */
  n: number;
  /** Route below the base path, without slashes. */
  slug: string;
  /** Title as shown in headings and navigation. */
  title: string;
  /** One plain sentence about the content. */
  summary: string;
}
