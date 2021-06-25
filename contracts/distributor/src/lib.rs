pub mod entrypoints;

mod executions;
mod queries;
mod states;

#[cfg(test)]
mod testing;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points_with_migration!(contract);
