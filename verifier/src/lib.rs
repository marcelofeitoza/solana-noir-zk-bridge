use pinocchio::{
    account_info::AccountInfo, entrypoint, msg, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let proof = parse_proof(instruction_data)?;
    let public_inputs = parse_public_inputs(instruction_data)?;
    let vk = parse_verification_key(instruction_data)?;

    let prepared_inputs = prepare_public_inputs(&public_inputs, &vk)?;
    verify_proof(&proof, &prepared_inputs, &vk)?;

    msg!("Proof verified successfully!");
    Ok(())
}

struct VerificationKey {
    vk_alpha_g1: [u8; 64],
    vk_beta_g2: [u8; 128],
    vk_gamma_g2: [u8; 128],
    vk_delta_g2: [u8; 128],
    vk_ic: Vec<[u8; 64]>,
}

struct NoirProof {
    proof_a: [u8; 64],
    proof_b: [u8; 128],
    proof_c: [u8; 64],
}

struct PublicInputs(Vec<[u8; 32]>);

fn parse_proof(data: &[u8]) -> Result<NoirProof, ProgramError> {
    if data.len() < 256 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let proof_a = data[0..64]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let proof_b = data[64..192]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let proof_c = data[192..256]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    Ok(NoirProof {
        proof_a,
        proof_b,
        proof_c,
    })
}

fn parse_public_inputs(data: &[u8]) -> Result<PublicInputs, ProgramError> {
    if data.len() < 256 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let public_inputs_start = 256;
    let public_inputs = data[public_inputs_start..]
        .chunks(32)
        .map(|chunk| {
            chunk
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)
        })
        .collect::<Result<Vec<[u8; 32]>, ProgramError>>()?;

    Ok(PublicInputs(public_inputs))
}

fn parse_verification_key(data: &[u8]) -> Result<VerificationKey, ProgramError> {
    let vk_start = 256 + data[256..].len() / 32 * 32; // Calculate offset after proof and inputs
    if data.len() < vk_start + 448 {
        // VK components size: 64 + 128 + 128 + 128
        return Err(ProgramError::InvalidInstructionData);
    }

    let vk_alpha_g1 = data[vk_start..vk_start + 64]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let vk_beta_g2 = data[vk_start + 64..vk_start + 192]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let vk_gamma_g2 = data[vk_start + 192..vk_start + 320]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let vk_delta_g2 = data[vk_start + 320..vk_start + 448]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // Extract IC points
    let vk_ic_start = vk_start + 448;
    let vk_ic = data[vk_ic_start..]
        .chunks(64)
        .map(|chunk| {
            chunk
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)
        })
        .collect::<Result<Vec<[u8; 64]>, ProgramError>>()?;

    Ok(VerificationKey {
        vk_alpha_g1,
        vk_beta_g2,
        vk_gamma_g2,
        vk_delta_g2,
        vk_ic,
    })
}

fn prepare_public_inputs(
    inputs: &PublicInputs,
    vk: &VerificationKey,
) -> Result<[u8; 64], ProgramError> {
    if inputs.0.len() + 1 != vk.vk_ic.len() {
        return Err(ProgramError::InvalidInstructionData);
    }

    let mut prepared_inputs = vk.vk_ic[0];
    for (i, input) in inputs.0.iter().enumerate() {
        let mul_res = solana_program::alt_bn128::prelude::alt_bn128_multiplication(
            &[&vk.vk_ic[i + 1][..], &input[..]].concat(),
        )
        .map_err(|_| ProgramError::InvalidInstructionData)?;

        prepared_inputs = solana_program::alt_bn128::prelude::alt_bn128_addition(
            &[&mul_res[..], &prepared_inputs[..]].concat(),
        )
        .map_err(|_| ProgramError::InvalidInstructionData)?[..]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?;
    }

    Ok(prepared_inputs)
}

fn verify_proof(
    proof: &NoirProof,
    prepared_inputs: &[u8; 64],
    vk: &VerificationKey,
) -> ProgramResult {
    let pairing_input = [
        proof.proof_a.as_slice(),
        proof.proof_b.as_slice(),
        prepared_inputs.as_slice(),
        vk.vk_gamma_g2.as_slice(),
        proof.proof_c.as_slice(),
        vk.vk_delta_g2.as_slice(),
        vk.vk_alpha_g1.as_slice(),
        vk.vk_beta_g2.as_slice(),
    ]
    .concat();

    let pairing_res = solana_program::alt_bn128::prelude::alt_bn128_pairing(&pairing_input)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    if pairing_res[31] != 1 {
        return Err(ProgramError::Custom(0));
    }

    Ok(())
}
