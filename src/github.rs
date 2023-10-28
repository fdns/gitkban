use serde_json;
use url::Url;

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub struct PullRequest {
    pub url: Url,
    pub is_private: bool,
    pub repository_owner: String,
    pub head_reference: String,
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub url: Url,
    pub body: String,
    pub pull_request_url: Option<Url>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait GithubApi {
    async fn get_issues(&self) -> Result<Vec<Issue>>;
    async fn get_pull_from_url(&self, url: &Url) -> Result<PullRequest>;
    async fn update_pull_request(&self, url: &Url, body: String) -> Result<PullRequest>;
}

pub struct Github {
    instance: octocrab::Octocrab,
    track_user: String,
}

impl Github {
    pub fn new(token: String, track_user: String) -> Box<dyn GithubApi> {
        Box::new(Github {
            instance: octocrab::OctocrabBuilder::new()
                .personal_token(token)
                .build()
                .unwrap(),
            track_user,
        })
    }
}

#[async_trait::async_trait]
impl GithubApi for Github {
    async fn get_issues(&self) -> Result<Vec<Issue>> {
        self.instance
            .search()
            .issues_and_pull_requests(&format!("is:open is:pr author:{}", self.track_user))
            .sort("created_at")
            .order("desc")
            .send()
            .await
            .map_err(|e| e.to_string())
            .map(|issues| {
                issues
                    .into_iter()
                    .map(|issue| issue.try_into())
                    .collect::<Result<Vec<Issue>>>()
            })?
    }

    async fn get_pull_from_url(&self, url: &Url) -> Result<PullRequest> {
        self.instance
            .get::<octocrab::models::pulls::PullRequest, &Url, ()>(url, None)
            .await
            .map_err(|e| Into::<Error>::into(e))
            .map(|v| v.try_into())?
    }

    async fn update_pull_request(&self, url: &Url, body: String) -> Result<PullRequest> {
        let data = serde_json::json!({
            "body": body
        });

        tracing::debug!("Updating body to: {}", data.to_string());
        self.instance
            .patch::<octocrab::models::pulls::PullRequest, &Url, serde_json::Value>(
                url,
                Some(&data),
            )
            .await
            .map_err(|e| Into::<Error>::into(e))
            .map(|v| v.try_into())?
    }
}

impl TryFrom<octocrab::models::issues::Issue> for Issue {
    type Error = Error;
    fn try_from(value: octocrab::models::issues::Issue) -> Result<Self, Self::Error> {
        Ok(Self {
            url: value.url,
            body: value.body.unwrap_or_default(),
            pull_request_url: value.pull_request.map(|pull| pull.url.clone()),
        })
    }
}

impl TryFrom<octocrab::models::pulls::PullRequest> for PullRequest {
    type Error = Error;
    fn try_from(value: octocrab::models::pulls::PullRequest) -> Result<Self, Self::Error> {
        return Ok(Self {
            url: value.url.parse()?,
            is_private: value
                .base
                .clone()
                .repo
                .ok_or("repository of pull request not found")?
                .private
                .unwrap_or(true),
            repository_owner: value
                .base
                .repo
                .ok_or("repository of pull request not found")?
                .owner
                .ok_or("owner not found")?
                .login,
            head_reference: value.head.ref_field,
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_get_issues() -> testresult::TestResult {
        let subscriber = tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .with_test_writer()
            .finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();

        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock(
                "GET",
                "/search/issues?q=is%3Aopen+is%3Apr+author%3Atrackuser&sort=created_at&order=desc",
            )
            .with_status(200)
            .with_body(
                r#"{
                "total_count": 15,
                "incomplete_results": false,
                "items": [
                    {
                        "url": "https://api.github.com/repos/oh-my-fish/plugin-pyenv/issues/11",
                        "repository_url": "https://api.github.com/repos/oh-my-fish/plugin-pyenv",
                        "labels_url": "https://api.github.com/repos/oh-my-fish/plugin-pyenv/issues/11/labels{/name}",
                        "comments_url": "https://api.github.com/repos/oh-my-fish/plugin-pyenv/issues/11/comments",
                        "events_url": "https://api.github.com/repos/oh-my-fish/plugin-pyenv/issues/11/events",
                        "html_url": "https://github.com/oh-my-fish/plugin-pyenv/pull/11",
                        "id": 1638578736,
                        "node_id": "PR_kwDOAi6Z6M5MynU1",
                        "number": 11,
                        "title": "fix docs: broken link in README",
                        "user": {
                            "login": "davidxia",
                            "id": 480621,
                            "node_id": "MDQ6VXNlcjQ4MDYyMQ==",
                            "avatar_url": "https://avatars.githubusercontent.com/u/480621?v=4",
                            "gravatar_id": "",
                            "url": "https://api.github.com/users/davidxia",
                            "html_url": "https://github.com/davidxia",
                            "followers_url": "https://api.github.com/users/davidxia/followers",
                            "following_url": "https://api.github.com/users/davidxia/following{/other_user}",
                            "gists_url": "https://api.github.com/users/davidxia/gists{/gist_id}",
                            "starred_url": "https://api.github.com/users/davidxia/starred{/owner}{/repo}",
                            "subscriptions_url": "https://api.github.com/users/davidxia/subscriptions",
                            "organizations_url": "https://api.github.com/users/davidxia/orgs",
                            "repos_url": "https://api.github.com/users/davidxia/repos",
                            "events_url": "https://api.github.com/users/davidxia/events{/privacy}",
                            "received_events_url": "https://api.github.com/users/davidxia/received_events",
                            "type": "User",
                            "site_admin": false
                        },
                        "labels": [
                    
                        ],
                        "state": "open",
                        "locked": false,
                        "assignee": null,
                        "assignees": [
                    
                        ],
                        "milestone": null,
                        "comments": 0,
                        "created_at": "2023-03-24T01:31:01Z",
                        "updated_at": "2023-03-24T01:31:01Z",
                        "closed_at": null,
                        "author_association": "NONE",
                        "active_lock_reason": null,
                        "draft": false,
                        "pull_request": {
                            "url": "https://api.github.com/repos/oh-my-fish/plugin-pyenv/pulls/11",
                            "html_url": "https://github.com/oh-my-fish/plugin-pyenv/pull/11",
                            "diff_url": "https://github.com/oh-my-fish/plugin-pyenv/pull/11.diff",
                            "patch_url": "https://github.com/oh-my-fish/plugin-pyenv/pull/11.patch",
                            "merged_at": null
                        },
                        "body": null,
                        "reactions": {
                            "url": "https://api.github.com/repos/oh-my-fish/plugin-pyenv/issues/11/reactions",
                            "total_count": 0,
                            "+1": 0,
                            "-1": 0,
                            "laugh": 0,
                            "hooray": 0,
                            "confused": 0,
                            "heart": 0,
                            "rocket": 0,
                            "eyes": 0
                        },
                        "timeline_url": "https://api.github.com/repos/oh-my-fish/plugin-pyenv/issues/11/timeline",
                        "performed_via_github_app": null,
                        "state_reason": null,
                        "score": 1.0
                    }
                ]
            }
        "#,
            )
            .create_async()
            .await;

        let github = Github {
            instance: octocrab::OctocrabBuilder::new()
                .base_uri(server.url())?
                .build()?,
            track_user: "trackuser".to_string(),
        };

        let issues = github.get_issues().await?;
        assert_eq!(
            issues[0].url,
            Url::parse("https://api.github.com/repos/oh-my-fish/plugin-pyenv/issues/11").unwrap()
        );
        assert_eq!(
            issues[0].pull_request_url.as_ref().unwrap(),
            &Url::parse("https://api.github.com/repos/oh-my-fish/plugin-pyenv/pulls/11").unwrap()
        );

        mock.assert_async().await;
        Ok(())
    }
}
