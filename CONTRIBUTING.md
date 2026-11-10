# Contributing to Dexorium

Thanks for taking the time to contribute! Dexorium is an offline-first desktop
Pokémon encyclopedia built with Tauri 2, Rust and Angular 20.

## Before you start

- Search existing issues and pull requests to avoid duplicate work.
- For significant changes, open an issue first so the design can be discussed.
- Keep the product promise intact: **no runtime network dependency, no account,
  no telemetry**.

## Development setup

```bash
npm install
npm run desktop:dev
```

For UI-only work, `npm run start` serves the Angular app in a browser. The IPC
bridge is intentionally unavailable there.

## Quality gates

Before opening a pull request, run:

```bash
npm run format
npm run check
```

`npm run check` verifies formatting, builds the Angular application and runs the
Rust workspace checks and tests. Pull requests that introduce warnings or break
the build will not be merged.

## Architectural rules

- `pdx-domain` contains pure business rules and **must not** depend on Tauri,
  SQLite, the file system or any I/O crate.
- `pdx-app` owns use cases and ports. It may depend on `pdx-domain`.
- `pdx-infra` implements adapters (SQLite, search, cache) and depends on the
  application layer.
- Tauri commands stay thin: one command maps to one use case.
- Angular components never call `invoke()` directly. Go through a feature facade
  and the shared typed IPC wrapper.
- Shared state uses signals / NgRx Signal Store. Use `ChangeDetectionStrategy.OnPush`
  and keep the application zoneless.
- Every user-facing string goes through Transloco, with both `en` and `fr`
  dictionaries updated.

## Commit and pull request guidelines

- Use clear, imperative commit messages (for example `feat(pokedex): add species
stats panel`).
- Keep pull requests focused. Separate refactors from behavior changes.
- Describe the problem, the chosen approach and how you tested it.
- Include screenshots or a short recording for UI changes.
- Add or update tests for the behavior you change.

## Reporting bugs

Include the operating system, application version, steps to reproduce and the
expected versus actual result. Do not attach copyrighted game data or official
sprites to issues.

## License

By contributing, you agree that your contributions are licensed under the
GNU Affero General Public License v3.0 or later, as described in the project
[LICENSE](./LICENSE).
