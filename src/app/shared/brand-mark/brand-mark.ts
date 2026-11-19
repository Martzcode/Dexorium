import { Component, input } from '@angular/core';

/**
 * Original Dexorium brand mark: a magnifier over an index list.
 *
 * It deliberately avoids any third-party trade dress so the packaged icon,
 * the in-app chrome and the Microsoft Store artwork stay consistently ours.
 */
@Component({
  selector: 'app-brand-mark',
  template: `
    <svg
      [attr.width]="size()"
      [attr.height]="size()"
      viewBox="0 0 32 32"
      fill="none"
      aria-hidden="true"
      focusable="false"
    >
      <rect
        x="1"
        y="1"
        width="30"
        height="30"
        rx="8"
        fill="var(--surface-muted)"
        stroke="var(--border)"
      />
      <path d="M19.6 19.6 25 25" stroke="var(--accent)" stroke-width="3.2" stroke-linecap="round" />
      <circle cx="14" cy="14" r="7.2" stroke="var(--accent)" stroke-width="2.6" />
      <g stroke="var(--text)" stroke-width="1.4" stroke-linecap="round">
        <path d="M10.4 11.2h7.2" opacity="0.95" />
        <path d="M10.4 14.2h7.2" opacity="0.8" />
        <path d="M10.4 17.2h4.6" opacity="0.6" />
      </g>
    </svg>
  `,
})
export class BrandMark {
  readonly size = input(32);
}
