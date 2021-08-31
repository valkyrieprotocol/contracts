use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use valkyrie::campaign::enumerations::*;
use valkyrie::campaign::execute_msgs::*;
use valkyrie::campaign::query_msgs::*;
use valkyrie::campaign_manager::execute_msgs::CampaignInstantiateMsg;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(Referrer), &out_dir);

    export_schema(&schema_for!(CampaignInstantiateMsg), &out_dir);

    export_schema(&schema_for!(CampaignConfigMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(Cw20HookMsg), &out_dir);
    export_schema(&schema_for!(DistributeResult), &out_dir);
    export_schema(&schema_for!(ReferralReward), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);

    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(CampaignConfigResponse), &out_dir);
    export_schema(&schema_for!(RewardConfigResponse), &out_dir);
    export_schema(&schema_for!(CampaignStateResponse), &out_dir);
    export_schema(&schema_for!(ShareUrlResponse), &out_dir);
    export_schema(&schema_for!(GetAddressFromReferrerResponse), &out_dir);
    export_schema(&schema_for!(ReferralRewardLimitAmount), &out_dir);
    export_schema(&schema_for!(ActorResponse), &out_dir);
    export_schema(&schema_for!(ActorsResponse), &out_dir);
}