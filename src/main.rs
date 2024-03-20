mod logging;
mod login;
mod miscellaneous;
mod post;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_exe: std::path::PathBuf = std::env::current_exe()?;
    let login_data: login::LoginData = login::read_login_data(&format!(
        "{}/config_{}.txt",
        current_exe.parent().unwrap().to_str().unwrap(),
        current_exe.file_name().unwrap().to_str().unwrap()
    ))
    .expect("Failed to get login data.");
    let login_page: String = format!("http://{}/accounts/m_login/", login_data.return_ip());
    let logger_page: String = format!("http://{}/m_logger/1/", login_data.return_ip());
    const HOUSE_CONFIG: &str = "house_config.txt";

    let client: reqwest::Client = match login::create_client(&login_data, &login_page).await {
        Ok(client) => client,
        Err(e) => {
            crate::miscellaneous::write_to_log(
                format!("ERROR Failed to create client: {}", e).as_str(),
                login_data.return_target_dict(),
            )?;
            std::process::exit(1);
        }
    };

    let house: logging::House = match logging::readout_logging_page(&client, &logger_page)
        .await{
            Ok(data) => data,
            Err(e) => {
                            crate::miscellaneous::write_to_log(
                format!("ERROR Failed to readout logging page: {}", e).as_str(),
                login_data.return_target_dict(),
            )?;
            std::process::exit(1);
            },
        };

    crate::miscellaneous::write_to_log(
        "INFO Script execution started.",
        login_data.return_target_dict(),
    )?;

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
