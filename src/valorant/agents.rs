use std::collections::HashMap;

use super::AgentDataJSON;

/* pub static AGENTS: phf::Map<&'static str, &'static str> = phf::phf_map! {
    // Agents
    "Breach_PC_C" => "Breach",
    "BountyHunter_PC_C" => "Fade",
    "Clay_PC_C" => "Raze",
    "Deadeye_PC_C" => "Chamber",
    "Gumshoe_PC_C" => "Cypher",
    "Grenadier_PC_C" => "KAY/O",
    "Guide_PC_C" => "Skye",
    "Hunter_PC_C" => "Sova",
    "Killjoy_PC_C" => "Killjoy",
    "Mage_PC_C" => "Harbor",
    "Pandemic_PC_C" => "Viper",
    "Phoenix_PC_C" => "Phoenix",
    "Rift_PC_C" => "Astra",
    "Sarge_PC_C" => "Brimstone",
    "Sprinter_PC_C" => "Neon",
    "Stealth_PC_C" => "Yoru",
    "Thorne_PC_C" => "Sage",
    "Vampire_PC_C" => "Reyna",
    "Wushu_PC_C" => "Jett",
    "Wraith_PC_C" => "Omen",
    "AggroBot_PC_C" => "Gekko",
    "Cable_PC_C" => "Deadlock",
    //
    // Bots
    "TrainingBot_Med_C" => "Training Bot",
    "TrainingBot_PC_C" => "Training Bot",
    "TrainingBot_Hard_C" => "Training Bot",
    "Pawn_TrainingBot_Strafe_C" => "Training Bot",
    "Pawn_TrainingBot_C" => "Training Bot",
};
 */

pub struct AgentManager {
    //actor_data: serde_json::Value,
    pub str_test_agent: String,
    pub agents: HashMap<String, String>,
    pub actor_name_mappings: Vec<(String, u16)>, // This could be a hasmap bruh // TODO
}

crate::utils::global_singleton!(DOES_THIS_EVEN_MATTER, super::AgentDataJSON);

impl AgentManager {
    pub fn contains_developer_name(&self, developer_name: &str) -> bool {
        self.agents.contains_key(&developer_name.to_lowercase()) || developer_name.contains("Bot")
    }
    pub fn translate_developer_name(&self, developer_name: &str) -> Option<&String> {
        if developer_name.contains("TrainingBot") {
            //log::info!("Returning bot {}", developer_name);
            return Some(&self.str_test_agent);
        }
        self.agents.get(&developer_name.to_lowercase())
    }

    pub fn init() -> AgentManager {
        let actor_json = get_agent_data();
        // We also want to check if image files for each Actor exists on disk
        // If not, we will download them

        let actor_data_array = actor_json["data"].as_array().unwrap().clone();
        let mut agents: HashMap<String, String> = HashMap::new();
        let _ = std::fs::create_dir_all("./static/assets/valorant_agents/");
        AgentDataJSON::get_mut().0 = serde_json::Value::default();

        let mut actor_name_mappings: Vec<(String, u16)> = vec![];
        for actor in actor_data_array {
            // We need a check for isPlayableCharacter -> true
            let is_playable = actor["isPlayableCharacter"].as_bool().unwrap();
            if !is_playable {
                continue;
            }

            let mut actor_name = actor["displayName"].as_str().unwrap().to_string();
            let developer_name = actor["developerName"].as_str().unwrap().to_owned() + "_PC_C";

            // We need to remove all non-alphabetic characters from the actor name
            actor_name.retain(|c| c.is_alphabetic());

            let actor_image_path = format!("./static/assets/valorant_agents/{}.png", actor_name);
            let actor_image_file = std::fs::File::open(&actor_image_path);

            if actor_image_file.is_err() {
                // Download the image
                log::info!("Downloading image for {}", actor_name);

                let actor_image_url = actor["displayIcon"].as_str().unwrap();
                let actor_image = reqwest::blocking::get(actor_image_url);
                if actor_image.is_err() {
                    log::error!("Failed to get actor image for {}", actor_name);
                    continue;
                }
                let actor_image = actor_image.unwrap();
                let actor_image = actor_image.bytes();
                if actor_image.is_err() {
                    log::error!("Failed to get actor image for {}", actor_name);
                    continue;
                }
                let actor_image = actor_image.unwrap();
                // Save the bytes to map_name.png
                let file = std::fs::File::create(&actor_image_path);
                if file.is_err() {
                    log::error!("Failed to get actor image for {}", actor_name);
                    continue;
                }
                let mut file = file.unwrap();
                let _ = std::io::Write::write_all(&mut file, &actor_image);
            }
            // We will add it to our hashmap with the developer name as the key

            //log::info!("Adding {}  \t {}", developer_name, actor_name);

            agents.insert(developer_name.to_lowercase(), actor_name.to_string());
            actor_name_mappings.push((actor_name.to_string(), actor_name_mappings.len() as u16));
        }

        AgentDataJSON::get_mut().0["agents"] = serde_json::Value::Array(
            actor_name_mappings
                .iter()
                .map(|x| serde_json::Value::String(x.0.clone()))
                .collect(),
        );
        AgentDataJSON::get_mut().0["total_agents"] =
            serde_json::Value::Number(serde_json::Number::from(actor_name_mappings.len()));

        AgentManager {
            //actor_data: actor_json,
            agents,
            actor_name_mappings,
            str_test_agent: "Jett".to_string(),
        }
    }
}

pub fn get_agent_data() -> serde_json::Value {
    // Check if file exists on disk
    use std::fs;
    let file = fs::File::open("agent_data.json");
    if let Ok(agent_json) = file {
        log::info!("Found agent data on disk");
        return serde_json::from_reader(agent_json).unwrap();
    }
    log::info!("Loading Agent data");
    let text_response = reqwest::blocking::get("https://valorant-api.com/v1/agents");
    if text_response.is_err() {
        panic!("Failed to get agent data");
    }
    let text_response = text_response.unwrap();
    let json_data = serde_json::from_str(&text_response.text().unwrap()).unwrap();
    let did_save = save_agent_data(&json_data);
    if did_save.is_err() {
        log::error!("Failed to save Agent data");
    }
    log::info!("Agent Data Loaded");
    json_data
}

pub fn save_agent_data(agent_json: &serde_json::Value) -> Result<(), serde_json::Error> {
    use std::fs;
    let file = fs::File::create("agent_data.json");
    let file = file.unwrap();
    serde_json::to_writer_pretty(file, &agent_json)
}
