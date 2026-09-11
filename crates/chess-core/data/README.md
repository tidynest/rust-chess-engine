# Opening names

`openings.tsv` is the Lichess opening table, the five files `a.tsv` to
`e.tsv` of <https://github.com/lichess-org/chess-openings> joined under one
header, taken at commit `4b8622759e7a` (2026-08-04). Lichess releases the
data under the CC0 1.0 Public Domain Dedication.

Columns: ECO code, name, and the moves as PGN. `chess_core::openings` keys
the rows by their moves without numbers and reports the longest row a game
begins with.

To refresh:

```bash
for f in a b c d e; do curl -sfL -o /tmp/$f.tsv https://raw.githubusercontent.com/lichess-org/chess-openings/master/$f.tsv; done
(head -1 /tmp/a.tsv; tail -q -n +2 /tmp/[a-e].tsv) > crates/chess-core/data/openings.tsv
```
