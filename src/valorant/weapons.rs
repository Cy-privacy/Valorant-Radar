use super::WeaponDataJSON;

// valorant-api doesn't have weapon developer names
static WEAPONS: phf::Map<&'static str, &'static str> = phf::phf_map! {
    // Agents
    "Ability_Melee_Base_C" => "Melee",
    "BasePistol_C" => "Classic",
    "TrainingBotBasePistol_C" => "Classic",
    "SawedOffShotgun_C" => "Shorty",
    "RevolverPistol_C" => "Sheriff",
    "AutomaticPistol_C" => "Frenzy",
    "LugerPistol_C" => "Ghost",
    "Vector_C" => "Stinger",
    "SubMachineGun_MP5_C" => "Spectre",
    "PumpShotgun_C" => "Bucky",
    "AutomaticShotgun_C" => "Judge",
    "AssaultRifle_Burst_C" => "Bulldog",
    "DMR_C" => "Guardian",
    "AssaultRifle_ACR_C" => "Phantom",
    "AssaultRifle_AK_C" => "Vandal",
    "LeverSniperRifle_C" => "Marshal",
    "BoltSniper_C" => "Operator",
    "LightMachineGun_C" => "Ares",
    "HeavyMachineGun_C" => "Odin",
    //"Ability_Wushu_X_Dagger_Production_C" => "Jett Kunai",
};

pub struct WeaponData {
    //weapon_json_data: serde_json::Value,
    pub weapon_name_mappings: Vec<(String, u16)>, // This could be a hasmap bruh
                                                  // TODO
}
crate::utils::global_singleton!(AAA, super::WeaponDataJSON);

impl WeaponData {
    pub fn init() -> WeaponData {
        let weapon_json = get_weapon_data();

        // Check if path exists
        // If not, create it
        let _ = std::fs::create_dir_all("./static/assets/valorant_weapons");
        // We also want to check if image files for each weapon exists on disk
        // If not, we will download them
        WeaponDataJSON::get_mut().0 = serde_json::Value::default();
        let weapon_data_array = weapon_json["data"].as_array().unwrap();

        let mut weapon_name_mappings: Vec<(String, u16)> = vec![];

        for weapon_data in weapon_data_array {
            let weapon_name = weapon_data["displayName"].as_str().unwrap();
            let weapon_image_path = format!("./static/assets/valorant_weapons/{}.png", weapon_name);
            let weapon_image_file = std::fs::File::open(&weapon_image_path);
            if weapon_image_file.is_err() {
                // Download the image
                log::info!("Downloading image for {}", weapon_name);
                let weapon_image_url = weapon_data["displayIcon"].as_str().unwrap();
                let weapon_image = reqwest::blocking::get(weapon_image_url);
                if weapon_image.is_err() {
                    log::error!("Failed to get weapon image for {}", weapon_name);
                    continue;
                }
                let weapon_image = weapon_image.unwrap();
                let weapon_image = weapon_image.bytes();
                if weapon_image.is_err() {
                    log::error!("Failed to read weapon image for {}", weapon_name);
                    continue;
                }
                let weapon_image = weapon_image.unwrap();
                // Save the bytes to map_name.png
                let file = std::fs::File::create(&weapon_image_path);
                if file.is_err() {
                    log::error!("Failed to save weapon image for {}", weapon_name);
                    continue;
                }
                let mut file = file.unwrap();
                let _ = std::io::Write::write_all(&mut file, &weapon_image);
            }

            weapon_name_mappings.push((weapon_name.to_string(), weapon_name_mappings.len() as u16));
        }

        WeaponDataJSON::get_mut().0["weapons"] = serde_json::Value::Array(
            weapon_name_mappings
                .iter()
                .map(|x| serde_json::Value::String(x.0.clone()))
                .collect(),
        );

        WeaponDataJSON::get_mut().0["total_weapons"] =
            serde_json::Value::from(weapon_name_mappings.len() as u32);

        WeaponData {
            // weapon_json_data: weapon_json,
            weapon_name_mappings,
        }
    }
    pub fn contains_developer_name(&self, developer_name: &str) -> bool {
        WEAPONS.contains_key(developer_name)
    }
    pub fn translate_developer_name(&self, developer_name: &str) -> Option<&str> {
        WEAPONS.get(developer_name).copied()
    }
}

fn get_weapon_data() -> serde_json::Value {
    // Check if file exists on disk
    use std::fs;
    let file = fs::File::open("weapon_data.json");
    if let Ok(weapon_json) = file {
        log::info!("Found weapon data from disk");
        return serde_json::from_reader(weapon_json).unwrap();
    }
    let text_response = reqwest::blocking::get("https://valorant-api.com/v1/weapons");
    if text_response.is_err() {
        panic!("Failed to get weapon data");
    }
    let text_response = text_response.unwrap();

    let json_data = serde_json::from_str(&text_response.text().unwrap()).unwrap();
    let did_save = save_weapon_data(&json_data);
    if did_save.is_err() {
        log::error!("Failed to save weapon data");
    }
    log::info!("Weapon Data Loaded");
    json_data
}

pub fn save_weapon_data(weapon_json: &serde_json::Value) -> Result<(), serde_json::Error> {
    use std::fs;
    let file = fs::File::create("weapon_data.json");
    let file = file.unwrap();
    serde_json::to_writer_pretty(file, &weapon_json)
}
