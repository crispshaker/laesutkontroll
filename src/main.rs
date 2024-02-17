mod logging;
mod login;
mod miscellaneous;
mod post;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const LOGIN_DATA: login::LoginData = login::LoginData::from("john.doe@example.com", "Tr0ub4dor&3");
    const LOGIN_PAGE: &str = "http://192.168.1.1/accounts/m_login/";
    const LOGGER_PAGE: &str = "http://192.168.1.1/m_logger/1/";
    const HOUSE_CONFIG: &str = "house_config.txt";

    let client: reqwest::Client = login::create_client(&LOGIN_DATA, LOGIN_PAGE)
        .await
        .expect("Failed to create client.");

    let house: logging::House = logging::readout_logging_page(&client, LOGGER_PAGE)
        .await
        .expect("Failed to get Sensors from logger page");

    miscellaneous::save_to_file(&house.to_string(), HOUSE_CONFIG, crate::miscellaneous::WriteMode::Overwrite).unwrap_or_else(|_| {
        crate::miscellaneous::write_to_log(&format!("Failed to save {}", &HOUSE_CONFIG)).unwrap();
    });

    for room in house.rooms {
        for sensor in room.sensors {
            let file_name: &str = &format!("{}.csv", sensor);
            let csv: &str = &post::request_sensor(
                &client,
                &sensor,
                &miscellaneous::read_last_log(file_name).unwrap(),
                &miscellaneous::current_time(),
                LOGGER_PAGE,
                &house.key,
            )
            .await
            .unwrap_or_else(|_| {
                crate::miscellaneous::write_to_log(&format!("Failed to get {}", &sensor))
                    .unwrap()
                    .to_string()
            });

            miscellaneous::save_to_file(csv, file_name, crate::miscellaneous::WriteMode::Append).unwrap_or_else(|_| {
                crate::miscellaneous::write_to_log(&format!("Failed to save {}", &file_name))
                    .unwrap();
            });
        }
    }
    Ok(())
}
