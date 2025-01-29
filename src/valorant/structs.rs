use std::collections::HashSet;

use crate::driver::{MemoryContext, ProcessContext};

#[derive(Default, Debug, Clone)]
#[allow(dead_code)]
pub struct GameEngine {
    pub fname_decryption_key: u64,
    pub uworld: u64,
    pub game_instance: u64,
    pub game_state: u64,
    pub persistence_level: u64,
    pub local_player_array: u64,
    pub local_player: u64,
    pub local_player_controller: u64,
    pub local_player_pawn: u64,
    pub local_player_state: u64,
    pub local_player_team_comp: u64,
    pub local_player_camera_pointer: u64,
    pub actor_array: u64, // TArray
    pub actor_count: u32,

    pub local_camera_manager: u64,
    pub local_team_id: u32,
    pub local_agent_name: String,

    pub is_in_game: bool,
}

pub struct ValorantContext<'m, 'v> {
    pub valorant: &'v mut ProcessContext,
    pub mem_ctx: &'m mut MemoryContext,
    ///guarded_memory_virtual: u64,
    pub guarded_memory_physical: u64,
    pub actor_pointer_array: HashSet<u64>,
    pub player_array: HashSet<ValorantActor>,
    pub game_engine: GameEngine,
    pub map_data: super::maps::MapData,
    pub weapon_data: super::weapons::WeaponData,
    pub agent_data_manager: super::agents::AgentManager,
    pub map_name: String,
    pub please_exit: bool,
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct ValorantActor {
    // pasted from cygnus
    pub(crate) pointer: u64,
    // pub(crate) id: u32,
    pub(crate) fname_id: u32,
    pub(crate) name: String,
    //pub(crate) is_visible: bool,
    //pub(crate) is_dormant: bool,
    pub(crate) team_id: i32,
    pub(crate) root_component: u64, // SceneComponent
    pub(crate) damage_handler: u64,
    pub(crate) inventory: u64,
    //pub(crate) health: u32,
}

#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct FName {
    pub comparison_id: u32,
    pub display_id: u32,
    pub number: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
#[repr(align(2))]
pub struct FNameEntryHeader(u16);

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct FNameEntry {
    index: u32,
    header: FNameEntryHeader,
    pub name: [u8; 1024],
}

impl Default for FNameEntry {
    fn default() -> Self {
        Self {
            index: 0,
            header: FNameEntryHeader(0),
            name: [0; 1024],
        }
    }
}

impl FNameEntry {
    pub const fn get_length(&self) -> u16 {
        self.header.0 >> 1
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
#[allow(dead_code)]
struct FNameEntryAllocator {
    frw_lock: [u8; 8],
    current_block: i32,
    current_byte_cursor: i32,
    blocks: [u8; 8192],
}

#[derive(Debug, Clone)]
#[repr(C)]
#[allow(dead_code)]
struct FNamePool {
    allocator: FNameEntryAllocator,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    #[allow(dead_code)]
    pub fn distance(self, other: &Vec3) -> f32 {
        let x = self.x - other.x;
        let y = self.y - other.y;
        let z = self.z - other.z;
        (x * x + y * y + z * z).sqrt()
    }
    // impl bitfield::Into<serde_json::Value>
    pub fn to_json(self) -> serde_json::Value {
        serde_json::json!({
            "x": self.x,
            "y": self.y,
            "z": self.z,
        })
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct FRotator {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

#[derive(Default, Debug, Clone)]
#[repr(C)]
pub struct FMinimalViewInfo {
    pub location: Vec3,
    pub rotation: FRotator,
    pub fov: f32,
    pub desired_fov: f32,
    pub orto_width: f32,
    pub ortho_near_clip_plane: f32,
    pub ortho_far_clip_plane: f32,
    pub aspect_ratio: f32,
}

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
#[repr(u16)]
#[derive(Debug)]
pub enum EAresGamePhase {
    NotStarted,
    GameStarted,
    BetweenRounds,
    RoundStarting,
    InRound,
    RoundEnding,
    SwitchingTeams,
    GameEnded,
    Count,
    Invalid,
    EAresGamePhase_MAX,
}
