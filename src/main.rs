mod kanbanize;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // construct a subscriber that prints formatted traces to stdout
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber).unwrap();
    dotenv::dotenv().ok().unwrap();

    let kanbanize = kanbanize::Kanbanize::new(
        std::env::var("KANBANIZE_BASE_PATH")
            .expect("KANBANIZE_BASE_PATH must be set")
            .as_str(),
        std::env::var("KANBANIZE_API_KEY")
            .expect("KANBANIZE_API_KEY must be set")
            .as_str(),
    );

    let card = kanbanize.find_by_id(13751 * 100).await?;
    process_card(card);
    return Ok(());
}

fn process_card(card: kanbanize_api::models::GetCard200Response) {
    let description =
        html2md::parse_html(card.data.unwrap().description.unwrap_or_default().as_str());
    tracing::info!("Got card:\n{}", description);
}
