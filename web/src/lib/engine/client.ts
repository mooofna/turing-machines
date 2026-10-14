// The page side of the engine workers.
//
// `engine()` serves interactive work (sessions, space-time diagrams); `batch()` serves long jobs
// (enumeration, Antihydra), so a running enumeration never delays a step in the playground.

import type { Envelope, Request, Response } from './protocol';

export type ResponseOf<T extends Response['type']> = Extract<Response, { type: T }>;

export interface Call {
  /** The request id, for cancelling. */
  readonly id: number;
  /** Resolves with the final response; rejects with an `error` response. */
  readonly done: Promise<Response>;
  /** Asks the worker to stop; the call then resolves with its last state or `cancelled`. */
  cancel(): void;
}

interface Pending {
  onMessage?: (response: Response) => void;
  resolve: (response: Response) => void;
  reject: (error: Error) => void;
}

export class Engine {
  #worker: Worker | null = null;
  #next = 1;
  readonly #pending = new Map<number, Pending>();
  readonly #name: string;

  constructor(name: string) {
    this.#name = name;
  }

  #ensure(): Worker {
    if (this.#worker) return this.#worker;
    const worker = new Worker(new URL('./engine.worker.ts', import.meta.url), {
      type: 'module',
      name: this.#name
    });
    worker.onmessage = (event: MessageEvent<Envelope<Response>>) => {
      const message = event.data;
      const pending = this.#pending.get(message.id);
      if (!pending) return;
      const { id: _id, final, ...response } = message;
      if (response.type === 'error') {
        this.#pending.delete(message.id);
        pending.reject(new Error(response.message));
        return;
      }
      pending.onMessage?.(response as Response);
      if (final !== false) {
        this.#pending.delete(message.id);
        pending.resolve(response as Response);
      }
    };
    worker.onerror = (event) => {
      const error = new Error(event.message || 'engine worker failed');
      for (const pending of this.#pending.values()) pending.reject(error);
      this.#pending.clear();
      this.#worker?.terminate();
      this.#worker = null;
    };
    this.#worker = worker;
    return worker;
  }

  /** Sends a request; `onMessage` sees every response, including the final one. */
  call(request: Request, onMessage?: (response: Response) => void): Call {
    const worker = this.#ensure();
    const id = this.#next++;
    const done = new Promise<Response>((resolve, reject) => {
      this.#pending.set(id, { onMessage, resolve, reject });
    });
    const message: Envelope<Request> = { ...request, id };
    worker.postMessage(message);
    return {
      id,
      done,
      cancel: () => {
        if (this.#pending.has(id)) worker.postMessage({ type: 'cancel', target: id, id: 0 });
      }
    };
  }

  /** Sends a request and waits for its final response, which must be of type `type`. */
  async request<T extends Response['type']>(request: Request, type: T): Promise<ResponseOf<T>> {
    const response = await this.call(request).done;
    if (response.type !== type) {
      throw new Error(`expected ${type}, got ${response.type}`);
    }
    return response as ResponseOf<T>;
  }

  /** Stops the worker and rejects every pending call. */
  terminate(): void {
    this.#worker?.terminate();
    this.#worker = null;
    for (const pending of this.#pending.values()) pending.reject(new Error('engine stopped'));
    this.#pending.clear();
  }
}

let interactive: Engine | null = null;
let background: Engine | null = null;

/** The worker for sessions and space-time diagrams. */
export function engine(): Engine {
  interactive ??= new Engine('engine');
  return interactive;
}

/** The worker for enumeration and Antihydra. */
export function batch(): Engine {
  background ??= new Engine('batch');
  return background;
}
