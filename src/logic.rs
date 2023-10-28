use std::time::Duration;

use octocrab::models::{issues::Issue, pulls::PullRequest};

use crate::{github::Github, kanbanize::Kanbanize};

pub struct Service {
    github: Github,
    kanbanize: Kanbanize,
    owner_whitelist: String,
}

impl Service {
    pub fn new(github: Github, kanbanize: Kanbanize, owner_whitelist: String) -> Self {
        Self {
            github,
            kanbanize,
            owner_whitelist,
        }
    }

    pub async fn run(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(5 * 60));
        loop {
            interval.tick().await;
            tracing::info!("Checking for new PR's");
            if let Err(e) = self.process().await {
                tracing::error!("Error processing issues: {}", e)
            };
        }
    }

    async fn process(&self) -> Result<(), Box<dyn std::error::Error>> {
        let issues = self.github.get_issues().await?;
        for issue in issues.items {
            tracing::debug!("Received issue {:?}", issue);
            if let Err(e) = self.process_issue(&issue).await {
                tracing::error!("Error processing issue {}: {}", issue.url, e);
            }
        }

        Ok(())
    }

    async fn process_issue(&self, issue: &Issue) -> Result<(), Box<dyn std::error::Error>> {
        // Don't do anything if the issue has a body
        if issue.body.clone().unwrap_or_default() != String::default() {
            return Ok(());
        }

        let pull = self
            .github
            .get_pull_from_url(&issue.pull_request.clone().unwrap().url)
            .await?;

        // Repository must be private
        if !pull.clone().base.repo.unwrap().private.unwrap_or_default() {
            return Ok(());
        }

        if pull.clone().base.repo.unwrap().owner.unwrap().login != self.owner_whitelist {
            return Ok(());
        }

        // Try to grab the ID from the branch
        let re = regex::Regex::new(r"\d+(\.\d+)?").unwrap();
        if let Some(cap) = re.captures(pull.head.ref_field.as_str()) {
            if let Some(id) = cap.get(0) {
                let card_id = id.as_str().parse()?;
                // Update issue body with card ID
                self.process_issue_with_card_id(&pull, card_id).await?;
            }
        }

        Ok(())
    }

    async fn process_issue_with_card_id(
        &self,
        pull: &PullRequest,
        card_id: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Updating pull request {}", pull.url);
        // Get card
        let card = self.kanbanize.find_by_id(card_id).await?;
        let body = card_to_markdown(card);

        // Update Pull request
        self.github.update_issue(&pull.url, body).await?;

        Ok(())
    }
}

fn card_to_markdown(card: kanbanize_api::models::GetCard200Response) -> String {
    html2md::parse_html(card.data.unwrap().description.unwrap_or_default().as_str())
}
