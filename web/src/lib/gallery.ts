// The gallery: every machine of the site, read at build time from `machines/gallery.json` and the
// `.tm` descriptions next to it. The JSON is the only list of machines; the playground opens an
// entry with `#g=<id>`.

import data from '../../../machines/gallery.json';
import type { MachineSpecDto, SpaceTimeModeDto } from './engine/pkg/turing_wasm';
import type { Wasm } from './engine/wasm';

/** An input and the result the machine must produce on it, checked by the tests. */
export interface GalleryExample {
  input: string;
  expect: string;
}

export interface GalleryGroup {
  id: string;
  title: string;
}

/** An entry of `gallery.json`. Exactly one of `file` and `standard` is set. */
export interface GalleryEntry {
  id: string;
  title: string;
  /** Id of a group in `groups`. */
  group: string;
  /** Path of a description in the course notation, relative to `machines/`. */
  file?: string;
  /** The machine in the bbchallenge standard format. */
  standard?: string;
  /** Default input on the tape. */
  input?: string;
  /** Where the machine comes from, as printed. */
  provenance: string;
  /** Id of a source in the bibliography. */
  source?: string;
  /** One plain sentence. */
  description: string;
  /** Step budget for machines that run long or forever. */
  budget?: number;
  examples: GalleryExample[];
}

export interface GalleryFile {
  version: 1;
  groups: GalleryGroup[];
  machines: GalleryEntry[];
}

/** An entry with the text it is parsed from. */
export interface GalleryMachine extends GalleryEntry {
  /** The `.tm` description or the standard-format string. */
  text: string;
  format: 'dsl' | 'standard';
}

const gallery = data as GalleryFile;

const files = import.meta.glob<string>('/../machines/**/*.tm', {
  query: '?raw',
  import: 'default',
  eager: true
});

/** `.tm` sources by path relative to `machines/`, for example `course/binary-increment.tm`. */
const sources = new Map(
  Object.entries(files).map(([path, text]) => [path.replace(/^.*?machines\//, ''), text])
);

function withText(entry: GalleryEntry): GalleryMachine {
  if (entry.standard) return { ...entry, text: entry.standard, format: 'standard' };
  const text = entry.file ? sources.get(entry.file) : undefined;
  if (text === undefined) {
    throw new Error(`gallery entry ${entry.id}: missing file machines/${entry.file ?? '?'}`);
  }
  return { ...entry, text, format: 'dsl' };
}

export const groups: readonly GalleryGroup[] = gallery.groups;
export const machines: readonly GalleryMachine[] = gallery.machines.map(withText);

/** The entry with `id`, if any. */
export function galleryMachine(id: string): GalleryMachine | undefined {
  return machines.find((m) => m.id === id);
}

/** The entries of a group, in file order. */
export function machinesIn(group: string): GalleryMachine[] {
  return machines.filter((m) => m.group === group);
}

/** Parses an entry with the editor services; undefined if it has errors. */
export function specOf(wasm: Wasm, machine: GalleryMachine): MachineSpecDto | undefined {
  const parsed =
    machine.format === 'standard'
      ? wasm.fromStandard(machine.text)
      : wasm.parseSource(machine.text);
  return parsed.spec ?? undefined;
}

/**
 * How a thumbnail shows a machine: an overview of the whole run for busy beavers and other
 * machines with a step budget, exact steps from the start for the others.
 */
export function thumbnailMode(machine: GalleryMachine, spec: MachineSpecDto): SpaceTimeModeDto {
  if (spec.kind === 'beaver' || machine.budget !== undefined) {
    return { kind: 'overview', maxSteps: machine.budget ?? 100_000_000 };
  }
  return { kind: 'exact', firstStep: 0, cellPx: 2 };
}
