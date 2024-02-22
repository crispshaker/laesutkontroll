mod logging;
mod login;
mod miscellaneous;
mod post;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let login_data: login::LoginData =
        login::read_login_data("config_laesutkontroll.txt").expect("Failed to get login data.");
    let login_page: String = format!("http://{}/accounts/m_login/", login_data.return_ip());
    let logger_page: String = format!("http://{}/m_logger/1/", login_data.return_ip());
    const HOUSE_CONFIG: &str = "house_config.txt";

    let client: reqwest::Client = login::create_client(&login_data, &login_page)
        .await
        .expect("Failed to create client.");

    let house: logging::House = logging::readout_logging_page(&client, &logger_page)
        .await
        .expect("Failed to get Sensors from logger page");

    miscellaneous::save_to_file(
        &house.to_string(),
        HOUSE_CONFIG,
        login_data.return_target_dict(),
        crate::miscellaneous::WriteMode::Overwrite,
    )
    .unwrap_or_else(|_| {
        crate::miscellaneous::write_to_log(
            &format!("ERROR Failed to save {}.", &HOUSE_CONFIG),
            login_data.return_target_dict(),
        )
        .unwrap();
    });

    for room in house.rooms {
        for sensor in room.sensors {
            let file_name: &str = &format!("{}.csv", sensor);
            let csv: &str = &post::request_sensor(
                &client,
                &sensor,
                &miscellaneous::read_last_log(file_name, login_data.return_target_dict()).unwrap(),
                &miscellaneous::current_time(),
                &logger_page,
                &house.key,
            )
            .await
            .unwrap_or_else(|_| {
                crate::miscellaneous::write_to_log(
                    &format!("ERROR Failed to get {}.", &sensor),
                    login_data.return_target_dict(),
                )
                .unwrap()
                .to_string()
            });

            miscellaneous::save_to_file(
                csv,
                file_name,
                login_data.return_target_dict(),
                crate::miscellaneous::WriteMode::Append,
            )
            .unwrap_or_else(|_| {
                crate::miscellaneous::write_to_log(
                    &format!("ERROR Failed to save {}.", &file_name),
                    login_data.return_target_dict(),
                )
                .unwrap();
            });
        }
    }
    crate::miscellaneous::write_to_log(
        "INFO Script execution completed successfully.",
        login_data.return_target_dict(),
    )
    .unwrap();
    Ok(())
}
