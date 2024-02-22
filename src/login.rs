struct CustomError<'a> {
    message: &'a str,
}

impl<'a> std::fmt::Display for CustomError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl<'a> std::error::Error for CustomError<'a> {}

impl<'a> std::fmt::Debug for CustomError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomError")
            .field("message", &self.message)
            .finish()
    }
}

impl<'a> CustomError<'a> {
    fn from(message: &str) -> CustomError {
        CustomError { message }
    }
}

pub struct LoginData {
    username: String,
    password: String,
    target_dict: String,
    ip: String,
}

impl LoginData {
    pub fn new() -> LoginData {
        LoginData {
            username: String::new(),
            password: String::new(),
            target_dict: String::new(),
            ip: String::new(),
        }
    }

    fn set_username(&mut self, username: &str) {
        self.username = username.to_owned();
    }

    fn set_password(&mut self, password: &str) {
        self.password = password.to_owned();
    }

    fn set_target_dict(&mut self, target_dict: &str) {
        self.target_dict = target_dict.to_owned();
    }

    fn set_ip(&mut self, ip: &str) {
        self.ip = ip.to_owned();
    }

    pub fn return_target_dict(&self) -> &str {
        &self.target_dict
    }

    pub fn return_ip(&self) -> &str {
        &self.ip
    }
}

pub async fn create_client<'a>(
    /* Create authenticated Client */
    login_data: &LoginData,
    login_page: &str,
) -> Result<reqwest::Client, Box<dyn std::error::Error>> {
    let cookie_jar: std::sync::Arc<reqwest::cookie::Jar> =
        std::sync::Arc::new(reqwest::cookie::Jar::default());
    let client: reqwest::Client = reqwest::ClientBuilder::new()
        .cookie_provider(std::sync::Arc::clone(&cookie_jar))
        .build()?;

    let raw_html: scraper::Html =
        scraper::Html::parse_document(&client.get(login_page).send().await?.text().await?);
    let csrf_selector: scraper::Selector =
        scraper::Selector::parse("input[name=csrfmiddlewaretoken][type=hidden]")?;

    let csrf_value = raw_html
        .select(&csrf_selector)
        .next()
        .and_then(|input| input.value().attr("value"))
        .ok_or("CSRF token not found")?;

    let login_response: &reqwest::Response = &client
        .post(login_page)
        .form(&[
            ("mail", login_data.username.clone()),
            ("pw", login_data.password.clone()),
            ("csrfmiddlewaretoken", csrf_value.to_owned()),
        ])
        .send()
        .await?;

    if !login_response.status().is_success() {
        eprintln!("Login failed!");
        return Err(Box::new(CustomError::from("Login failed")));
    }

    Ok(client)
}

pub fn read_login_data(file_path: &str) -> Result<LoginData, Box<dyn std::error::Error>> {
    let mut output: LoginData = LoginData::new();

    if let Ok(file) = std::fs::File::open(file_path) {
        let username_regex: regex::Regex = regex::Regex::new(r"Username:\s*(\S+)")?;
        let password_regex: regex::Regex = regex::Regex::new(r"Password:\s*(\S+)")?;
        let target_dict_regex: regex::Regex = regex::Regex::new(r"Log location:\s*(\S+)")?;
        let ip_regex: regex::Regex = regex::Regex::new(r"IP address:\s*(\S+)")?;

        fn process_line<T: FnMut(&str)>(line: &str, regex: &regex::Regex, mut action: T) {
            if let Some(captures) = regex.captures(line) {
                if let Some(value) = captures.get(1) {
                    action(value.as_str());
                }
            }
        }

        for line in std::io::BufRead::lines(std::io::BufReader::new(file)) {
            let line: String = line?;

            process_line(&line, &username_regex, |username| {
                output.set_username(username);
            });
            process_line(&line, &password_regex, |password| {
                output.set_password(password);
            });
            process_line(&line, &target_dict_regex, |target| {
                output.set_target_dict(target);
            });
            process_line(&line, &ip_regex, |ip| {
                output.set_ip(ip);
            });
        }
    };
    let mut save_to_file: bool = false;

    fn get_input(prompt: &str, field: &mut String) -> std::io::Result<()> {
        print!("{}", prompt);
        std::io::Write::flush(&mut std::io::stdout())?;

        if let Err(err) = std::io::stdin().read_line(field) {
            eprintln!("Error reading input: {}", err);
        }
        Ok(())
    }

    while output.username.is_empty() {
        save_to_file = true;
        get_input("Please input your username: ", &mut output.username)?;
    }

    while output.password.is_empty() {
        save_to_file = true;
        get_input("Please input your password: ", &mut output.password)?;
    }

    while output.ip.is_empty() {
        save_to_file = true;
        get_input("Please input IP address: ", &mut output.ip)?;
    }

    while output.target_dict.is_empty() {
        save_to_file = true;
        get_input("Please input Log location: ", &mut output.target_dict)?;
    }

    output.password = output.password.trim().to_owned();
    output.username = output.username.trim().to_owned();
    output.target_dict = output.target_dict.trim().to_owned();
    output.ip = output.ip.trim().to_owned();
    output.target_dict = output.target_dict.trim().to_owned();

    if !output.target_dict.ends_with('/') {
        output.target_dict.push('/');
    }

    if save_to_file {
        std::fs::write(
            file_path,
            format!(
                "Username:{}\nPassword:{}\nIP address:{}\nLog location:{}",
                output.username, output.password, output.ip, output.target_dict,
            ),
        )
        .expect("Failed to save user credentials");
    }
    Ok(output)
}
