use super::structs::EAresGamePhase;
use super::{
    offsets::{self},
    structs::{FMinimalViewInfo, ValorantActor, Vec3},
    valorant_utils::ReadGame,
    RadarJSON, ValorantContext,
};
use crate::driver::ReadPhysicalMemory;
use std::collections::HashSet;

impl ValorantContext<'_, '_> {
    fn actor_cache(&mut self) {
        self.game_engine.uworld = self
            .mem_ctx
            .driver_ctx
            .read_physical_memory(self.guarded_memory_physical + offsets::UWORLD);

        self.update_world_pointers();

        self.update_map_name();

        self.game_engine.fname_decryption_key = self
            .mem_ctx
            .driver_ctx
            .read_physical_memory(self.guarded_memory_physical);

        let mut actor_pointers = vec![0u64; self.game_engine.actor_count as usize];

        /*   for a in 0u64..self.game_engine.actor_count as u64 {
            actor_pointers[a as usize] = self.read(self.game_engine.actor_array + (a * 8));
        } */

        self.read_into(
            self.game_engine.actor_array,
            actor_pointers.as_mut_ptr() as *mut u8,
            self.game_engine.actor_count as usize * 8,
        );
        actor_pointers.retain(|&x| x != 0);

        let actor_set: HashSet<u64> = HashSet::from_iter(actor_pointers);

        let new_actors: Vec<_> = actor_set
            .difference(&self.actor_pointer_array)
            .cloned()
            .collect();
        let removed_actors: Vec<_> = self
            .actor_pointer_array
            .difference(&actor_set)
            .cloned()
            .collect();

        for &actor_ptr in &removed_actors {
            self.player_array.retain(|x| x.pointer != actor_ptr);
        }

        for actor_ptr in new_actors {
            // log::info!("new actor {:X}", actor_ptr);
            if actor_ptr == self.game_engine.local_player_pawn {
                continue;
            }

            let actor_uid: u32 = self.read(actor_ptr + offsets::UNIQUE_ID);

            if actor_uid != 18743553 {
                continue;
            }
            let actor_fname_id: u32 = self.read(actor_ptr + offsets::ACTOR_ID);

            let actor_name = self.get_fname(actor_fname_id);
            if actor_name.is_err() {
                continue;
            }
            let actor_name = actor_name.unwrap();
            if !self.agent_data_manager.contains_developer_name(&actor_name) {
                //log::info!("Actor Name: {}", actor_name);
                continue;
            }

            let scene_component: u64 = self.read(actor_ptr + offsets::ROOT_COMPONENT);
            let player_state: u64 = self.read(actor_ptr + offsets::PLAYER_STATE);
            let team_component: u64 = self.read(player_state + offsets::TEAM_COMPONENT);
            let damage_handler: u64 = self.read(actor_ptr + offsets::DAMAGE_HANDLER);
            let inventory: u64 = self.read(actor_ptr + offsets::PLAYER_INVENTORY);

            let team_id: u32 = if player_state == 0 {
                69420
            } else {
                self.read(team_component + offsets::TEAM_ID)
            };

            let current_actor = ValorantActor {
                //id: actor_uid,
                fname_id: actor_fname_id,
                name: self
                    .agent_data_manager
                    .translate_developer_name(&actor_name)
                    .unwrap()
                    .to_string(),
                pointer: actor_ptr,
                root_component: scene_component,
                team_id: team_id as i32,
                //is_dormant,
                damage_handler,
                inventory,
                //health: actor_hp as u32,
            };

            // Hashset already checks for duplicates, so we don't need to check it again.
            let _inserted = self.player_array.insert(current_actor);
        }
        self.actor_pointer_array = actor_set;
    }

    pub fn update_map_name(&mut self) {
        let client_game_instance: u64 =
            self.read(self.game_engine.game_state + offsets::CLIENT_GAME_INSTANCE);
        let map_load_model: u64 = self.read(client_game_instance + offsets::MAP_LOAD_MODEL);
        let map_name_ptr: u64 = self.read(map_load_model + offsets::MAP_NAME);
        let mut name_buffer = vec![0u16; 0x10];
        /* log::info!("Map Name Ptr: {:X}", map_name_ptr);
        log::info!("maploadmodel: {:X}", map_load_model);
        log::info!("clientgameinstance: {:X}", client_game_instance);
        log::info!("gamestate: {:X}", game_state);
        log::info!("Map Name: {}", self.map_name); */
        //self.display_memory(map_load_model, 0x200);
        self.read_into(
            map_name_ptr,
            name_buffer.as_mut_ptr() as *mut u8,
            (2 * 0x10) as usize,
        );
        name_buffer.truncate(name_buffer.iter().position(|&x| x == 0).unwrap_or(0));
        let map_name = String::from_utf16(&name_buffer);
        //log::info!("map name : {:?}", &map_name);

        //println!();
        if map_name.is_err() {
            self.map_name = "Ascent".to_string();
        } else if let Ok(map_name) = map_name {
            if map_name.trim().len() < 4 {
                self.map_name = "Ascent".to_string();
            }
            if map_name.contains("Range") || map_name.contains("䮘镤") {
                self.map_name = "Ascent".to_string();
            } else {
                self.map_name = map_name.trim().to_string();
            }
        }
        //log::info!("Map Name: {}", self.map_name);
        self.map_data.set_current_map_developer_name(&self.map_name);
        if std::fs::File::open(format!(
            "./static/assets/valorant_maps/{}.png",
            self.map_data.current_map_display_name
        ))
        .is_err()
        {
            let _ = self.map_data.save_map_image();
        }
        let _ = self.map_data.update_map_scalers();
    }

