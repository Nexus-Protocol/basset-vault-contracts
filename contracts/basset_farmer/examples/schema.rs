use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use yield_optimizer::basset_farmer::{
    AnyoneMsg, ChildContractsInfoResponse, ConfigResponse, Cw20HookMsg, ExecuteMsg, GovernanceMsg,
    InstantiateMsg, IsRewardsClaimableResponse, QueryMsg, RebalanceResponse, YourselfMsg,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(YourselfMsg), &out_dir);
    export_schema(&schema_for!(AnyoneMsg), &out_dir);
    export_schema(&schema_for!(GovernanceMsg), &out_dir);
    export_schema(&schema_for!(Cw20HookMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(RebalanceResponse), &out_dir);
    export_schema(&schema_for!(ChildContractsInfoResponse), &out_dir);
    export_schema(&schema_for!(IsRewardsClaimableResponse), &out_dir);
}
