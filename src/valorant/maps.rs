use super::structs::{Vec2, Vec3};

pub struct MapData {
    pub json_data: serde_json::Value,
    pub current_map_developer_name: String,
    pub current_map_display_name: String,
    // callouts
    pub x_multiplier: f32,
    pub x_scalar_to_add: f32,
    pub y_multiplier: f32,
    pub y_scalar_to_add: f32,
    //pub map_scale: f32,
}

pub fn get_json_data() -> serde_json::Value {
    // Check if file exists on disk
    use std::fs;
    let file = fs::File::open("map_data.json");
    if let Ok(map_json) = file {
        log::info!("Found map data from disk");
        return serde_json::from_reader(map_json).unwrap();
    }
    let text_response = reqwest::blocking::get("https://valorant-api.com/v1/maps");
    if text_response.is_err() {
        panic!("Failed to get map data");
    }
    let text_response = text_response.unwrap();
    let json_data = serde_json::from_str(&text_response.text().unwrap()).unwrap();
    let did_save = save_json_data(&json_data);
    if did_save.is_err() {
        log::error!("Failed to save map data");
    }
    json_data
}

pub fn save_json_data(map_json: &serde_json::Value) -> Result<(), serde_json::Error> {
    use std::fs;
    let file = fs::File::create("map_data.json");
    let file = file.unwrap();
    serde_json::to_writer_pretty(file, &map_json)
}

impl MapData {
    pub fn init() -> MapData {
        let map_json = get_json_data();
        // ?? We dont cache map images on init because, map switches will be
        // much less frequent than how often we would need weapon/agent images.
        // We save those on init so we can preload them on the web-browser.
        MapData {
            json_data: map_json,
            current_map_developer_name: "Range".to_string(),
            current_map_display_name: "Range".to_string(),
            x_multiplier: 0.0,
            x_scalar_to_add: 0.0,
            y_multiplier: 0.0,
            y_scalar_to_add: 0.0,
            // map_scale: 0.0,
        }
    }
    pub fn set_current_map_developer_name(&mut self, map_name: &str) {
        self.current_map_developer_name = map_name.to_string();
        let _ = self.translate_developer_map_name();
    }

    #[allow(dead_code)]
    pub fn get_current_map_developer_name(&self) -> &str {
        &self.current_map_developer_name
    }

    pub fn update_map_scalers(&mut self) -> Result<(), ()> {
        let map_name = &self.current_map_display_name;
        let map_data = self.json_data["data"].as_array().unwrap();
        let map_data = map_data
            .iter()
            .find(|x| x["displayName"].as_str().unwrap() == map_name);
        if map_data.is_none() {
            log::info!("Failed to find map data for {}", map_name);
            self.current_map_display_name = "Ascent".to_string();
            return Err(());
        }
        let map_data = map_data.unwrap();
        let x_multiplier = map_data["xMultiplier"].as_f64().unwrap() as f32;
        let x_scalar_to_add = map_data["xScalarToAdd"].as_f64().unwrap() as f32;
        let y_multiplier = map_data["yMultiplier"].as_f64().unwrap() as f32;
        let y_scalar_to_add = map_data["yScalarToAdd"].as_f64().unwrap() as f32;
        //let map_scale = map_data["mapScale"].as_f64().unwrap().unwrap_or(0.0) as f32;
        self.x_multiplier = x_multiplier;
        self.x_scalar_to_add = x_scalar_to_add;
        self.y_multiplier = y_multiplier;
        self.y_scalar_to_add = y_scalar_to_add;
        Ok(())
    }

    pub fn save_map_image(&self) -> Result<(), ()> {
        // We will use the current map name to save the image from the
        // displayIcon from the json
        let _ = std::fs::create_dir_all("./static/assets/valorant_maps/");

        let map_data_json = self.json_data["data"].as_array().unwrap();
        /*  let map_data = map_data
        .iter()
        .find(|x| x["displayName"].as_str().unwrap() == map_name); */
        let current_map_data = map_data_json.iter().find(|x| {
            x["mapUrl"]
                .as_str()
                .unwrap()
                .contains(&self.current_map_developer_name)
        });

        if current_map_data.is_none() {
            return Err(());
        }

        log::info!("Saving map image for {}", self.current_map_display_name);
        let map_data = current_map_data.unwrap();

        let map_image_url = map_data["displayIcon"].as_str().unwrap();
        let map_image = reqwest::blocking::get(map_image_url);
        if map_image.is_err() {
            return Err(());
        }
        let map_image = map_image.unwrap();
        let map_image = map_image.bytes();
        if map_image.is_err() {
            return Err(());
        }
        let map_image = map_image.unwrap();
        // Save the bytes to map_name.png
        let file = std::fs::File::create(format!(
            "./static/assets/valorant_maps/{}.png",
            self.current_map_display_name
        ));
        if file.is_err() {
            return Err(());
        }
        let mut file = file.unwrap();
        let _ = std::io::Write::write_all(&mut file, &map_image);
        Ok(())
    }

    pub fn get_map_position(&self, position: &Vec3) -> Vec2 {
        let x = (position.y * self.x_multiplier + self.x_scalar_to_add) * 1024.0;
        let y = (position.x * self.y_multiplier + self.y_scalar_to_add) * 1024.0;
        Vec2 { x, y }
    }

    fn translate_developer_map_name(&mut self) -> Result<(), ()> {
        // Developer Map Name is part of a string in the map data
        // string is mapUrl: "/Game/Maps/Ascent/Ascent"
        // the self.current_map field is the developer name, so we need to
        // return the displayName from the json
        let map_data = self.json_data["data"].as_array().unwrap();
        let map_data = map_data
            .iter()
            .find(|x| {
                x["mapUrl"]
                    .as_str()
                    .unwrap()
                    .contains(&self.current_map_developer_name)
            })
            .ok_or(())?;
        let mut map_name = map_data["displayName"].as_str().unwrap().to_string();
        map_name.retain(|c| !c.is_whitespace());
        self.current_map_display_name = map_name;
        Ok(())
    }
}
