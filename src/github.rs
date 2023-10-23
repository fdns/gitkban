use octocrab::{
    models::{issues::Issue, pulls::PullRequest},
    Page,
};
use serde_json;
use url::Url;

pub struct Github {
    instance: octocrab::Octocrab,
    track_user: String,
}

impl Github {
    pub fn new(token: String, track_user: String) -> Self {
        Github {
            instance: octocrab::OctocrabBuilder::new()
                .personal_token(token)
                .build()
                .unwrap(),
            track_user,
        }
    }

    pub async fn get_issues(&self) -> Result<Page<Issue>, Box<dyn std::error::Error>> {
        self.instance
            .search()
            .issues_and_pull_requests(&format!("is:open is:pr author:{}", self.track_user))
            .sort("created_at")
            .order("desc")
            .send()
            .await
            .map_err(|e| e.into())
    }

    pub async fn get_pull_from_url(
        &self,
        url: &Url,
    ) -> Result<PullRequest, Box<dyn std::error::Error>> {
        self.instance
            .get::<PullRequest, &Url, ()>(url, None)
            .await
            .map_err(|e| e.into())
    }

    pub async fn update_issue(
        &self,
        url: &str,
        body: String,
    ) -> Result<PullRequest, Box<dyn std::error::Error>> {
        //self.instance.pulls(owner, repo).update(pull_number).body(body).send()
        let data = serde_json::json!({
            "body": body
        });

        tracing::debug!("Updating body to: {}", data.to_string());
        self.instance
            .patch::<PullRequest, &str, serde_json::Value>(url, Some(&data))
            .await
            .map_err(|e| e.into())
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
        assert_eq!(issues.items.get(0).unwrap().id, 1638578736.into());

        mock.assert_async().await;
        Ok(())
    }
}
