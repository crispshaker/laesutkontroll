#[allow(unused)]
pub async fn request_room(
    /* Send POST request bundled per room */
    client: &reqwest::Client,
    room: &crate::logging::Room,
    start_time: &str,
    end_time: &str,
    url: &str,
    key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut post_form: Vec<(&str, &str)> = vec![
        ("von", start_time),
        ("bis", end_time),
        ("csrfmiddlewaretoken", key),
    ];

    post_form.extend(room.sensors.iter().map(|sensor| (sensor.as_str(), "on")));

    Ok(client
        .post(url)
        .form(&post_form)
        .send()
        .await?
        .text()
        .await?)
}

pub async fn request_sensor(
    /* Send POST for a single sensor */
    client: &reqwest::Client,
    sensor: &str,
    start_time: &str,
    end_time: &str,
    url: &str,
    key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let post_form: Vec<(&str, &str)> = vec![
        ("von", start_time),
        ("bis", end_time),
        ("csrfmiddlewaretoken", key),
        (sensor, "on"),
    ];

    Ok(client
        .post(url)
        .form(&post_form)
        .send()
        .await?
        .text()
        .await?)
}
