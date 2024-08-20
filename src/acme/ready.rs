use anyhow::{bail, Result};
use essentials::{error, info};
use instant_acme::{Order, OrderStatus};
use std::time::Duration;
use tokio::time::sleep;

pub(super) trait Ready {
    async fn ready(&mut self) -> Result<()>
    where
        Self: Sized;
}

impl Ready for Order {
    async fn ready(&mut self) -> Result<()> {
        let mut tries = 1u8;
        let mut delay = Duration::from_secs(1);
        loop {
            sleep(delay).await;
            let state = self.refresh().await?;
            if let OrderStatus::Ready | OrderStatus::Invalid = state.status {
                info!("order state: {:#?}", state);
                if state.status == OrderStatus::Ready {
                    return Ok(());
                }
                bail!("order is invalid");
            }

            delay *= 2;
            tries += 1;
            match tries < 10 {
                true => info!(?state, tries, "order is not ready, waiting {delay:?}"),
                false => {
                    error!(tries, "order is not ready: {state:#?}");
                    bail!("order is not ready");
                }
            }
        }
    }
}
