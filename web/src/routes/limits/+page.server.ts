import { resolve } from '$app/paths';
import type { RouteId } from '$app/types';
import { loadChapter } from '#lib/content/render.server.ts';

export const load = () => loadChapter('limits', (c) => resolve(`/${c.slug}` as RouteId));
