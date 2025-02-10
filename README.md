# Zero-Knowledge Proof Verifier for Solana

## Overview

This project demonstrates a zero-knowledge proof system integration with Solana, using Noir for circuit definition and a custom Solana program for proof verification.

## Architecture

### 1. Circuit (Noir)

The circuit implements a simple arithmetic proof:

-   Takes two inputs: `x` (private) and `y` (public)
-   Verifies that their sum equals 42
-   Returns true if the assertion passes

```rust
fn main(x: u32, y: pub u32) -> pub bool {
    let z = x + y;
    assert(z == 42);
    true
}
```

### 2. Verifier (Solana Program)

A Solana program that verifies Noir-generated zero-knowledge proofs using:

-   BN128 elliptic curve operations
-   Groth16 proof system verification
-   On-chain verification of public inputs

Key components:

-   Proof parsing and validation
-   Verification key handling
-   Public input preparation
-   Final proof verification using pairing checks

## Setup

1. Install Dependencies

```bash
# Install Noir
curl -L https://raw.githubusercontent.com/noir-lang/installables/main/install.sh | bash

# Install Solana Tool Suite
sh -c "$(curl -sSfL https://release.solana.com/v1.17.14/install)"
```

2. Build Circuit

```bash
cd circuit
nargo check
```

3. Build Verifier

```bash
cd verifier
cargo build-bpf
```

## Usage

1. Generate Proof

```bash
cd circuit
nargo prove
```

2. Deploy Verifier

```bash
solana program deploy dist/solana_zk_verifier.so
```

3. Verify Proof

```bash
cargo-test-sbf
```

## Testing

The project includes tests for both components:

1. Circuit Test (Noir):

```rust
#[test]
fn test_main() {
    let result = main(23, 19);
    assert(result == true);
}
```

2. Verifier Test (Solana):

```rust
#[cfg(test)]
mod test {
    use mollusk_svm::{result::Check, Mollusk};
    use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

    #[test]
    fn test_noir_zk_verifier() {
        let program_id_keypair_bytes = std::fs::read("dist/solana_zk_verifier-keypair.json")
            .unwrap()[..32]
            .try_into()
            .expect("slice with incorrect length");
        let program_id = Pubkey::new_from_array(program_id_keypair_bytes);
        let mollusk = Mollusk::new(&program_id, "dist/solana_zk_verifier");

        let x: u32 = 23;
        let y: u32 = 19;
        let instruction_data = [x.to_le_bytes(), y.to_le_bytes()].concat();

        let instruction = Instruction::new_with_bytes(program_id, &instruction_data, vec![]);

        let result =
            mollusk.process_and_validate_instruction(&instruction, &[], &[Check::success()]);

        assert!(
            !result.program_result.is_err(),
            "Program execution failed: {:?}",
            result.program_result
        );

        println!("Compute Units: {}", result.compute_units_consumed);
    }
}
```

## Technical Details

### Proof Structure

```rust
struct NoirProof {
    proof_a: [u8; 64],    // G1 point
    proof_b: [u8; 128],   // G2 point
    proof_c: [u8; 64]     // G1 point
}
```

### Verification Key

```rust
struct VerificationKey {
    vk_alpha_g1: [u8; 64],
    vk_beta_g2: [u8; 128],
    vk_gamma_g2: [u8; 128],
    vk_delta_g2: [u8; 128],
    vk_ic: Vec<[u8; 64]>
}
```

## Security Considerations

-   The verifier implements robust input validation
-   Uses Solana's native BN128 precompiles for cryptographic operations
-   Follows zero-knowledge proof verification best practices
