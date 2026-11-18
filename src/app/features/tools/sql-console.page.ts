import { Component, inject, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { invokeCommand, isTauriRuntime } from '../../core/ipc/tauri-ipc';

@Component({
  selector: 'app-sql-console',
  imports: [TranslocoPipe],
  templateUrl: './sql-console.page.html',
})
export class SqlConsolePage {
  private readonly transloco = inject(TranslocoService);

  protected readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly sql = signal('SELECT slug, name FROM species LIMIT 20');
  protected readonly columns = signal<string[]>([]);
  protected readonly rows = signal<string[][]>([]);
  protected readonly error = signal<string | null>(null);
  protected readonly running = signal(false);

  protected async run(): Promise<void> {
    if (!isTauriRuntime()) {
      this.error.set('Desktop runtime required');
      return;
    }
    this.running.set(true);
    this.error.set(null);
    try {
      const result = await invokeCommand<[string[], string[][]]>('run_sql_query', { sql: this.sql() });
      this.columns.set(result[0]);
      this.rows.set(result[1]);
    } catch (error) {
      this.error.set(String(error));
      this.columns.set([]);
      this.rows.set([]);
    } finally {
      this.running.set(false);
    }
  }
}
