import { Injectable, signal } from '@angular/core';

/** Shared open/close state for the `Ctrl+K` command palette. */
@Injectable({ providedIn: 'root' })
export class CommandPaletteService {
  private readonly visible = signal(false);

  readonly open = this.visible.asReadonly();

  show(): void {
    this.visible.set(true);
  }

  hide(): void {
    this.visible.set(false);
  }

  toggle(): void {
    this.visible.update((value) => !value);
  }
}
