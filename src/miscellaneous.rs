pub enum WriteMode {
    Append,
    Overwrite,
}

impl WriteMode {
    fn as_bool(&self) -> bool {
        match self {
            WriteMode::Append => true,
            WriteMode::Overwrite => false,
        }
    }
}

pub fn read_last_log(
    file_name: &str,
    file_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    /* Read last log entery from file */
    let mut timestamp: String = String::new();
    if let Ok(file) = std::fs::File::open(format!("{}/{}", file_path, file_name)) {
        if let Some(Ok(last_line)) = std::io::BufRead::lines(std::io::BufReader::new(file)).last() {
            timestamp = last_line
                .split(',')
                .next()
                .unwrap_or_default()
                .to_string()
                .replace(' ', "T");
        }
    }
    if timestamp.is_empty() {
        timestamp = (chrono::Local::now() - chrono::Duration::days(30))
            .format("%Y-%m-%dT00:00")
            .to_string()
    }

    Ok(timestamp)
}

pub fn current_time() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M").to_string()
}

pub fn save_to_file(
    data: &str,
    file_name: &str,
    file_path: &str,
    write_mode: WriteMode,
) -> Result<(), Box<dyn std::error::Error>> {
    /* Create/Append to data to */
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .append(write_mode.as_bool())
        .create(true)
        .truncate(!write_mode.as_bool())
        .open(format!("{}/{}", file_path, file_name))?;

    let cleaned_data = data
        .lines()
        .filter(|&line| line != "Zeit,Name,Wert")
        .map(|line| format!("{}\n", line).to_string())
        .collect::<String>();

    std::io::Write::write_all(&mut file, cleaned_data.as_bytes())?;
    Ok(())
}

pub fn write_to_log(
    error_message: &str,
    file_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    const FILE_NAME: &str = "log.txt";
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(format!("{}/{}", file_path, FILE_NAME))?;

    let error_message: String = format!(
        "\n{} {}",
        chrono::Local::now().format("%Y.%m.%d %H:%M:%S"),
        error_message
    );

    std::io::Write::write_all(&mut file, error_message.as_bytes())?;
    Ok(error_message)
}
