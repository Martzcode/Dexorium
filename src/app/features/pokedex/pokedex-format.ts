const TYPE_BADGE_BASE =
  'inline-flex items-center rounded-full border px-2 py-0.5 font-mono text-[0.62rem] font-semibold uppercase tracking-[0.06em]';

const TYPE_CLASSES: Readonly<Record<string, string>> = {
  normal: 'border-stone-500/30 bg-stone-500/15 text-stone-700 dark:text-stone-300',
  fire: 'border-orange-500/30 bg-orange-500/15 text-orange-700 dark:text-orange-300',
  water: 'border-blue-500/30 bg-blue-500/15 text-blue-700 dark:text-blue-300',
  electric: 'border-yellow-500/30 bg-yellow-400/20 text-yellow-700 dark:text-yellow-300',
  grass: 'border-green-500/30 bg-green-500/15 text-green-700 dark:text-green-300',
  ice: 'border-cyan-500/30 bg-cyan-400/15 text-cyan-700 dark:text-cyan-300',
  fighting: 'border-red-600/30 bg-red-600/15 text-red-700 dark:text-red-300',
  poison: 'border-fuchsia-500/30 bg-fuchsia-500/15 text-fuchsia-700 dark:text-fuchsia-300',
  ground: 'border-amber-500/30 bg-amber-500/15 text-amber-700 dark:text-amber-300',
  flying: 'border-indigo-400/30 bg-indigo-400/15 text-indigo-700 dark:text-indigo-300',
  psychic: 'border-pink-500/30 bg-pink-500/15 text-pink-700 dark:text-pink-300',
  bug: 'border-lime-500/30 bg-lime-500/15 text-lime-700 dark:text-lime-300',
  rock: 'border-stone-500/30 bg-stone-500/15 text-stone-700 dark:text-stone-300',
  ghost: 'border-violet-600/30 bg-violet-600/15 text-violet-700 dark:text-violet-300',
  dragon: 'border-indigo-600/30 bg-indigo-600/15 text-indigo-700 dark:text-indigo-300',
  dark: 'border-neutral-600/30 bg-neutral-700/15 text-neutral-700 dark:text-neutral-300',
  steel: 'border-slate-400/40 bg-slate-400/15 text-slate-600 dark:text-slate-300',
  fairy: 'border-rose-400/30 bg-rose-400/15 text-rose-700 dark:text-rose-300',
};

const TYPE_FALLBACK = 'border-line bg-panel-muted text-ink-soft';

/** Returns the full Tailwind class list for a type badge. */
export function typeBadgeClass(type: string): string {
  return `${TYPE_BADGE_BASE} ${TYPE_CLASSES[type] ?? TYPE_FALLBACK}`;
}

/** Formats `25` as `#0025`. */
export function formatDexNumber(value: number): string {
  return `#${value.toString().padStart(4, '0')}`;
}
