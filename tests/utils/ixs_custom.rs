use borsh::BorshSerialize;
use challenge::{
    challenge_id, ixs::ChallengeInstruction, state::Redeem, Solution,
};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[allow(clippy::too_many_arguments)]
#[allow(unused)] // it actually is in 01_create_challenge.rs
pub fn create_challenge_with_pda(
    payer: Pubkey,
    creator: Pubkey,
    id: String,
    admit_cost: u64,
    tries_per_admit: u8,
    solutions: Vec<Solution>,
    challenge_pda: Pubkey,
) -> Result<Instruction, ProgramError> {
    let (redeem, _) = Redeem::pda(&challenge_pda);
    let ix = Instruction {
        program_id: challenge_id(),
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(creator, false),
            AccountMeta::new(challenge_pda, false),
        ],
        data: ChallengeInstruction::CreateChallenge {
            id,
            admit_cost,
            tries_per_admit,
            redeem,
            solutions,
        }
        .try_to_vec()?,
    };

    Ok(ix)
}
