# Chess Smart Contract

A Solana smart contract for managing escrow accounts in chess games between two parties. This program enables secure deposit and withdrawal of SOL (Solana's native token) for competitive gaming scenarios.

## Overview

This Solana program implements an escrow system where two players can deposit SOL into a secure account. The funds are held in a Program Derived Address (PDA) until the game result is determined, at which point an admin can release the funds to the winner. The contract includes fee distribution mechanisms and ensures proper account management on the Solana blockchain.

## Features

- **Initialize Escrow**: Either the creator or competitor can create or join an escrow by depositing SOL
- **Withdraw Escrow**: After game completion, an admin can withdraw escrowed funds to the winner
- **Fee Distribution**: Automatic fee allocation (1.9% to fee account, 0.1% to admin, 98% to escrow)
- **Rent Exemption**: Ensures escrow accounts meet Solana's rent exemption requirements
- **Secure Storage**: Uses PDA (Program Derived Address) for secure fund custody
- **Access Control**: Validates that only authorized parties can interact with escrow accounts

## Architecture

### Program Structure

The program is built as a native Solana program using Rust:

- `src/entrypoint.rs` - Program entrypoint that receives and routes instructions
- `src/processor.rs` - Main business logic for processing instructions
- `src/instruction.rs` - Instruction definitions and serialization (InitEscrow, WithdrawEscrow)
- `src/state.rs` - Escrow account data structure and packing/unpacking logic
- `src/error.rs` - Custom error types for the program
- `src/lib.rs` - Module declarations and program entry point

### Account Structure

The Escrow state account contains:
- `is_initialized`: Boolean flag indicating if the escrow is active
- `creator_pubkey`: Public key of the escrow creator
- `competitor_pubkey`: Public key of the second party
- `amount`: Total amount of SOL deposited in the escrow

### Instructions

#### InitEscrow

Initializes or joins an escrow account. When a creator initializes (is_cretor = 1), their public key and deposit amount are recorded. When a competitor joins (is_cretor = 0), their public key is recorded and the amount is added to the existing escrow.

**Accounts Required:**
- Sender (signer)
- Fee account
- Admin account
- Escrow account
- PDA account
- Rent sysvar
- System program

**Fee Distribution:**
- 1.9% of deposit → Fee account
- 0.1% of deposit → Admin account
- 98% of deposit → PDA escrow account

#### WithdrawEscrow

Allows an admin to withdraw funds from the escrow to the winner. The result parameter (0 or 1) determines the payout recipient, and the amount must match the total escrow amount.

**Accounts Required:**
- Admin account (signer)
- Escrow account
- PDA account
- System program
- Creator account
- Competitor account (if result = 1)
- Withdraw account (recipient)

**Withdrawal Amount:**
- 98% of total escrow amount → Winner account

## Technical Details

### Fee Structure

- **Fee Account**: Receives 1.9% of each deposit
- **Admin Account**: Receives 0.1% of each deposit
- **Escrow Account**: Holds 98% of each deposit in the PDA

### PDA Derivation

The Program Derived Address is derived using the seed "chess" and the program ID, ensuring secure and deterministic account generation.

### Error Handling

The program includes custom error types:
- InvalidInstruction
- NotRentExempt
- ExpectedAmountMismatch
- AmountOverflow
- InvalidAccount
- InvalidAmount
- InvalidOwner
- InvalidPdaSeeds

## Development

### Prerequisites

- Rust (latest stable version)
- Solana CLI tools
- Node.js and npm/yarn
- Anchor framework (for testing)

### Building

```bash
# Build the program
anchor build
```

### Testing

```bash
# Run tests
anchor test
```

### Deployment

The program is configured for localnet deployment. Update `Anchor.toml` for different clusters (devnet, mainnet).

## Project Structure

```
chess-smart-contract/
├── programs/
│   └── chess-smart-contract/
│       └── src/
│           ├── entrypoint.rs    # Program entrypoint
│           ├── processor.rs     # Instruction processing logic
│           ├── instruction.rs   # Instruction definitions
│           ├── state.rs         # Escrow state structure
│           ├── error.rs         # Custom error types
│           └── lib.rs           # Module declarations
├── tests/
│   └── chess-smart-contract.ts  # TypeScript test suite
├── migrations/
│   └── deploy.ts                # Deployment script
├── Anchor.toml                   # Anchor configuration
├── Cargo.toml                   # Rust workspace configuration
└── package.json                 # Node.js dependencies
```

## Security Considerations

- Only the admin account can execute withdrawals
- Escrow accounts must be rent-exempt
- Amount validation ensures correct withdrawal amounts
- Account ownership is verified before operations
- PDA ensures secure fund custody

## License

ISC
