use challenge::{ixs, shank_utils::RenderShankAccounts};
use solana_program::pubkey::Pubkey;

const PAYER_DESC: &str = "pays for the transaction";
const CREATOR_DESC: &str = "challenge authority";
const CHALLENGE_PDA_DESC: &str = "PDA for the challenge";
const CHALLENGER_DESC: &str =
    "challenger account which receives the redeemed token";
const CHALLENGER_PDA_DESC: &str = "PDA for the challenger";

const REDEEM_PDA_DESC: &str = "PDA of token to redeem for correct solution";
const REDEEM_ATA_DESC: &str = "ATA for redeem PDA and challenger";

fn main() {
    {
        let ix = ixs::create_challenge(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            "id".to_string(),
            0,
            0,
            vec![],
        )
        .unwrap();

        eprintln!(
            "{}\n    CreateChallenge {{",
            ix.render_shank_accounts(&[
                ("payer", PAYER_DESC),
                ("creator", CREATOR_DESC),
                ("challenge_pda", CHALLENGE_PDA_DESC),
                ("redeem_pda", REDEEM_PDA_DESC),
                ("token_program", "Token Program"),
                ("system_program", "System Program")
            ])
        );
    }
    {
        let ix = ixs::add_solutions(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            "id".to_string(),
            vec![],
        )
        .unwrap();
        eprintln!(
            "{}\n    AddSolutions {{",
            ix.render_shank_accounts(&[
                ("payer", PAYER_DESC),
                ("creator", CREATOR_DESC),
                ("challenge_pda", CHALLENGE_PDA_DESC),
                ("system_program", "System Program")
            ])
        );
    }
    {
        let ix = ixs::start_challenge(Pubkey::new_unique(), "id".to_string())
            .unwrap();
        eprintln!(
            "{}\n    StartChallenge {{",
            ix.render_shank_accounts(&[
                ("creator", CREATOR_DESC),
                ("challenge_pda", CHALLENGE_PDA_DESC),
            ])
        );
    }
    {
        let ix = ixs::admit_challenger(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            "id",
            Pubkey::new_unique(),
        )
        .unwrap()
        .ix;
        eprintln!(
            "{}\n    AdmitChallenger {{",
            ix.render_shank_accounts(&[
                ("payer", PAYER_DESC),
                ("creator", CREATOR_DESC),
                ("challenge_pda", CHALLENGE_PDA_DESC),
                ("challenger", CHALLENGER_DESC),
                ("challenger_pda", CHALLENGER_PDA_DESC),
                ("system_program", "System Program")
            ])
        );
    }
    {
        let ix = ixs::redeem(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            "id",
            Pubkey::new_unique(),
            "solution",
        )
        .unwrap();
        eprintln!(
            "{}\n    Redeem {{",
            ix.render_shank_accounts(&[
                ("payer", PAYER_DESC),
                ("challenge_pda", CHALLENGE_PDA_DESC),
                ("challenger", CHALLENGER_DESC),
                ("challenger_pda", CHALLENGER_PDA_DESC),
                ("redeem", REDEEM_PDA_DESC),
                ("redeem_ata", REDEEM_ATA_DESC),
                ("token_program", "Token Program"),
                ("associated_token_program", "Associated Token Program"),
                ("system_program", "System Program")
            ])
        );
    }
}
