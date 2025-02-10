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
