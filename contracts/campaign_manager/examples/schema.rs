use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use valkyrie::campaign_manager::execute_msgs::*;
use valkyrie::campaign_manager::query_msgs::*;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ReferralRewardLimitOptionMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);
    export_schema(&schema_for!(CampaignInstantiateMsg), &out_dir);

    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(ReferralRewardLimitOptionResponse), &out_dir);
    export_schema(&schema_for!(CampaignResponse), &out_dir);
    export_schema(&schema_for!(CampaignsResponse), &out_dir);
}