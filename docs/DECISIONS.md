# Decisions waiting on the owner

Everything else from `docs/AUDIT_2026-09-09.md` is done. These three change
the licence, the dependency list or what is published, so they were not made
by an autonomous session until the owner said so on 2026-09-26.

## 1. Replace the `chess` crate

Decided 2026-09-26: cozy-chess 0.3.4. It keeps the MIT licence, has no
`unsafe` and no third-party dependencies, and generates moves about as fast
as shakmaty. Its one quirk, castling written as the king taking its rook, is
handled in `chess_core::moves`. What was weighed:

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

Decided 2026-09-26: rodio 0.22 with only its `playback` feature. The cues
are sine tones built in `app/sound.rs`, so no decoder and no sample files
ship; that leaves rodio, cpal and a handful of small crates. kira is a game
audio engine with mixing tracks and tweening, far more than five short
tones need. Sound is on by default, as on Lichess, and View > Sound turns
it off. A machine without an audio device plays nothing and says so in the
menu. Linux builds need `libasound2-dev`.

What was weighed:

`rodio` 0.22 or `kira` 0.12 (both MIT or Apache-2.0); either pulls `cpal`
and a few platform crates.

## 3. First release

Nothing has been tagged. To cut v0.1.0:

1. Rename `## Unreleased` in `CHANGELOG.md` to `## 0.1.0 - <date>`.
2. `git tag -a v0.1.0 -m "First release"` and push the tag to both remotes.
3. A GitHub release can attach the two release binaries from
   `cargo build --release` if you want downloads; CI does not build them.

The workspace version is already 0.1.0 in `Cargo.toml`.
