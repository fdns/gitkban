use std::time::Duration;

use crate::{
    github::{GithubApi, Issue, PullRequest},
    kanbanize::{Card, KanbanizeApi},
};

pub struct Service {
    github: Box<dyn GithubApi>,
    kanbanize: Box<dyn KanbanizeApi>,
    owner_whitelist: String,
}

impl Service {
    pub fn new(
        github: Box<dyn GithubApi>,
        kanbanize: Box<dyn KanbanizeApi>,
        owner_whitelist: String,
    ) -> Self {
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
        for issue in issues.iter() {
            tracing::debug!("Received issue {:?}", issue);
            if let Err(e) = self.process_issue(&issue).await {
                tracing::error!("Error processing issue {}: {}", issue.url, e);
            }
        }

        Ok(())
    }

    async fn process_issue(&self, issue: &Issue) -> Result<(), Box<dyn std::error::Error>> {
        // Don't do anything if the issue has a body
        if issue.body != String::default() {
            return Ok(());
        }

        let pull = self
            .github
            .get_pull_from_url(
                &issue
                    .pull_request_url
                    .clone()
                    .ok_or("issue is not a pull request")?,
            )
            .await?;

        // Repository must be private
        if !pull.is_private {
            return Ok(());
        }

        if pull.repository_owner != self.owner_whitelist {
            return Ok(());
        }

        // Try to grab the ID from the branch
        let re = regex::Regex::new(r"\d+(\.\d+)?").unwrap();
        if let Some(cap) = re.captures(pull.head_reference.as_str()) {
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
        self.github.update_pull_request(&pull.url, body).await?;

        Ok(())
    }
}

fn card_to_markdown(card: Card) -> String {
    html2md::parse_html(card.description.as_str())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{github::MockGithubApi, kanbanize::MockKanbanizeApi};
    use url::Url;

    #[tokio::test]
    async fn test_process_no_issues() {
        let mut github = MockGithubApi::new();
        let kanbanize = MockKanbanizeApi::new();
        github
            .expect_get_issues()
            .return_once(|| Ok(Vec::<Issue>::default()));

        let logic = Service::new(Box::new(github), Box::new(kanbanize), "owner".to_string());
        let result = logic.process().await;
        assert!(result.is_ok(), "result error: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_process() -> testresult::TestResult {
        let mut github = MockGithubApi::new();
        let mut kanbanize = MockKanbanizeApi::new();
        let pull = PullRequest {
            url: Url::parse("https://example.com/pull")?,
            head_reference: String::from("request-123"),
            repository_owner: String::from("owner"),
            is_private: true,
        };
        github.expect_get_issues().return_once(|| {
            Ok(vec![Issue {
                url: Url::parse("https://example.com/issue")?,
                body: String::default(),
                pull_request_url: Some(Url::parse("https://example.com/pull")?),
            }])
        });

        let pull1 = pull.clone();
        github
            .expect_get_pull_from_url()
            .with(mockall::predicate::eq(Url::parse(
                "https://example.com/pull",
            )?))
            .return_once(move |_| Ok(pull1));

        kanbanize
            .expect_find_by_id()
            .with(mockall::predicate::eq(123))
            .return_once(|_| {
                Ok(Card {
                    description: String::from("description"),
                })
            });

        github
            .expect_update_pull_request()
            .with(
                mockall::predicate::eq(Url::parse("https://example.com/pull")?),
                mockall::predicate::eq(String::from("description")),
            )
            .return_once(|_, _| Ok(pull));

        let logic = Service::new(Box::new(github), Box::new(kanbanize), "owner".to_string());
        let result = logic.process().await;
        assert!(result.is_ok(), "result error: {:?}", result.err());
        Ok(())
    }

    #[tokio::test]
    async fn test_process_ignore_not_owner_whitelist() -> testresult::TestResult {
        let mut github = MockGithubApi::new();
        let kanbanize = MockKanbanizeApi::new();
        let pull = PullRequest {
            url: Url::parse("https://example.com/pull")?,
            head_reference: String::from("request-123"),
            repository_owner: String::from("not owner"),
            is_private: true,
        };
        github.expect_get_issues().return_once(|| {
            Ok(vec![Issue {
                url: Url::parse("https://example.com/issue")?,
                body: String::default(),
                pull_request_url: Some(Url::parse("https://example.com/pull")?),
            }])
        });

        let pull1 = pull.clone();
        github
            .expect_get_pull_from_url()
            .with(mockall::predicate::eq(pull.url.clone()))
            .return_once(move |_| Ok(pull1));

        let logic = Service::new(Box::new(github), Box::new(kanbanize), "owner".to_string());
        let result = logic.process().await;
        assert!(result.is_ok(), "result error: {:?}", result.err());
        Ok(())
    }

    #[tokio::test]
    async fn test_process_ignore_public() -> testresult::TestResult {
        let mut github = MockGithubApi::new();
        let kanbanize = MockKanbanizeApi::new();
        let pull = PullRequest {
            url: Url::parse("https://example.com/pull")?,
            head_reference: String::from("request-123"),
            repository_owner: String::from("owner"),
            is_private: false,
        };
        github.expect_get_issues().return_once(|| {
            Ok(vec![Issue {
                url: Url::parse("https://example.com/issue")?,
                body: String::default(),
                pull_request_url: Some(Url::parse("https://example.com/pull")?),
            }])
        });

        let pull1 = pull.clone();
        github
            .expect_get_pull_from_url()
            .with(mockall::predicate::eq(Url::parse(
                "https://example.com/pull",
            )?))
            .return_once(move |_| Ok(pull1));

        let logic = Service::new(Box::new(github), Box::new(kanbanize), "owner".to_string());
        let result = logic.process().await;
        assert!(result.is_ok(), "result error: {:?}", result.err());
        Ok(())
    }

    #[tokio::test]
    async fn test_process_ignore_pull_with_body() -> testresult::TestResult {
        let mut github = MockGithubApi::new();
        let kanbanize = MockKanbanizeApi::new();
        github.expect_get_issues().return_once(|| {
            Ok(vec![Issue {
                url: Url::parse("https://example.com/issue")?,
                body: String::from("existing"),
                pull_request_url: Some(Url::parse("https://example.com/pull")?),
            }])
        });

        let logic = Service::new(Box::new(github), Box::new(kanbanize), "owner".to_string());
        let result = logic.process().await;
        assert!(result.is_ok(), "result error: {:?}", result.err());
        Ok(())
    }
}
