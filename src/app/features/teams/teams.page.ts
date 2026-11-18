import { Component, inject, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

import { PokedexService } from '../pokedex/pokedex.service';
import type { SpeciesSummary } from '../pokedex/pokedex.models';

interface TeamMember {
  species: SpeciesSummary | null;
  level: number;
  nature: string;
  ability: string;
  item: string;
  moves: string[];
  evs: number[];
  ivs: number[];
  shiny: boolean;
  teraType: string;
}

interface Team {
  id: string;
  name: string;
  format: string;
  members: TeamMember[];
  notes: string;
}

const STORAGE_KEY = 'dexorium.teams';

@Component({
  selector: 'app-teams',
  imports: [TranslocoPipe],
  templateUrl: './teams.page.html',
})
export class TeamsPage {
  private readonly pokedex = inject(PokedexService);
  private readonly transloco = inject(TranslocoService);

  protected readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly teams = signal<Team[]>(this.load());
  protected readonly selected = signal<Team | null>(null);
  protected readonly editing = signal<Team | null>(null);

  private load(): Team[] {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      return raw ? (JSON.parse(raw) as Team[]) : [];
    } catch {
      return [];
    }
  }

  private persist(): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.teams()));
    } catch {
      /* ignore */
    }
  }

  protected newTeam(): void {
    const team: Team = {
      id: crypto.randomUUID(),
      name: 'New team',
      format: 'Gen 9',
      members: Array.from({ length: 6 }, () => ({
        species: null,
        level: 50,
        nature: 'hardy',
        ability: '',
        item: '',
        moves: ['', '', '', ''],
        evs: [0, 0, 0, 0, 0, 0],
        ivs: [31, 31, 31, 31, 31, 31],
        shiny: false,
        teraType: '',
      })),
      notes: '',
    };
    this.teams.update((teams) => [...teams, team]);
    this.persist();
    this.selected.set(team);
  }

  protected deleteTeam(team: Team): void {
    this.teams.update((teams) => teams.filter((t) => t.id !== team.id));
    this.persist();
    if (this.selected()?.id === team.id) {
      this.selected.set(null);
    }
  }

  protected exportShowdown(team: Team): string {
    return team.members
      .filter((m) => m.species)
      .map((m) => {
        const lines: string[] = [];
        lines.push(`${m.species!.name} @ ${m.item || ''}`.trim());
        if (m.ability) lines.push(`Ability: ${m.ability}`);
        if (m.shiny) lines.push('Shiny: Yes');
        if (m.teraType) lines.push(`Tera Type: ${m.teraType}`);
        lines.push(`EVs: ${m.evs.map((v, i) => `${v} ${['HP', 'Atk', 'Def', 'SpA', 'SpD', 'Spe'][i]}`).filter((x) => !x.startsWith('0')).join(' / ')}`);
        lines.push(`${m.nature} Nature`);
        lines.push(`IVs: ${m.ivs.map((v, i) => `${v} ${['HP', 'Atk', 'Def', 'SpA', 'SpD', 'Spe'][i]}`).join(' / ')}`);
        for (const move of m.moves) {
          if (move) lines.push(`- ${move}`);
        }
        return lines.join('\n');
      })
      .join('\n\n');
  }
}
