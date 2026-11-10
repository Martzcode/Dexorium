import { Component } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';

import { AppIcon, type IconName } from '../../shared/icon/app-icon';

interface FoundationItem {
  readonly key: string;
  readonly icon: IconName;
}

type RoadmapStatus = 'current' | 'next' | 'planned';

interface RoadmapItem {
  readonly version: string;
  readonly key: string;
  readonly status: RoadmapStatus;
}

@Component({
  selector: 'app-home',
  imports: [TranslocoPipe, AppIcon],
  templateUrl: './home.page.html',
})
export class HomePage {
  protected readonly foundation: readonly FoundationItem[] = [
    { key: 'shell', icon: 'app-window' },
    { key: 'rust', icon: 'cpu' },
    { key: 'ipc', icon: 'arrow-left-right' },
    { key: 'ui', icon: 'layout-dashboard' },
    { key: 'data', icon: 'database' },
    { key: 'offline', icon: 'wifi-off' },
  ];

  protected readonly roadmap: readonly RoadmapItem[] = [
    { version: 'v1.0', key: 'mvp', status: 'current' },
    { version: 'v1.1', key: 'v11', status: 'next' },
    { version: 'v1.2', key: 'v12', status: 'planned' },
  ];

  protected roadmapItemClass(status: RoadmapStatus): string {
    const base = 'rounded-2xl border p-[22px]';
    return status === 'current'
      ? `${base} border-accent/45 bg-[linear-gradient(150deg,color-mix(in_srgb,var(--accent)_8%,transparent),transparent_60%),var(--surface)]`
      : `${base} border-line bg-panel`;
  }

  protected roadmapStatusClass(status: RoadmapStatus): string {
    const base = 'rounded-full px-2 py-0.5 font-mono text-[0.62rem] uppercase tracking-[0.08em]';
    return status === 'current'
      ? `${base} bg-accent/15 text-accent`
      : `${base} bg-panel-muted text-ink-muted`;
  }
}
