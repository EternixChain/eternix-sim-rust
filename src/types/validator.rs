#[derive(Debug, Clone)]
pub struct Validator {
    pub id: u64,
    pub state: ValidatorState,
    pub vault_balance: u128,
    pub initial_bond: u128,
    pub miss_counter: u32,
    pub double_sign_offenses: u8,
    pub cooldown_until_epoch: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidatorState {
    Active,
    PausedLowVault,
    PunishedCooldown,
    Inactive,
    Jailed,
}
