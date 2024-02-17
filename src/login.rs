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

pub struct LoginData<'a> {
    username: &'a str,
    password: &'a str,
}

#[allow(dead_code)]
impl<'a> LoginData<'a> {
    pub fn new() -> LoginData<'static> {
        LoginData {
            username: "",
            password: "",
        }
    }

    pub const fn from(username: &'a str, password: &'a str) -> LoginData<'a> {
        LoginData { username, password }
    }

    pub fn change_username(&mut self, username: &'a str) {
        self.username = username;
    }

    pub fn change_password(&mut self, password: &'a str) {
        self.password = password;
    }
}

pub async fn create_client<'a>(
    /* Create authenticated Client */
    login_data: &LoginData<'a>,
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
            ("mail", login_data.username),
            ("pw", login_data.password),
            ("csrfmiddlewaretoken", csrf_value),
        ])
        .send()
        .await?;

    if !login_response.status().is_success() {
        eprintln!("Login failed!");
        return Err(Box::new(CustomError::from("Login failed")));
    }

    Ok(client)
}
