import { Component, inject, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

@Component({
  selector: 'app-collection',
  imports: [TranslocoPipe],
  templateUrl: './collection.page.html',
})
export class CollectionPage {
  private readonly transloco = inject(TranslocoService);

  protected readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly profile = signal('default');
  protected readonly captured = signal(0);
  protected readonly seen = signal(0);
  protected readonly shiny = signal(0);
}
