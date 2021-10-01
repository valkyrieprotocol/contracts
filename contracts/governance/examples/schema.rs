use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use valkyrie::governance::enumerations::*;
use valkyrie::governance::execute_msgs::*;
use valkyrie::governance::models::*;
use valkyrie::governance::query_msgs::*;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(VoteOption), &out_dir);
    export_schema(&schema_for!(PollStatus), &out_dir);

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ContractConfigInitMsg), &out_dir);
    export_schema(&schema_for!(PollConfigInitMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(Cw20HookMsg), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);

    export_schema(&schema_for!(VoteInfoMsg), &out_dir);

    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(ContractConfigResponse), &out_dir);
    export_schema(&schema_for!(StakingStateResponse), &out_dir);
    export_schema(&schema_for!(StakerStateResponse), &out_dir);
    export_schema(&schema_for!(PollConfigResponse), &out_dir);
    export_schema(&schema_for!(PollStateResponse), &out_dir);
    export_schema(&schema_for!(PollResponse), &out_dir);
    export_schema(&schema_for!(PollsResponse), &out_dir);
    export_schema(&schema_for!(PollCountResponse), &out_dir);
    export_schema(&schema_for!(VotersResponse), &out_dir);
    export_schema(&schema_for!(VotingPowerResponse), &out_dir);
}