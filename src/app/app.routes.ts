import type { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    title: 'Dexorium — Offline Pokémon Desktop Encyclopedia',
    loadComponent: () => import('./features/home/home.page').then((module) => module.HomePage),
  },
  {
    path: '**',
    redirectTo: '',
  },
];
