mod github;
mod kanbanize;
mod logic;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // construct a subscriber that prints formatted traces to stdout
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Load env vars
    dotenv::dotenv().ok().unwrap();

    // Load services
    let kanbanize = kanbanize::Kanbanize::new(
        std::env::var("KANBANIZE_BASE_PATH")
            .expect("KANBANIZE_BASE_PATH must be set")
            .as_str(),
        std::env::var("KANBANIZE_API_KEY")
            .expect("KANBANIZE_API_KEY must be set")
            .as_str(),
    );
    let github = github::Github::new(
        std::env::var("GITHUB_PERSONAL_TOKEN").expect("GITHUB_PERSONAL_TOKEN must be set"),
        std::env::var("GITHUB_TRACK_USER").expect("GITHUB_TRACK_USER must be set"),
    );

    // Run process
    logic::run(github, kanbanize).await;
    return Ok(());
}
