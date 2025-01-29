#![allow(dead_code)]
#![allow(clippy::unreadable_literal)]
#![cfg_attr(rustfmt, rustfmt_skip)]



pub const FNAME_DECRYPTION_PTR: u64 = 0x0; // Guarded phys > fname decryption ptr
pub const FNAME_POOL: u64 = 0xA7B6780; // Game module base > fname pool (48 8D 1D ? ? ? ? EB 16)
pub const UWORLD: u64 = 0x60; // Guarded phys > uworld

// World
pub const PERSISTENT_LEVEL: u64 = 0x38; // world > persistent_level // ULevel*              PersistentLevel;
pub const GAME_INSTANCE: u64 = 0x1A0; // world > game_instance      // UGameInstance*       OwningGameInstance; 
pub const GAME_MODE: u64 = 0x138; // world > game_mode              // AGameModeBase*       AuthorityGameMode;
pub const GAME_STATE: u64 = 0x140; // world > game_state            // AGameStateBase*      GameState;

// Game mode
//pub const MATCH_STATE: u64 = 0x420; // world > game_mode > match_state

// Game state
pub const GAME_PHASE: u64 = 0x0B10; // world > game_state > EAresGamePhase; // EAresGamePhase Phase;
pub const MAP_LOAD_MODEL: u64 = 0x240 + 0x0270; // world > game_state > FAllInitSystems > UMapLoadModel* MapLoadModel; // 0x0218   (0x0008)  
pub const MAP_NAME: u64 = 0x58; // world > game_state > UMapLoadModel > MapName
pub const CLIENT_GAME_INSTANCE : u64 = 0x6a0; // world > game_state >  UAresClientGameInstance* ClientGameInstance;
// Player
pub const LOCAL_PLAYER_ARRAY: u64 = 0x40; // world > game_instance > localplayer_array
pub const LOCAL_PLAYER: u64 = 0x40; // world > game_instance > localplayer_array[0]
pub const PLAYER_CONTROLLER: u64 = 0x38; // world > game_instance > localplayer_array[0] > playercontroller
pub const VIEWPORT_CLIENT: u64 = 0x78; // world > game_instance > localplayer_array[0] > playercontroller -> UGameViewportClient*
pub const VIEWPORT: u64 = 0xf0; // world > game_instance > localplayer_array[0] > playercontroller -> UGameViewportClient* -> FViewport*
pub const SCREEN_RES: u64 = 0xb0; // world > game_instance > localplayer_array[0] > playercontroller -> FViewport* -> UVec2
pub const CAMERA_CONTROLLER: u64 = 0x0448; // world > game_instance > localplayer_array[0] > playercontroller > cameracontroller //? is this ControlRotation
pub const CAMERA_MANAGER: u64 = 0x460; // world > game_instance > localplayer_array[0] > playercontroller > APlayerCameraManager*   PlayerCameraManager
pub const CAMERA_INFO: u64 = 0x1f80 + 0x10; // world > game_instance > localplayer_array[0] > playercontroller > cameramanager > FCameraCacheEntry  CameraCachePrivate;
pub const DAMAGE_HANDLER: u64 = 0x09E8; // world > game_instance > localplayer_array[0] > playercontroller > UDamageableComponent* DamageHandler; (class AShooterCharacter)
pub const HEALTH: u64 = 0x1B0; // world > game_instance > localplayer_array[0] > playercontroller > damagehandler > health // struct FHealthValue CachedLife[0x3]; // 0x1b0(0x18))
pub const ALIVE: u64 = 0x01A9; // world > game_instance > localplayer_array[0] > playercontroller > damagehandler > ALIVE
pub const APAWN: u64 = 0x448; // world > game_instance > localplayer_array[0] > playercontroller > APawn* AcknowledgedPawn;
pub const PLAYER_INVENTORY: u64 = 0x0988; // world > game_instance > localplayer_array[0] > playercontroller > apawn > UAresInventory* Inventory;

// Level > Actors

pub const ACTOR_ARRAY: u64 = 0xA0  ; // world > persistent_level > actor_array
pub const ACTOR_COUNT: u64 = ACTOR_ARRAY + 0x8; // world > persistent_level > actor_count
//pub const ACTOR_ARRAY: u64 = 0xA0; // world > persistent_level > actor_array
//pub const ACTOR_COUNT: u64 = 0xB8; // world > persistent_level > actor_count

// Vectors
pub const ROOT_COMPONENT: u64 = 0x0238; // world > game_instance > localplayers_array[0] > playercontroller > apawn > USceneComponent* RootComponent;
pub const ROOT_POSITION: u64 = 0x164; // world > game_instance > localplayers_array[0] > playercontroller > apawn > root_component > FVector RelativeLocation;
pub const ROOT_ROTATION: u64 = 0x0170; // world > game_instance > localplayers_array[0] > playercontroller > apawn > root_component > FVector RelativeRotation;

// Actor > Actor data
pub const ACTOR_ID: u64 = 0x18;
pub const UNIQUE_ID: u64 = 0x38;
pub const IS_DORMANT: u64 = 0x100;
pub const MESH: u64 = 0x418; // USkeletalMeshComponent* Mesh; (class ACharacter : public APawn)
pub const SKELETAL_MESH: u64 = 0x598; // mesh > USkeletalMesh* SkeletalMesh; 
pub const PLAYER_STATE: u64 = 0x3d8; // APlayerState* PlayerState; (class APawn : public AActor) 
// ? Theres another one in class AController : public AActor at 0x03e0
pub const TEAM_COMPONENT: u64 = 0x0610; // player_state > UBaseTeamComponent* TeamComponent; 
pub const TEAM_ID: u64 = 0xF8; // team_component > team_id
pub const CURRENT_WEAPON: u64 = 0x0248; // actor > actor_inventory (UAresInventory) > AAresEquippable* CurrentEquippable;
pub const PLAYER_VELOCITY: u64 = 0x188; // actor > root(USceneComponent) > c

// Actor > Actor data > Mesh
pub const COMPONENT_TO_WORLD: u64 = 0x250;
pub const BONE_ARRAY: u64 = 0x5C8;
pub const BONE_COUNT: u64 = 0x5E0;

