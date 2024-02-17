pub struct House {
    pub rooms: Vec<Room>,
    pub key: String,
}

impl House {
    fn new() -> House {
        House {
            rooms: Vec::new(),
            key: String::new(),
        }
    }

    fn change_key(&mut self, key: &str) {
        self.key = String::from(key);
    }

    fn add_room(&mut self, room: Room) {
        self.rooms.push(room);
    }

    #[allow(dead_code)]
    pub fn all_sensors(&self) -> Vec<String> {
        let mut output: Vec<String> = Vec::new();
        for room in &self.rooms {
            for sensor in room.all_sensors() {
                output.push(sensor)
            }
        }
        output
    }
}

impl std::fmt::Display for House {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "House {{")?;
        // writeln!(f, "\tkey: \"{}\",", self.key)?;
        writeln!(f, "\trooms: [")?;
        for room in &self.rooms {
            writeln!(f, "\t\t{},", room)?;
        }
        writeln!(f, "\t],")?;
        write!(f, "}}")
    }
}

pub struct Room {
    pub name: String,
    pub sensors: Vec<String>,
}

#[allow(dead_code)]
impl Room {
    fn new() -> Room {
        Room {
            name: String::new(),
            sensors: Vec::new(),
        }
    }

    fn change_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    fn add_sensor(&mut self, sensor: &str) {
        self.sensors.push(sensor.to_owned())
    }

    fn all_sensors(&self) -> Vec<String> {
        self.sensors.clone()
    }
}

impl std::fmt::Display for Room {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Room {{")?;
        writeln!(f, "\tname: \"{}\",", self.name)?;
        writeln!(f, "\tsensors: [")?;
        for sensor in &self.sensors {
            writeln!(f, "\t\t\"{}\",", sensor)?;
        }
        writeln!(f, "\t],")?;
        write!(f, "}}")
    }
}

pub async fn readout_logging_page(
    /* Parese through logging page. Returns CSRF-Token and Layout with every sensor as a struct */
    client: &reqwest::Client,
    url: &str,
) -> Result<House, Box<dyn std::error::Error>> {
    let mut house: House = House::new();

    let raw_html: scraper::Html =
        scraper::Html::parse_document(&client.get(url).send().await?.text().await?);

    let csrf_selector: scraper::Selector =
        scraper::Selector::parse("input[name=csrfmiddlewaretoken][type=hidden]")?;
    let room_selector: scraper::Selector =
        scraper::Selector::parse("div[data-role='collapsible']")?;
    let sensor_selector: scraper::Selector = scraper::Selector::parse("input[type='checkbox']")?;
    let field_selector: scraper::Selector = scraper::Selector::parse("fieldset")?;
    let titel_selector: scraper::Selector = scraper::Selector::parse("h5")?;

    house.change_key(
        raw_html
            .select(&csrf_selector)
            .next()
            .and_then(|input| input.value().attr("value"))
            .ok_or("CSRF token not found")?,
    );

    for room_element in raw_html.select(&room_selector) {
        let mut room_holder = Room::new();
        if let Some(title_element) = room_element.select(&titel_selector).next() {
            room_holder.change_name(title_element.text().collect::<String>().trim());
        }

        if let Some(fieldset_element) = room_element.select(&field_selector).next() {
            for sensor_element in fieldset_element.select(&sensor_selector) {
                room_holder.add_sensor(sensor_element.value().attr("id").unwrap_or_default().trim())
            }
        }
        house.add_room(room_holder)
    }
    Ok(house)
}
