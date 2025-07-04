# Chess game smart contract

A Solana smart contract for Chess game between two parties, designed for use cases like chess or other competitive games.

## Overview

This program allows two users to securely deposit SOL into an escrow account, with funds released to the winner (or split as needed) after the result is determined. The contract handles fee distribution, rent exemption, and ensures only valid parties can interact with the escrow.

## Contact

If you have any quesion, contact here: [Telegram](https://t.me/shiny0103) | [Twitter](https://x.com/0xTan1319)

## Features

- **Initialize Escrow:** Either party can create or join an escrow by depositing SOL.
- **Withdraw Escrow:** After the result is determined, the winner (or both parties) can withdraw the escrowed funds.
- **Fee Distribution:** A small fee is distributed to admin and fee accounts, with the majority of funds held in escrow.
- **Rent Exemption:** Ensures escrow accounts are rent-exempt on Solana.

## Directory Structure

- `src/processor.rs` — Main logic for processing instructions and handling escrow state.
- `src/instruction.rs` — Defines the instructions (init, withdraw) and their serialization.
- `src/state.rs` — Escrow account structure and packing/unpacking logic.
- `src/error.rs` — Custom error types for the program.
- `src/entrypoint.rs` — Solana program entrypoint.

## Accounts

- **Creator:** The user who initializes the escrow.
- **Competitor:** The second party joining the escrow.
- **Admin/Fee Accounts:** Receive a small portion of the wager as fees.
- **PDA Account:** Holds the escrowed funds securely.
