mod agents;
mod maps;
mod offsets;
mod structs;
mod threads;
mod valorant_utils;
mod weapons;

use std::{collections::HashSet, time::Duration};

use crate::{
    driver::{MemoryContext, ProcessContext},
    utils,
    valorant::structs::{GameEngine, ValorantContext},
};

#[derive(Default, Clone)]
pub struct RadarJSON(pub serde_json::Value);

#[derive(Default, Clone)]
pub struct AgentDataJSON(pub serde_json::Value);

#[derive(Default, Clone)]
pub struct WeaponDataJSON(pub serde_json::Value);

utils::global_singleton!(VALORANT_RADAR, RadarJSON);

pub fn init(mem_ctx: &mut MemoryContext, bigpool_table: u64, bigpool_table_size: u64) {
    /*     let agent_data = valorant_utils::get_agent_data();
    let _ = valorant_utils::save_agent_data(&agent_data);
    log::info!("Got Agent Data"); */

    log::info!("Initializing Valorant module");

    log::info!("Parsing Game Data");
    let map_data = super::valorant::maps::MapData::init();
    let weapon_data = crate::valorant::weapons::WeaponData::init();
    let agent_data_manager = crate::valorant::agents::AgentManager::init();

    log::info!("Finding Guarded Memory");
    let guarded_region =
        valorant_utils::get_guarded_memory_address(mem_ctx, bigpool_table, bigpool_table_size);
    if guarded_region.is_err() {
        log::error!("Failed to find guarded memory");
        return;
    }
    let guarded_region = guarded_region.unwrap();
    // We will only use guarded region physical address, as it is guaranteed to be constigous
    let guarded_memory_physical = mem_ctx
        .driver_ctx
        .translate_linear_address(mem_ctx.kernel_ctx.cr3_phys, guarded_region);
    log::info!(
        "Guarded Memory: 0x{:X} (Physical: 0x{:X})",
        guarded_region,
        guarded_memory_physical
    );
    log::info!("Finding Valorant Process");

    let mut valorant: ProcessContext;
    loop {
        std::thread::sleep(Duration::from_millis(1));

        let valorant_process = utils::get_process(mem_ctx, "VALORANT-Win64", "VALORANT-Win64");
        if valorant_process.is_ok() {
            valorant = valorant_process.ok().unwrap();

            log::info!("Found Valorant");
            log::info!("\tBase Address: 0x{:X}", valorant.base_addr);
            log::info!(
                "\tDirectoryTableBase: 0x{:X}",
                valorant.directory_table_base
            );
            break;
        }
        //std::process::exit(0);
    }

    let mut val_ctx = ValorantContext {
        valorant: &mut valorant,
        mem_ctx,
        // guarded_memory_virtual: guarded_region,
        guarded_memory_physical,
        actor_pointer_array: HashSet::default(),
        player_array: HashSet::default(),
        game_engine: GameEngine::default(),
        map_name: String::new(),
        map_data,
        weapon_data,
        agent_data_manager,
        please_exit: false,
    };
    log::info!("Initialization Complete");
    // We're going single threaded for now.
    val_ctx.radar_thread();
}
