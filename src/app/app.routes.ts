import type { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    title: 'Dexorium — Offline Pokémon Desktop Encyclopedia',
    loadComponent: () => import('./features/home/home.page').then((module) => module.HomePage),
  },
  {
    path: 'pokedex',
    title: 'Pokédex — Dexorium',
    loadComponent: () =>
      import('./features/pokedex/pokedex.page').then((module) => module.PokedexPage),
  },
  {
    path: 'pokedex/:id',
    title: 'Pokédex — Dexorium',
    loadComponent: () =>
      import('./features/pokedex/species-detail.page').then((module) => module.SpeciesDetailPage),
  },
  {
    path: 'moves',
    title: 'Attaques — Dexorium',
    loadComponent: () => import('./features/moves/moves.page').then((module) => module.MovesPage),
  },
  {
    path: 'moves/:id',
    title: 'Attaque — Dexorium',
    loadComponent: () =>
      import('./features/moves/move-detail.page').then((module) => module.MoveDetailPage),
  },
  {
    path: 'abilities',
    title: 'Talents — Dexorium',
    loadComponent: () =>
      import('./features/abilities/abilities.page').then((module) => module.AbilitiesPage),
  },
  {
    path: 'abilities/:id',
    title: 'Talent — Dexorium',
    loadComponent: () =>
      import('./features/abilities/ability-detail.page').then((module) => module.AbilityDetailPage),
  },
  {
    path: 'tools/types',
    title: 'Table des types — Dexorium',
    loadComponent: () =>
      import('./features/tools/type-matrix.page').then((module) => module.TypeMatrixPage),
  },
  {
    path: 'settings',
    title: 'Réglages — Dexorium',
    loadComponent: () =>
      import('./features/settings/settings.page').then((module) => module.SettingsPage),
  },
  {
    path: '**',
    redirectTo: '',
  },
];
