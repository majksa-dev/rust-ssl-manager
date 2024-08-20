mod local;

#[tokio::main]
async fn main() {
    essentials::install();
    local::run().await;
}
