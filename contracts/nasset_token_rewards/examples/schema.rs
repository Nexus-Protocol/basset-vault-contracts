use std::env::current_dir;
use std::fs::create_dir_all;

use basset_vault::nasset_token_rewards::{
    AccruedRewardsResponse, AnyoneMsg, ConfigResponse, ExecuteMsg, GovernanceMsg, HolderResponse,
    HoldersResponse, InstantiateMsg, MigrateMsg, QueryMsg, StateResponse, TokenMsg,
};
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(AnyoneMsg), &out_dir);
    export_schema(&schema_for!(GovernanceMsg), &out_dir);
    export_schema(&schema_for!(TokenMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(StateResponse), &out_dir);
    export_schema(&schema_for!(AccruedRewardsResponse), &out_dir);
    export_schema(&schema_for!(HolderResponse), &out_dir);
    export_schema(&schema_for!(HoldersResponse), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);
}
