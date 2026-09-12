# Decisions waiting on the owner

Everything else from `docs/AUDIT_2026-09-09.md` is done. These three change
the licence, the dependency list or what is published, so they are not made
by an autonomous session.

## 1. Replace the `chess` crate

`chess` 3.2.0 is unmaintained and pulls `failure` (RUSTSEC-2019-0036,
RUSTSEC-2020-0036) and a build-only `rand` 0.7. `cargo audit` and
`cargo deny` carry four ignores for it. Everything in chess-core builds on
its `Board`, `ChessMove` and `MoveGen`, and the GUI uses its `Square`,
`Piece` and `Color` directly, so the swap touches every crate.

| Option | Licence | What changes |
|---|---|---|
| shakmaty 0.30 | GPL-3.0-or-later | The project becomes GPL. Brings its own SAN, FEN, repetition and insufficient-material code, so `notation.rs` and half of `game.rs` could go. |
| cozy-chess 0.3 | MIT | Licence stays. Move generation only; SAN, PGN, draw rules and the opening lookup stay as they are. Types differ in shape (`Square`, `Piece`, `Color` are plain enums, moves carry no "is capture" flag), so the port is mechanical but wide. |
| Keep `chess` | MIT | Nothing to do; the ignores stay honest as long as `failure` is only reached through `Board::from_str` errors. |

Recommendation if the ignores bother you: cozy-chess. If GPL is acceptable,
shakmaty removes the most code.

## 2. Sound

A move click and a low-time tick need an audio dependency. `rodio` 0.22 or
`kira` 0.12 (both MIT or Apache-2.0); either pulls `cpal` and a few
platform crates. Off by default behind a setting, one short sample compiled
in. Nothing in the code waits on this.

## 3. First release

Nothing has been tagged. To cut v0.1.0:

1. Rename `## Unreleased` in `CHANGELOG.md` to `## 0.1.0 - <date>`.
2. `git tag -a v0.1.0 -m "First release"` and push the tag to both remotes.
3. A GitHub release can attach the two release binaries from
   `cargo build --release` if you want downloads; CI does not build them.

The workspace version is already 0.1.0 in `Cargo.toml`.
