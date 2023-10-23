use std::time::Duration;

use octocrab::models::{issues::Issue, pulls::PullRequest};

use crate::{github::Github, kanbanize::Kanbanize};

pub async fn run(github: Github, kanbanize: Kanbanize) {
    let mut interval = tokio::time::interval(Duration::from_secs(5 * 60));
    loop {
        interval.tick().await;
        process(&github, &kanbanize).await;
    }
}

async fn process(github: &Github, kanbanize: &Kanbanize) {
    let issues = github.get_issues().await.unwrap();
    for issue in issues.items {
        tracing::debug!("Received issue {:?}", issue);
        if let Err(e) = process_issue(github, kanbanize, &issue).await {
            tracing::error!("Error processing issue {}: {}", issue.url, e);
        }
        break;
    }
}

async fn process_issue(
    github: &Github,
    kanbanize: &Kanbanize,
    issue: &Issue,
) -> Result<(), Box<dyn std::error::Error>> {
    // Don't do anything if the issue has a body
    if issue.body.clone().unwrap_or_default() != String::default() {
        return Ok(());
    }

    let pull = github
        .get_pull_from_url(&issue.pull_request.clone().unwrap().url)
        .await?;

    // Repository must be private
    if !pull.clone().base.repo.unwrap().private.unwrap_or_default() {
        return Ok(());
    }

    // Try to grab the ID from the branch
    let re = regex::Regex::new(r"\d+(\.\d+)?").unwrap();
    if let Some(cap) = re.captures(pull.head.ref_field.as_str()) {
        if let Some(id) = cap.get(0) {
            let card_id = id.as_str().parse()?;
            // Update issue body with card ID
            process_issue_with_card_id(github, kanbanize, &pull, card_id).await?;
        }
    }

    Ok(())
}

async fn process_issue_with_card_id(
    github: &Github,
    kanbanize: &Kanbanize,
    pull: &PullRequest,
    card_id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get card
    let card = kanbanize.find_by_id(card_id).await?;
    let body = card_to_markdown(card);

    // Update Pull request
    github.update_issue(&pull.url, body).await?;

    Ok(())
}

fn card_to_markdown(card: kanbanize_api::models::GetCard200Response) -> String {
    html2md::parse_html(card.data.unwrap().description.unwrap_or_default().as_str())
}
