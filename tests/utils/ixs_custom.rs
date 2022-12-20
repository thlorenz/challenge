use borsh::BorshSerialize;
use challenge::{challenge_id, ixs::ChallengeInstruction};
use solana_program::{
    hash::HASH_BYTES,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn create_challenge_with_pda(
    payer: Pubkey,
    creator: Pubkey,
    admit_cost: u64,
    tries_per_admit: u8,
    redeem: Pubkey,
    solutions: Vec<[u8; HASH_BYTES]>,
    max_solutions: Option<u8>,
    challenge_pda: Pubkey,
) -> Result<Instruction, ProgramError> {
    let ix = Instruction {
        program_id: challenge_id(),
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(creator, false),
            AccountMeta::new(challenge_pda, false),
        ],
        data: ChallengeInstruction::CreateChallenge {
            admit_cost,
            tries_per_admit,
            redeem,
            solutions,
            max_solutions,
        }
        .try_to_vec()?,
    };

    Ok(ix)
}
