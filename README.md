# Asset Version Control

A local-first desktop app for version-controlling large binary asset files —
initially scoped to 3D assets — with independent per-file version histories,
built on top of Git, with no server, sync, or cloud component.

## Why

Git is excellent at versioning text-based source code, but its model
assumes one shared project-wide commit history. This tool layers an
independent, per-asset versioning scheme on top of a single Git repository,
so that each tracked file gets its own version sequence, commit log, and
tag history — fully inspectable with plain `git log` / `git tag`, with or
without this app installed.

This is a personal tool, built primarily for solo use, and open-sourced as
a side effect rather than as a collaborative project.

## Status

🚧 In active architecture/development. Currently past Phase 1 (Architecture
& Tech Stack Selection); environment setup and core implementation are in
progress.

## Core Features

- Per-file, independent version history (`vX.X.XXXX`), with automatic patch
  increment unless manually overridden.
- A single shared Git repository as the underlying store — no per-file
  repositories.
- Robust rename/move detection that survives files being relocated within
  the project folder, using live filesystem watching plus a similarity- and
  time-based fallback heuristic for missed events.
- Per-asset Markdown logs (version, author, timestamp, commit hash, commit
  messages) stored flat in `.logs/`, fully rebuildable from Git history.
- Paste-to-attach images in commit detail messages, stored in `.image/`
  with stable, auto-generated filenames.
- A two-tier in-app terminal: guided commit/version workflow buttons for
  everyday use, plus a full interactive terminal for arbitrary `git`/`gh`
  commands.
- Built-in `.gitignore` / Git config editor.
- Portable: moving the project's root folder does not break tracking.

## Requirements

- [Git](https://git-scm.com/) installed locally.
- macOS or Linux. (See `TECH_STACK.md` for platform notes — Linux is a
  primary target but not yet directly tested by the maintainer.)

## Tech Stack

See [`TECH_STACK.md`](./TECH_STACK.md) for the full breakdown. In short:
**Tauri** (Rust core + native OS webview) with an HTML/CSS/JS-TS frontend,
**SQLite** (via `rusqlite`) as a fast-lookup cache, and the user's local
**Git** installation as the underlying version control engine.

## Architecture

See [`ARCHITECTURE.md`](./ARCHITECTURE.md) for the full design, including
the identity model, versioning strategy, rename/move detection logic, and
data recoverability guarantees.

## File Structure

See [`FILE_STRUCTURE.md`](./FILE_STRUCTURE.md) for the on-disk layout of a
tracked project and the naming conventions used throughout.

## Scope

This project is intentionally scoped:
- Local-only — no cloud backup or sync built in.
- Built for personal use first; not designed around hypothetical
  collaborators, despite being open source.
- 3D-asset-focused for now, though the underlying data model is kept
  generic enough to extend to other binary asset types later without a
  rework.
- Multi-window/multi-project support and a background-daemon mode are
  deferred until after the core single-project system is complete.

## License

_TBD._