    pub fn radar_thread(&mut self) {
        let mut start_time = std::time::Instant::now();
        let mut ticks = 0u128;
        self.actor_cache();
        loop {
            // We update the data first
            ticks += 1;

            if start_time.elapsed().as_millis() > 1000 {
                self.actor_cache();
                //log::info!("Actor Count: {}", self.game_engine.actor_count);
                log::info!("Average Tickrate: {}", ticks);
                ticks = 0;
                start_time = std::time::Instant::now();
            }

            if self.please_exit {
                break;
            }
            // Get camera cache
            let camera_cache: FMinimalViewInfo =
                self.read(self.game_engine.local_camera_manager + offsets::CAMERA_INFO);

            let local_position = camera_cache.location;
            let local_view_angle = camera_cache.rotation;

            // Start caching data for the json
            let mut player_count = 0;
            let mut radar_json: serde_json::Value = serde_json::Value::default();
            radar_json["players"] = serde_json::json!([]);

            //let players: HashSet<ValorantActor> = self.player_array.clone();
            for player in &self.player_array {
                //log::info!("Player: {:X}", player.pointer);
                let mut current_player_json: serde_json::Value = serde_json::Value::default();

                let is_dormant: bool = self.read(player.pointer + offsets::IS_DORMANT);

                let health: f32 = self.read(player.damage_handler + offsets::HEALTH);
                //let scene_component: u64 = self.read(player.pointer + offsets::ROOT_COMPONENT);

                let entity_position: Vec3 =
                    self.read(player.root_component + offsets::ROOT_POSITION);
                let entity_rotation: Vec3 =
                    self.read(player.root_component + offsets::ROOT_ROTATION);

                if health < 0.1 {
                    continue;
                }
                let entity_weapon_base: u64 = self.read(player.inventory + offsets::CURRENT_WEAPON);
                let entity_weapon: u32 = self.read(entity_weapon_base + offsets::ACTOR_ID);
                let weapon_fname = self.get_fname(entity_weapon);

                let weapon_dev_name = weapon_fname
                    .unwrap_or(String::from("Unknown"))
                    .trim()
                    .to_string();

                let is_ability = !self.weapon_data.contains_developer_name(&weapon_dev_name)
                    || entity_weapon == 0;
                let equippable_display_name = if is_ability {
                    //log::info!("Ability: {}", entity_weapon_name);
                    weapon_dev_name.clone() // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                } else {
                    self.weapon_data
                        .translate_developer_name(&weapon_dev_name)
                        .unwrap_or("Unknown")
                        .to_string()
                };

                current_player_json["team"] =
                    if player.team_id == self.game_engine.local_team_id as i32 {
                        "ally".into()
                    } else {
                        "enemy".into()
                    };

                current_player_json["nickname"] = player.name.clone().into();
                current_player_json["networkable"] = is_dormant.into(); // its actually not "dormant" but networkable
                current_player_json["health"] = (health as u32).into();
                current_player_json["weapon_name"] = equippable_display_name.clone().into();

                let player_map_positon = self.map_data.get_map_position(&entity_position);

                current_player_json["map_position_x"] = player_map_positon.x.into();
                current_player_json["map_position_y"] = player_map_positon.y.into();
                current_player_json["is_ability"] = is_ability.into();
                current_player_json["pInventory"] = player.inventory.into();
                current_player_json["rotation"] = entity_rotation.to_json();
                // Agent index
                let agent_index = self
                    .agent_data_manager
                    .actor_name_mappings
                    .iter()
                    .position(|x| x.0 == player.name)
                    .unwrap_or(0);
                current_player_json["agent_index"] = agent_index.into();

                if !is_ability {
                    // We get the current weapon index
                    let weapon_index = self
                        .weapon_data
                        .weapon_name_mappings
                        .iter()
                        .position(|x| x.0 == equippable_display_name)
                        .unwrap_or(0);
                    current_player_json["weapon_index"] = weapon_index.into();
                }

                radar_json["players"]
                    .as_array_mut()
                    .unwrap()
                    .push(serde_json::json!(current_player_json));
                player_count += 1;
            }

            radar_json["local_view_angle_x"] = local_view_angle.pitch.into();
            radar_json["local_view_angle_y"] = local_view_angle.yaw.into();
            radar_json["local_agent_index"] = self
                .agent_data_manager
                .actor_name_mappings
                .iter()
                .position(|x| x.0 == self.game_engine.local_agent_name)
                .unwrap_or(0)
                .into();

            let local_map_position = self.map_data.get_map_position(&local_position);

            radar_json["local_map_coordinate"] =
                serde_json::json!([local_map_position.x, local_map_position.y]);

            radar_json["map_name"] = self.map_data.current_map_display_name.clone().into();
            radar_json["entity_count"] = player_count.into();
            radar_json["command"] = "render".into();
            *RadarJSON::get_mut() = RadarJSON(radar_json);
        }
    }

