#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    riot_token: &'static str,
}

impl Client {
    pub fn try_new() -> anyhow::Result<Self> {
        let riot_token =
            std::env::var("RIOT_TOKEN").map_err(|_| anyhow::anyhow!("No RIOT_TOKEN"))?;
        Ok(Self {
            client: reqwest::Client::new(),
            riot_token: riot_token.leak(),
        })
    }

    pub fn get_with_riot_token(&self, url: &str) -> reqwest::RequestBuilder {
        self.client.get(url).header("X-riot-token", self.riot_token)
    }
}
