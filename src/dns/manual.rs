use std::io;

use anyhow::Result;

#[derive(Debug, Default)]
pub struct Client;

impl Client {
    pub fn new() -> Self {
        Self
    }
}

impl super::Client for Client {
    fn wrap(self) -> super::Clients {
        super::Clients::Manual(self)
    }

    async fn create_record(
        &self,
        domain: String,
        name: String,
        value: String,
    ) -> Result<(String, String)> {
        println!("Please set the following DNS record then press the Return key:");
        println!("{name}.{domain} IN TXT {value}");
        io::stdin().read_line(&mut String::new()).unwrap();
        Ok((domain, name))
    }

    async fn delete_record(&self, domain_id: String, dns_id: String) -> Result<()> {
        println!("Please remove the following DNS record then press the Return key:");
        println!("{dns_id}.{domain_id} IN TXT");
        io::stdin().read_line(&mut String::new()).unwrap();
        Ok(())
    }
}
