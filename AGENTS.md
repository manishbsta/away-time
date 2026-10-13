# Agent instructions

These instructions apply to all work in this repository. Read
[CONTRIBUTING.md](CONTRIBUTING.md) before creating branches, commits, or pull
requests. Agents must follow its conventions strictly.

## Branches and pull requests

- Start work branches from the latest `origin/develop`. Fetch before branching
  when network access is available. Preserve existing user changes.
- Use `<type>/<short-description>` with a lowercase type and a specific,
  lowercase, hyphen-separated description.
- Allowed types: `feat`, `fix`, `docs`, `refactor`, `test`, `perf`, `build`, `ci`,
  and `chore`.
- Do not add `codex/`, agent names, or other tool-specific prefixes.
- Examples: `feat/break-preview-shortcut`, `fix/tray-menu-resume`,
  `docs/agent-conventions`, and `ci/check-rust-formatting`.
- Include a tracked issue identifier in lowercase when applicable, for example
  `fix/at-42-snapshot-recovery`.
- Do not implement changes directly on the shared `main` or `develop` branches.
- Target contribution PRs at `develop`. Only release promotion PRs from
  `develop` target `main`.

## Commits and PR titles

- Use Conventional Commits: `<type>(<optional-scope>): <description>`.
- Use the allowed types above. Keep types and scopes lowercase; omit scope
  parentheses when no scope is needed.
- Start the description with a lowercase imperative action verb, such as `add`,
  `fix`, `remove`, `update`, or `prevent`. Explain the concrete change.
- Keep the first line at most 72 characters, with no trailing period.
- Keep each commit focused on one coherent change. Avoid vague messages such
  as `wip`, `updates`, or `final changes`.
- For additional context, leave a blank line after the subject and explain why
  the change is needed. Put issue references in the body.
- Mark incompatible changes with `!` before the colon and a
  `BREAKING CHANGE:` footer describing impact and migration.
- PR titles must follow the same format so squash commits remain compliant.
- Examples: `docs: add agent contribution instructions`,
  `fix(storage): preserve the recovery snapshot`, and
  `ci: check Rust formatting on pull requests`.
- Before committing or opening a PR, check branch names and commit subjects
  against these rules. The current hooks do not validate names or messages.
  Correct noncompliant names or messages created during your task before
  requesting review. Do not rewrite shared history without authorization.

## Validation

- Install dependencies with `npm ci`; this also installs the Husky hooks.
- Keep hooks enabled. Do not use `--no-verify` or disable Husky to bypass a
  failing check.
- Pre-commit checks staged frontend files with Oxlint and Prettier, then checks
  Rust workspace formatting. Pre-push checks frontend TypeScript types.
- Run checks appropriate to the change. Available checks include
  `npm run lint`, `npm run format:check`, `npm run typecheck`,
  `npm run format:rust:check`, and the foundation checks in `npm run check`.
- Report which checks passed and any checks that could not be run. Do not claim
  that local checks prove hosted CI has passed.
