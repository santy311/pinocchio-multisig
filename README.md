# Pinocchio Multisig

A decentralized multisignature wallet implementation built on the Pinocchio blockchain framework. This project provides a secure and flexible multisig solution for managing digital assets with configurable approval thresholds and member management.

## Overview

Pinocchio Multisig is a smart contract that enables multiple parties to jointly control digital assets through a consensus mechanism. It supports:

- **Configurable thresholds**: Set approval and veto thresholds for proposals
- **Member management**: Add and remove multisig members dynamically
- **Proposal system**: Create, vote on, and execute proposals
- **Time-based expiry**: Automatic proposal expiration for security
- **Vault integration**: Secure asset storage with dedicated vault accounts

## Features

### Core Functionality

- **Multisig Creation**: Initialize a new multisig with configurable parameters
- **Member Management**: Add and remove members with admin privileges
- **Proposal System**: Create proposals that require member consensus
- **Voting Mechanism**: Support for yes/no/veto votes with configurable thresholds
- **Execution**: Execute approved proposals through the vault system
- **Configuration**: Modify multisig parameters (thresholds, expiry duration)

### Security Features

- **PDA (Program Derived Address)**: Secure account derivation for state and vault
- **Threshold-based approval**: Require minimum votes for proposal execution
- **Veto mechanism**: Allow members to veto proposals with sufficient votes
- **Time-based expiry**: Automatic proposal expiration to prevent stale proposals
- **Admin controls**: Restricted member management operations

## Architecture

### State Management

- **Multisig**: Core multisig configuration and state
- **Proposal**: Individual proposal data and voting status
- **Member**: Member information and permissions
- **Vault**: Secure asset storage account

### Instructions

1. **Init Multisig**: Create a new multisig wallet
2. **Add Member**: Add new members to the multisig
3. **Remove Member**: Remove existing members
4. **Create Proposal**: Create a new proposal for voting
5. **Vote**: Cast votes on proposals (yes/no/veto)
6. **Execute**: Execute approved proposals
7. **Modify Config**: Update multisig configuration

## Getting Started

### Prerequisites

- Rust toolchain
- Pinocchio framework dependencies
- Solana CLI tools (for testing)

### Building

```bash
# Build the program
cargo build-sbf

# Run tests
cargo test -- --nocapture
```

### Program ID

```
7ut7NJGp4uGXavSNHxgdVDRyiEkVgk95uWMEcuNaDW7c
```

## Usage

### Creating a Multisig

1. Initialize a new multisig with desired parameters:
   - Threshold: Minimum votes required for approval
   - Number of members: Initial member count
   - Max expiry duration: Maximum proposal lifetime
   - Veto threshold: Minimum votes required for veto

### Managing Members

- **Add Member**: Existing members can add new members
- **Remove Member**: Admin can remove members (requires proper permissions)

### Proposal Workflow

1. **Create Proposal**: Any member can create a proposal
2. **Vote**: Members cast their votes (yes/no/veto)
3. **Execute**: Approved proposals can be executed through the vault
4. **Expiry**: Proposals automatically expire after the configured duration

## Development

### Project Structure

```
src/
├── entrypoint.rs          # Program entry point
├── error.rs              # Error definitions
├── instruction/          # Instruction handlers
│   ├── add_member.rs
│   ├── create_proposal.rs
│   ├── execute.rs
│   ├── init_multisig.rs
│   ├── modify_config.rs
│   ├── remove_member.rs
│   └── vote.rs
├── state/               # State structures
│   ├── member.rs
│   ├── multisig.rs
│   ├── proposal.rs
│   ├── vault.rs
│   └── utils.rs
└── lib.rs              # Main library file
```

### Testing

The project includes comprehensive unit tests covering:

- Multisig initialization and configuration
- Member management operations
- Proposal creation and voting
- Execution workflows
- Edge cases and error conditions

Run tests with:

```bash
cargo test -- --nocapture
```

## Security Considerations

- **Threshold Configuration**: Carefully consider approval and veto thresholds
- **Member Management**: Regularly review and update member lists
- **Proposal Expiry**: Set appropriate expiry durations for your use case
- **Vault Security**: Ensure proper access controls for vault operations

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

Built with the [Pinocchio framework](https://github.com/anza-xyz/pinocchio) for secure blockchain development.
