# Architecture

Three crates, one process, one child process.

```
chess-desktop (egui GUI, CLI)
   |            \
chess-core       chess-engine  ---- stdin/stdout ----> stockfish
(board, history, (UCI client)
 SAN, PGN)
```

## chess-core

`GameHistory` is the game: the start board, every move played, the board
after each, and the SAN of each move computed when it was played. Undo and
redo move an index; a new move truncates what lies ahead. It also answers
draw questions (repetition from the stored boards, the fifty-move rule from
a halfmove clock it derives, insufficient material from bitboards), reads
and writes PGN, and gives the UCI move list the engine needs.

`notation` formats SAN with disambiguation and parses it by matching the
formatter against the legal moves. `ChessEngine` and `GameState` wrap a
single board for the CLI. Move generation and legality come from the
`chess` crate throughout.

## chess-engine

`StockfishEngine` spawns the binary with piped stdin and stdout and runs a
writer task and a reader task on tokio. It knows nothing about the GUI. Its
parser turns stdout lines into `EngineResponse`: search info with an exact
`Score` (centipawns or mate distance, from the side to move) and the line's
number under MultiPV, a best move, or nothing for lines that carry no score.

## chess-desktop

`ChessApp` holds one `GameHistory`, the selection and drag state, the engine
link and the theme. The board is always read from the history; there is no
second copy. The engine's lines sit in `engine_lines`, best first; the eval
bar reads the first, the board draws an arrow for each.

The engine thread lives in `app/engine_link.rs`. The UI sends
`EngineCommand`s (search, stop, new game, quit) over a tokio channel and
receives `EngineEvent`s over a std channel; the thread asks egui to repaint
after each event. Every search carries an id. The UI bumps the id and sends
Stop whenever the position changes, and drops any reply with an older id,
so a move found for a position the user has left is never played. A search
is either for play, whose best move is applied, or for analysis, whose best
move is only shown.

`app/engine_comm.rs` is the UI side of that link and the place where moves
are played: `play_move` is the single entry for human and engine moves, and
`position_changed` clears the selection and aborts the running search.

The panels under `ui/` draw from `ChessApp` and the theme tokens in
`ui/theme.rs`. Pieces are shape drawings in `ui/pieces.rs`, one triangulated
silhouette per piece. Settings live in `app/settings.rs` as `key = value`
lines in the user's config directory; saved games are PGN files in the data
directory, written and listed by `app/games.rs`.

## From a click to a reply

1. `board.rs` maps the pointer to a square and finds the legal move for the
   selected pair, asking for a promotion piece when there are several.
2. `play_move` records it in the history, which computes the SAN, then
   `position_changed` aborts any search and clears the selection.
3. On the next frame `auto_request` sees it is the computer's turn on the
   live line and sends a search with a fresh id and the full move list.
4. The engine thread stops any previous search, waits for its bestmove,
   sets the position and starts the new search, forwarding tagged replies.
5. `poll_engine_responses` updates the evaluation from info lines and, on
   bestmove with the current id, returns the move, which `play_move` plays.
