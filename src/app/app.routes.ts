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
    path: '**',
    redirectTo: '',
  },
];
