use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Wrong Configuration")]
    WrongConfig {},

    #[error("Coin already exists")]
    ExistCoin {},

    #[error("Coin does not exist")]
    InvalidCoin {},

    #[error("No Funds Needed")]
    NoFundsNeed {},

    #[error("You sent several coins")]
    SeveralCoinsSent {},

    #[error("Presale is not started")]
    PresaleNotStarted {},

    #[error("Presale is finished")]
    PresaleEnded {},

    #[error("Presale is not finished")]
    PresaleNotEnded {},

    #[error("You already claimed")]
    AlreadyClaimed {},

    #[error("There are no enough tokens as your demand")]
    NoEnoughTokens {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },
}
