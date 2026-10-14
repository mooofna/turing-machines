// Messages between the page and the engine workers.
//
// Every request carries an id. A request is answered by one or more responses with the same id;
// the last one has `final: true`. Long jobs (run, spacetime, enumerate, antihydra) stream
// progress and can be cancelled with a `cancel` request naming their id.

import type {
  AntihydraRowDto,
  BreakpointsDto,
  CellRefDto,
  EnumProgressDto,
  MachineSpecDto,
  PaletteDto,
  RunResultDto,
  SnapshotDto,
  SpaceTimeProgressDto,
  SpaceTimeRequestDto,
  VerdictDto,
  WalkSummaryDto
} from './pkg/turing_wasm';

export type Request =
  /** Opens a session; answered with `snapshot`. */
  | { type: 'open'; session: string; spec: MachineSpecDto; input: string; radius: number }
  | { type: 'snapshot'; session: string; radius: number }
  | { type: 'step'; session: string; count: number; radius: number }
  | { type: 'back'; session: string; count: number; radius: number }
  | { type: 'seek'; session: string; step: number; radius: number }
  /** Streams `progress` about ten times a second and ends with `stopped`. */
  | {
      type: 'run';
      session: string;
      maxSteps: number;
      breakpoints: BreakpointsDto;
      radius: number;
    }
  | { type: 'close'; session: string }
  /** Streams `spacetime` frames as rows are filled; the worker keeps the job for `recolor`. */
  | {
      type: 'spacetime';
      spec: MachineSpecDto;
      input: string;
      request: SpaceTimeRequestDto;
      palette: PaletteDto;
    }
  | { type: 'recolor'; job: number; palette: PaletteDto }
  | { type: 'locate'; job: number; x: number; y: number }
  | { type: 'release'; job: number }
  /** Streams `enum` progress; the final message carries examples and holdouts. */
  | {
      type: 'enumerate';
      states: number;
      symbols: number;
      shard?: { index: number; count: number };
      examples?: number;
    }
  | { type: 'decide'; spec: MachineSpecDto }
  /** Streams rows observed on the machine until `maxRows` rows or the check ends. */
  | { type: 'antihydra-check'; maxRows: number }
  /** Streams summaries while iterating the rule `iterations` times. */
  | { type: 'antihydra-walk'; iterations: number; samples: number }
  | { type: 'cancel'; target: number };

export type Response =
  | { type: 'snapshot'; snapshot: SnapshotDto }
  | { type: 'progress'; snapshot: SnapshotDto; stepsPerSecond: number }
  | { type: 'stopped'; result: RunResultDto; elapsedMs: number }
  | {
      type: 'spacetime';
      job: number;
      width: number;
      height: number;
      pixels: Uint8ClampedArray<ArrayBuffer>;
      progress: SpaceTimeProgressDto;
      elapsedMs: number;
    }
  | { type: 'cell'; cell: CellRefDto | null }
  | {
      type: 'enum';
      progress: EnumProgressDto;
      elapsedMs: number;
      examples?: Record<string, string[]>;
      holdouts?: string[];
    }
  | { type: 'verdict'; verdict: VerdictDto }
  | { type: 'antihydra-rows'; rows: AntihydraRowDto[]; done: boolean }
  | { type: 'antihydra-walk'; summary: WalkSummaryDto; done: boolean; elapsedMs: number }
  | { type: 'ok' }
  | { type: 'cancelled' }
  | { type: 'error'; message: string };

export type Envelope<T> = T & { id: number; final?: boolean };
