use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use valkyrie::proxy::asset::*;
use valkyrie::proxy::execute_msgs::*;
use valkyrie::proxy::query_msgs::*;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(Asset), &out_dir);
    export_schema(&schema_for!(AssetInfo), &out_dir);
    export_schema(&schema_for!(PairInfo), &out_dir);

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(SwapOperation), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(DexType), &out_dir);
    export_schema(&schema_for!(DexInfo), &out_dir);
    export_schema(&schema_for!(Cw20HookMsg), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);

    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(SimulateSwapOperationsResponse), &out_dir);
}