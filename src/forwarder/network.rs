use std::error::Error;

pub struct Transmitter<'a> {
    client: reqwest::Client,
    url: &'a str,
}

impl Transmitter<'_> {
    pub fn new(url: &str) -> Transmitter {
        Transmitter {
            url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn transmit_data_chunk(&self, data: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let resp = self.client.post(self.url).body(data).send().await;
        println!("{:?}", resp);
        Ok(())
    }
}
