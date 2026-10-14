// The menus of a state and of a symbol, shared by the table, the list and the diagram.

import type { MenuItem } from './Menu.svelte';
import type { Origin, Playground } from './state.svelte';

export function stateMenu(
  pg: Playground,
  name: string,
  actions: { rename: () => void; addTransition?: () => void },
  origin: Origin = 'table'
): MenuItem[] {
  const spec = pg.spec;
  if (!spec) return [];
  const state = spec.states.find((s) => s.name === name);
  const items: MenuItem[] = [];
  if (actions.addTransition) {
    items.push({ label: 'Add transition', onselect: actions.addTransition });
  }
  items.push(
    { label: 'Rename', onselect: actions.rename },
    {
      label: 'Start state',
      checked: spec.start === name,
      onselect: () => pg.setStart(name, origin)
    },
    {
      label: 'Final state',
      checked: state?.isFinal ?? false,
      onselect: () => pg.toggleFinal(name, origin)
    },
    {
      label: 'Stop when entered',
      checked: pg.hasBreakpoint({ kind: 'state', state: name }),
      onselect: () => pg.toggleBreakpoint({ kind: 'state', state: name })
    },
    {
      label: 'Delete state',
      disabled: spec.states.length <= 1,
      onselect: () => pg.removeState(name, origin)
    }
  );
  return items;
}

export function symbolMenu(pg: Playground, glyph: string): MenuItem[] {
  const spec = pg.spec;
  if (!spec) return [];
  const blank = glyph === spec.blank;
  const input = spec.inputAlphabet.includes(glyph);
  return [
    {
      label: 'Allowed in the input',
      checked: input,
      disabled: blank,
      onselect: () => pg.setInputSymbol(glyph, !input)
    },
    { label: 'Remove symbol', disabled: blank, onselect: () => pg.removeSymbol(glyph) }
  ];
}
