import { NgComponentOutlet } from '@angular/common';
import { Component, computed, input } from '@angular/core';
import {
  LucideArrowLeft,
  LucideBookOpen,
  LucideBrain,
  LucideCircleCheck,
  LucideCommand,
  LucideCopy,
  LucideDatabase,
  LucideExternalLink,
  LucideLayoutDashboard,
  LucideLayoutGrid,
  LucideLibraryBig,
  LucideList,
  LucideMenu,
  LucideMinus,
  LucideMonitor,
  LucideMoon,
  LucidePackage,
  LucideSearch,
  LucideSettings,
  LucideShieldCheck,
  LucideSparkles,
  LucideSquare,
  LucideStar,
  LucideSun,
  LucideSwords,
  LucideTable,
  LucideUsers,
  LucideWrench,
  LucideX,
} from '@lucide/angular';

const ICONS = {
  'arrow-left': LucideArrowLeft,
  'book-open': LucideBookOpen,
  brain: LucideBrain,
  'circle-check': LucideCircleCheck,
  command: LucideCommand,
  copy: LucideCopy,
  database: LucideDatabase,
  'external-link': LucideExternalLink,
  'layout-dashboard': LucideLayoutDashboard,
  'layout-grid': LucideLayoutGrid,
  'library-big': LucideLibraryBig,
  list: LucideList,
  menu: LucideMenu,
  minus: LucideMinus,
  monitor: LucideMonitor,
  moon: LucideMoon,
  package: LucidePackage,
  search: LucideSearch,
  settings: LucideSettings,
  'shield-check': LucideShieldCheck,
  sparkles: LucideSparkles,
  square: LucideSquare,
  star: LucideStar,
  sun: LucideSun,
  swords: LucideSwords,
  table: LucideTable,
  users: LucideUsers,
  wrench: LucideWrench,
  x: LucideX,
} as const;

export type IconName = keyof typeof ICONS;

@Component({
  selector: 'app-icon',
  imports: [NgComponentOutlet],
  template: `<ng-container *ngComponentOutlet="icon(); inputs: iconInputs()" />`,
  host: {
    class: 'inline-flex shrink-0 items-center justify-center',
  },
})
export class AppIcon {
  readonly name = input.required<IconName>();
  readonly size = input(18);

  protected readonly icon = computed(() => ICONS[this.name()]);
  protected readonly iconInputs = computed(() => ({ size: this.size() }));
}
