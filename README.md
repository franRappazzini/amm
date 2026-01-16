# AMM - Automated Market Maker on Solana

An Automated Market Maker (AMM) protocol implemented on Solana using the Anchor framework. This project implements a liquidity pool system with token swapping based on the constant product formula (x \* y = k).

## 🚀 Features

- **Protocol Initialization**: Configuration of protocol and swap fees
- **Liquidity Pool Creation**: Create pools for token pairs
- **Liquidity Deposits**: Users can deposit tokens and receive LP tokens
- **Liquidity Withdrawals**: Redeem LP tokens for underlying tokens
- **Token Swaps**: Exchange tokens with slippage protection
- **Protocol Fees**: Fee system with withdrawal capability

## Auditware Radar audit

<img src="https://img.shields.io/github/actions/workflow/status/franRappazzini/amm/radar.yaml">

## 📋 Prerequisites

Before starting, make sure you have installed:

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (v1.17 or higher)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) (v0.32.1 or higher)
- [Node.js](https://nodejs.org/) (v18 or higher)
- [Yarn](https://yarnpkg.com/getting-started/install) (package manager)

### Verify installations

```bash
rustc --version
solana --version
anchor --version
node --version
yarn --version
```

## 🔧 Installation

### 1. Clone the repository

```bash
git clone git@github.com:franRappazzini/amm.git
cd amm
```

### 2. Install dependencies

```bash
yarn install
```

### 3. Configure Solana CLI

Set up your Solana wallet and make sure you're on the localnet network:

```bash
# Create a new wallet if you don't have one
solana-keygen new

# Set cluster to localnet
solana config set --url localhost
```

## 🧪 Local Testing

### 1. Start a local Solana validator

In a separate terminal, run:

```bash
solana-test-validator -r
```

Keep this terminal open while running tests.

### 2. Build the program

```bash
anchor build
```

### 3. Deploy the program locally

```bash
anchor deploy
```

### 4. Run the tests

Run all tests:

```bash
anchor test --skip-local-validator
```

Or use the yarn script:

```bash
yarn test
```

### Run specific tests

```bash
# Main test
yarn ts-mocha -p ./tsconfig.json -t 1000000 tests/amm-local.test.ts

# Decimals test
yarn ts-mocha -p ./tsconfig.json -t 1000000 tests/amm-decimals-local.test.ts

# Slippage test
yarn ts-mocha -p ./tsconfig.json -t 1000000 tests/amm-slippage-local.test.ts
```

## 📁 Project Structure

```
amm/
├── programs/
│   └── amm/
│       └── src/
│           ├── lib.rs              # Program entry point
│           ├── constants.rs        # Program constants
│           ├── errors.rs           # Custom errors
│           ├── instructions/       # Instruction logic
│           ├── states/             # Program data structures
│           └── utils/              # Helper functions
├── tests/
│   ├── amm-local.test.ts          # Main tests
│   ├── amm-decimals-local.test.ts # Decimals tests
│   ├── amm-slippage-local.test.ts # Slippage tests
│   └── utils/                      # Testing utilities
├── target/
│   └── idl/
│       └── amm.json               # Generated IDL
├── Anchor.toml                     # Anchor configuration
├── Cargo.toml                      # Rust configuration
├── package.json                    # Node.js dependencies
└── MATH.md                         # AMM mathematical documentation
```

## 🔑 Program Instructions

### Initialize

Initializes the protocol's global configuration with fees.

### Create Liquidity Pool

Creates a new liquidity pool for a token pair.

### Deposit Liquidity

Allows users to deposit tokens into a pool and receive LP tokens.

### Redeem LP

Allows users to burn LP tokens and receive the underlying tokens.

### Swap

Allows swapping one token for another with slippage protection.

### Withdraw Protocol Fees

Allows withdrawing accumulated protocol fees.

## 📐 Mathematical Model

The AMM uses the constant product formula:

```
x * y = L
```

Where:

- `x` = amount of token A in the pool
- `y` = amount of token B in the pool
- `L` = liquidity constant

For more details about the formulas and calculations, see [MATH.md](MATH.md).

## 🛠️ Development

### Linting

```bash
# Check formatting
yarn lint

# Auto-fix formatting
yarn lint:fix
```

### Build program only

```bash
anchor build
```

### View program logs

```bash
solana logs
```

## 🔍 Troubleshooting

### Validator won't start

```bash
# Clean the ledger and restart
solana-test-validator --reset
```

### Build errors

```bash
# Clean and rebuild
anchor clean
anchor build
```

### Tests fail

```bash
# Make sure the validator is running
# Verify the program is deployed
solana program show 92NnZLZ8TS5Ay1UwAnQmtbYWbAFcEWtZcn7MwVkLhhMZ
```

## 📄 License

ISC

## 🤝 Contributing

Contributions are welcome. Please open an issue or pull request for suggestions or improvements.

---

**Note**: This project is configured to work on `localnet` by default. To deploy on devnet or mainnet, modify the configuration in `Anchor.toml`.
