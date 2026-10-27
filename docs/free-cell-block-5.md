## BLOCK 5: Game Win Sequence & Unattended Loop Integration
Hook the completed FreeCell execution cycle into the existing unattended controller loop:
1. **Complete Win (`cards_banked == 52`):** After the final deliberate move triggers the cascade/victory animation, invoke the existing post-game win handler (wait for victory banner $\to$ trigger "New Game" / next deal sequence $\to$ return to Block 1 RECON at $T=0$).
2. **Best-Effort Termination (`cards_banked < 52`):** After executing all moves to bank the maximum possible cards into the Suit piles, log the final telemetry (`cards_banked`, `moves_executed`, `nodes_explored`) and invoke the existing reset / "New Game" handler for unattended continuous operation.
