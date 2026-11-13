import { Component, computed, input } from '@angular/core';

/** Original type-coloured placeholder used until a media pack is installed. */
const TYPE_COLORS: Readonly<Record<string, readonly [string, string]>> = {
  normal: ['#a8a77a', '#6d6c4f'],
  fire: ['#f97316', '#b91c1c'],
  water: ['#38bdf8', '#1d4ed8'],
  electric: ['#facc15', '#b45309'],
  grass: ['#4ade80', '#15803d'],
  ice: ['#67e8f9', '#0e7490'],
  fighting: ['#f87171', '#991b1b'],
  poison: ['#c084fc', '#6b21a8'],
  ground: ['#d6b25e', '#8a5a1f'],
  flying: ['#93c5fd', '#475569'],
  psychic: ['#f472b6', '#9d174d'],
  bug: ['#a3e635', '#4d7c0f'],
  rock: ['#b8a038', '#6b5a1f'],
  ghost: ['#818cf8', '#3730a3'],
  dragon: ['#7c3aed', '#4c1d95'],
  dark: ['#57534e', '#1c1917'],
  steel: ['#94a3b8', '#475569'],
  fairy: ['#f9a8d4', '#be185d'],
};

const FALLBACK: readonly [string, string] = ['#94a3b8', '#475569'];

@Component({
  selector: 'app-pokemon-sprite',
  template: `
    <span
      class="relative inline-grid shrink-0 place-items-center overflow-hidden rounded-full border border-white/25 shadow-inner"
      [style.width.px]="size()"
      [style.height.px]="size()"
      [style.background]="background()"
      role="img"
      [attr.aria-label]="label()"
    >
      <span class="absolute inset-x-0 top-1/2 h-px bg-black/30"></span>
      <span class="absolute inset-[18%] rounded-full border border-white/25"></span>
      <span class="absolute left-1/2 top-1/2 h-[18%] w-[18%] -translate-x-1/2 -translate-y-1/2 rounded-full border border-white/50 bg-black/20"></span>
      <span
        class="relative z-10 mt-[38%] font-mono font-bold leading-none text-white/95"
        [style.font-size.px]="size() * 0.2"
      >
        {{ dexLabel() }}
      </span>
    </span>
  `,
})
export class PokemonSprite {
  readonly dexNumber = input.required<number>();
  readonly name = input.required<string>();
  readonly types = input<readonly string[]>([]);
  readonly size = input(72);

  protected readonly dexLabel = computed(() => `#${String(this.dexNumber()).padStart(4, '0')}`);
  protected readonly label = computed(() => this.name());
  protected readonly background = computed(() => {
    const [top, bottom] = TYPE_COLORS[this.types()[0] ?? 'normal'] ?? FALLBACK;
    return `linear-gradient(160deg, ${top}, ${bottom})`;
  });
}
