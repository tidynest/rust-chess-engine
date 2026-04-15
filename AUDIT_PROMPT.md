# Comprehensive Codebase Audit Prompt

I need you to use several agents and go through the ENTIRE codebase and documentation of this project (chess-engine), analyse it, and identify every way it could and should be improved and fixed. Be vigilant while doing your analysis — look for bugs, architectural issues, documentation inconsistencies, missing error handling, performance bottlenecks, and anything that could be working incorrectly or suboptimally.

I don't wish that you change anything in the code, no git commands that would change the online repos, and that you do this job fully autonomously, without my input, or that you need my approval. Make a backup of the project the first thing you do, and make sure that you can complete this task in the way that I have described.

The end result should be a report of all findings and a plan on how to solve all issues, saved to `docs/COMPREHENSIVE_AUDIT_REPORT.md`.

DO NOT START TO MAKE ANY CHANGES AT ALL! For that I wish to be present.

## Analysis Areas (use parallel agents for each)

1. **Chess Core Logic** — Trace the move generation, validation, and game state management. Are all piece movements correct (including en passant, castling, promotion)? Are check, checkmate, and stalemate detected properly? Any illegal moves accepted or legal moves rejected?

2. **Engine/Search** — Analyse the search algorithm (minimax, alpha-beta, etc.). Is evaluation correct? Are there search bugs, horizon effects, or incorrect pruning? How does time management work?

3. **Desktop/UI** — If there's a GUI (chess-desktop crate), check the full user flow. Does the board render correctly? Are moves input properly? Does state sync between engine and display?

4. **Build System & Tests** — Does it compile cleanly? Do all tests pass? Are there untested critical paths (e.g., edge-case positions, perft tests)? Check clippy, feature flags, and CI/CD.

5. **Documentation** — Are docs accurate? Version numbers consistent? Any contradictions between README, CHANGELOG, and actual code?

Good luck!
