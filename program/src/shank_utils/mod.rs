use solana_program::instruction::Instruction;

/// Render the shank accounts of an instruction.
/// This is a stop gap until shank does the opposite, i.e. generate the instruction method
/// including the account metadata for each account from the attributes that this renders.
/// Namely the process is reversed.
/// Basically doing in Rust what solita already does for TypeScript.
pub trait RenderShankAccounts {
    fn render_shank_accounts(&self, attrs: &[(&str, &str)]) -> String;
}

impl RenderShankAccounts for Instruction {
    fn render_shank_accounts(&self, attrs: &[(&str, &str)]) -> String {
        let mut s = String::new();
        s.push_str("\n    #[rustfmt::skip]");
        for (idx, (account, (name, desc))) in
            self.accounts.iter().zip(attrs).enumerate()
        {
            s.push_str(&format!("\n    #[account({idx}, name = \"{name}\""));
            if account.is_writable {
                s.push_str(", mut");
            }
            if account.is_signer {
                s.push_str(", sig");
            }
            s.push_str(&format!(", desc=\"{desc}\")]"));
        }
        s
    }
}
