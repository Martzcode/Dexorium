import { NgComponentOutlet } from '@angular/common';
import { Component, computed, input } from '@angular/core';
import {
  LucideArrowLeft,
  LucideBookOpen,
  LucideBrain,
  LucideLayoutDashboard,
  LucideLibraryBig,
  LucideMenu,
  LucideMoon,
  LucidePackage,
  LucideSearch,
  LucideSettings,
  LucideSun,
  LucideSwords,
  LucideUsers,
  LucideWrench,
} from '@lucide/angular';

const ICONS = {
  'arrow-left': LucideArrowLeft,
  'book-open': LucideBookOpen,
  brain: LucideBrain,
  'layout-dashboard': LucideLayoutDashboard,
  'library-big': LucideLibraryBig,
  menu: LucideMenu,
  moon: LucideMoon,
  package: LucidePackage,
  search: LucideSearch,
  settings: LucideSettings,
  sun: LucideSun,
  swords: LucideSwords,
  users: LucideUsers,
  wrench: LucideWrench,
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