    pub fn update_world_pointers(&mut self) {
        //log::info!("{:X}\t{:X}", uworld, self.game_engine.fname_decryption_key);

        self.game_engine.persistence_level =
            self.read(self.game_engine.uworld + offsets::PERSISTENT_LEVEL);

        if self.game_engine.persistence_level > 0x7FFFFFFFFFFF {
            return;
        }
        self.game_engine.game_instance =
            self.read(self.game_engine.uworld + offsets::GAME_INSTANCE);

        self.game_engine.game_state = self.read(self.game_engine.uworld + offsets::GAME_STATE);

        let game_phase: EAresGamePhase =
            self.read(self.game_engine.game_state + offsets::GAME_PHASE);
        match game_phase {
            EAresGamePhase::Invalid
            | EAresGamePhase::BetweenRounds
            | EAresGamePhase::GameEnded
            | EAresGamePhase::GameStarted
            | EAresGamePhase::RoundStarting
            | EAresGamePhase::SwitchingTeams => {
                //log::info!("Game Phase: {:?}", game_phase);
                self.actor_pointer_array.clear();
                self.player_array.clear();
            }
            _ => {}
        }

        //log::info!("{:X}\t{:X}", persistent_level, game_instance);

        self.game_engine.actor_array =
            self.read(self.game_engine.persistence_level + offsets::ACTOR_ARRAY);

        self.game_engine.actor_count =
            self.read(self.game_engine.persistence_level + offsets::ACTOR_COUNT);

        /* log::info!(
            "Actor Count: {:X}\t{}",
            self.game_engine.persistence_level + offsets::ACTOR_ARRAY,
            self.game_engine.actor_count
        ); */
        self.game_engine.local_player_array =
            self.read(self.game_engine.game_instance + offsets::LOCAL_PLAYER_ARRAY);

        self.game_engine.local_player = self.read(self.game_engine.local_player_array);

        self.game_engine.local_player_controller =
            self.read(self.game_engine.local_player + offsets::PLAYER_CONTROLLER);

        let local_pawn = self.read(self.game_engine.local_player_controller + offsets::APAWN);

        if local_pawn == 0 || local_pawn != self.game_engine.local_player_pawn {
            self.actor_pointer_array.clear();
            self.player_array.clear();
        }
        self.game_engine.local_player_pawn = local_pawn;
        self.game_engine.local_camera_manager =
            self.read(self.game_engine.local_player_controller + offsets::CAMERA_MANAGER);

        self.game_engine.local_player_state =
            self.read(self.game_engine.local_player_pawn + offsets::PLAYER_STATE);

        self.game_engine.local_player_team_comp =
            self.read(self.game_engine.local_player_state + offsets::TEAM_COMPONENT);

        self.game_engine.local_team_id =
            self.read(self.game_engine.local_player_team_comp + offsets::TEAM_ID);

        // We get the local agent name
        let actor_id = self.read(self.game_engine.local_player_pawn + offsets::ACTOR_ID);
        let actor_fname = self
            .get_fname(actor_id)
            .unwrap_or(String::from("Wushu_PC_C"));
        let local_agent_name = self
            .agent_data_manager
            .translate_developer_name(&actor_fname)
            .unwrap_or(&String::from("Wushu_PC_C"))
            .to_string();

        self.game_engine.local_agent_name = local_agent_name;

        //log::info!("Local Player Pawn: {:X}", local_player_controller);
        //log::info!("Actor Array Count: {}", actor_array_count);
    }

    #[allow(dead_code)]
    fn display_memory(&self, address: u64, size: usize) {
        let data = vec![0u8; 0x1000];
        self.read_into(address, data.as_ptr() as *mut u8, size);
        data.iter().take(size).enumerate().for_each(|(i, x)| {
            if i % 16 == 0 {
                //if i != 0 {
                if i > 0 {
                    print!(
                        "      {}",
                        String::from_utf8_lossy(&data[(i - 16)..i]).replace(' ', ".")
                    );
                }
                println!();
                //}
                print!("0x{:04x} : ", i);
            }
            print!("{:02x} ", x);
        });
    }
}
