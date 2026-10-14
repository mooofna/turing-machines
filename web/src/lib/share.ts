// Share links of the playground, kept entirely in the URL hash so that no server is involved.
//
//   #g=<gallery id>          a machine of the gallery
//   #tm=<standard format>    a busy beaver style machine, for example 1RB1LB_1LA1RZ
//   #src=1.<payload>         a description, deflated and base64url encoded by the engine
//
// Each may be followed by &in=<input>, the input word to start with.

export type SharedMachine =
  | { kind: 'gallery'; id: string }
  | { kind: 'standard'; text: string }
  | { kind: 'source'; payload: string };

export interface SharedLink {
  machine: SharedMachine;
  input?: string;
}

const KEYS: Record<string, SharedMachine['kind']> = {
  g: 'gallery',
  tm: 'standard',
  src: 'source'
};

function decode(value: string): string | null {
  try {
    return decodeURIComponent(value.replace(/\+/g, '%20'));
  } catch {
    return null;
  }
}

/** Reads a hash such as `#g=bb4&in=101`; null if it names no machine. */
export function parseHash(hash: string): SharedLink | null {
  const body = hash.startsWith('#') ? hash.slice(1) : hash;
  if (!body) return null;
  let machine: SharedMachine | null = null;
  let input: string | undefined;
  for (const part of body.split('&')) {
    const eq = part.indexOf('=');
    if (eq < 0) continue;
    const key = part.slice(0, eq);
    const value = decode(part.slice(eq + 1));
    if (value === null) continue;
    if (key === 'in') {
      input = value;
      continue;
    }
    const kind = KEYS[key];
    if (!kind || machine || value === '') continue;
    if (kind === 'gallery') machine = { kind, id: value };
    else if (kind === 'standard') machine = { kind, text: value };
    else machine = { kind, payload: value };
  }
  if (!machine) return null;
  return input === undefined ? { machine } : { machine, input };
}

/** Characters that may stay unescaped in a hash value (RFC 3986 fragment minus & and =). */
function encode(value: string): string {
  return encodeURIComponent(value).replace(/%2C/g, ',').replace(/%3A/g, ':').replace(/%40/g, '@');
}

export function buildHash(link: SharedLink): string {
  const m = link.machine;
  const head =
    m.kind === 'gallery'
      ? `g=${encode(m.id)}`
      : m.kind === 'standard'
        ? `tm=${encode(m.text)}`
        : `src=${encode(m.payload)}`;
  return link.input === undefined ? `#${head}` : `#${head}&in=${encode(link.input)}`;
}
