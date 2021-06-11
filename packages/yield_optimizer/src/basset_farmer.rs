use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub token_code_id: u64,
    //address for bLuna token, for example
    pub basset_token_addr: String,
    //Luna / ETH / Sol, will be converted to cLuna, cETH, cSol
    pub collateral_token_symbol: String,
    //Nexus overseer addr
    pub overseer_addr: String,
    pub governance_addr: String,
    pub custody_basset_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Anyone { anyone_msg: AnyoneMsg },
    OverseerMsg { overseer_msg: OverseerMsg },
    Receive(Cw20ReceiveMsg),
    Yourself { yourself_msg: YourselfMsg }, // WithdrawTokens {
                                            //     depositor: Addr,
                                            //     amount: Uint256,
                                            // }
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum YourselfMsg {
    AfterBorrow {
        borrowed_amount: Uint256,
        buffer_size: Uint256,
    },
    AfterAterraRedeem {
        repay_amount: Uint256,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnyoneMsg {
    Rebalance {},
    Sweep {},
    SwapAnc {},
    BuyPsiTokens {},
    DisributeRewards {},
    ClaimRewards {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OverseerMsg {
    Deposit { farmer: String, amount: Uint256 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Deposit {},
    Withdraw {},
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub governance_contract: String,
    pub overseer_contract: String,
    pub custody_basset_contract: String,
    pub casset_token: String,
    pub basset_token: String,
}
