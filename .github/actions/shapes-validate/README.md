# shapes-validate GitHub Action

A reusable composite action that runs `shapes validate` and `shapes ci-check` against your repository's `.shapes/` directory.

## What it enforces

| Check | Default | Description |
|---|---|---|
| `shapes validate` | always on | All graph integrity invariants — INV-001 through INV-018. Cycles, dangling references, reciprocal links, profile field requirements, binding target existence, and more. |
| **CI-001** — PR touches `.shapes/` | on | Fails if a pull request does not touch the shapes directory at all. Set `require-shapes-changes: 'false'` to opt out. |
| **CI-002** — amendment required | on, strict | Fails when a shape, constraint, or profile that was already in `promoted` or `canonical` state on the base ref is modified (or deleted) and no amendment in the PR targets it. By default every field counts, including `realization` / `evidence` / `provenance` bindings. See "Binding refreshes — a policy choice" below. |
| **CI-003** — amendment immutability | on | Fails when an existing amendment file is modified. Amendments are append-only per constraint:10. |

CI-002 and CI-003 only run on `pull_request` and `pull_request_target` events. `shapes validate` runs on every event.

## Binding refreshes — a policy choice

Some projects consider pure binding refreshes (file renames, `metadata.summary` tweaks, moving realization pointers after a refactor) "mechanical hygiene" and don't want those to require an amendment. Other projects treat every change on a canonical node as worth recording in the amendment log. This is a per-project policy call, so the three binding scopes are exposed as opt-out inputs:

- `allow-realization-refresh` (default `'false'`)
- `allow-evidence-refresh` (default `'false'`)
- `allow-provenance-refresh` (default `'false'`)

The defaults are strict. Flip any of them to `'true'` to tell CI-002 "treat this scope as hygiene, not semantic change" for *your* project. Leave them `'false'` if you want every canonical-node edit to land through the amendment workflow.

## Usage

```yaml
name: Shapes
on:
  pull_request:
  push:
    branches: [main]

jobs:
  shapes:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0   # required so the base ref is available for git show
      - uses: shapes-fyi/shapes/.github/actions/shapes-validate@v1
```

> **Important:** the action does not check out your repository for you, and it requires `fetch-depth: 0` so `git show <base>:<path>` can read the base ref. Forgetting this is the most common reason CI-002 reports false negatives.

## Inputs

| Input | Default | Description |
|---|---|---|
| `shapes-dir` | `.shapes` | Path to the shapes directory, relative to the repo root. |
| `base-ref` | `${{ github.event.pull_request.base.sha }}` | Base ref to diff against. Set explicitly when reusing the action outside the `pull_request` event. |
| `require-shapes-changes` | `'true'` | Set to `'false'` to allow PRs that don't touch the shapes directory. |
| `shapes-version` | `v1` | Git ref of `shapes-fyi/shapes` to install. Pin to a release tag for stability. |
| `install-shapes` | `'true'` | Set to `'false'` if a previous step has already placed `shapes` on PATH. |
| `allow-realization-refresh` | `'false'` | When `'true'`, realization binding changes on promoted/canonical nodes don't require an amendment. See "Binding refreshes — a policy choice" above. |
| `allow-evidence-refresh` | `'false'` | Same, for evidence bindings. |
| `allow-provenance-refresh` | `'false'` | Same, for provenance bindings. |

## Examples

### Allow code-only PRs (opt-out of CI-001)

```yaml
- uses: shapes-fyi/shapes/.github/actions/shapes-validate@v1
  with:
    require-shapes-changes: 'false'
```

### Treat realization refreshes as mechanical hygiene

Use this if your project does a lot of refactors and doesn't want an amendment for every file rename or summary tweak. This is a deliberate policy choice — the opposite default is also reasonable.

```yaml
- uses: shapes-fyi/shapes/.github/actions/shapes-validate@v1
  with:
    allow-realization-refresh: 'true'
```

### Custom shapes directory

```yaml
- uses: shapes-fyi/shapes/.github/actions/shapes-validate@v1
  with:
    shapes-dir: docs/shapes
```

### Pin to a specific shapes-cli release

```yaml
- uses: shapes-fyi/shapes/.github/actions/shapes-validate@v1
  with:
    shapes-version: v1.2.0
```

### Reuse a binary already on PATH

```yaml
- run: cargo install --locked --git https://github.com/shapes-fyi/shapes shapes-cli
- uses: shapes-fyi/shapes/.github/actions/shapes-validate@v1
  with:
    install-shapes: 'false'
```

## Running locally

The same checks work locally — `shapes ci-check --base origin/main` is the local equivalent of what this action runs in CI. Combine with `shapes validate` for a full pre-push gate.
