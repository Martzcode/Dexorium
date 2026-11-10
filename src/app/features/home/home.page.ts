import { Component, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';

import { AppInfoService, type AppInfoStatus } from '../../core/app-info/app-info.service';

interface FoundationItem {
  readonly key: string;
  readonly icon: string;
}

interface RoadmapItem {
  readonly version: string;
  readonly key: string;
  readonly status: 'current' | 'next' | 'planned';
}

@Component({
  selector: 'app-home',
  imports: [TranslocoPipe],
  templateUrl: './home.page.html',
  styleUrl: './home.page.scss',
})
export class HomePage {
  protected readonly appInfo = inject(AppInfoService);

  protected readonly foundation: readonly FoundationItem[] = [
    { key: 'shell', icon: '▣' },
    { key: 'rust', icon: '⚙' },
    { key: 'ipc', icon: '⇄' },
    { key: 'ui', icon: '◈' },
    { key: 'data', icon: '◫' },
    { key: 'offline', icon: '⊘' },
  ];

  protected readonly roadmap: readonly RoadmapItem[] = [
    { version: 'v1.0', key: 'mvp', status: 'current' },
    { version: 'v1.1', key: 'v11', status: 'next' },
    { version: 'v1.2', key: 'v12', status: 'planned' },
  ];

  protected statusLabelKey(status: AppInfoStatus): string {
    switch (status) {
      case 'ready':
        return 'home.runtime.connected';
      case 'browser':
        return 'home.runtime.browser';
      case 'error':
        return 'home.runtime.error';
      default:
        return 'home.runtime.loading';
    }
  }
}
