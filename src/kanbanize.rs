use kanbanize_api::models::GetCard200Response;

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait KanbanizeApi {
    async fn find_by_id(&self, id: i32) -> Result<Card>;
}

#[derive(Debug, Clone)]
pub struct Card {
    pub description: String,
}

#[derive(Debug)]
pub struct Kanbanize {
    config: kanbanize_api::apis::configuration::Configuration,
}

impl Kanbanize {
    pub fn new(base_path: &str, api_key: &str) -> Box<dyn KanbanizeApi> {
        Box::new(Kanbanize {
            config: Self::new_config(base_path, api_key),
        })
    }

    fn new_config(
        base_path: &str,
        api_key: &str,
    ) -> kanbanize_api::apis::configuration::Configuration {
        kanbanize_api::apis::configuration::Configuration {
            base_path: base_path.to_string(),
            api_key: Some(kanbanize_api::apis::configuration::ApiKey {
                key: api_key.to_string(),
                prefix: None,
            }),
            ..Default::default()
        }
    }
}

#[async_trait::async_trait]
impl KanbanizeApi for Kanbanize {
    async fn find_by_id(&self, id: i32) -> Result<Card> {
        let result = kanbanize_api::apis::cards_api::get_card(&self.config, id).await;
        tracing::debug!("Response: {:?}", result);
        return result
            .map_err(|e| Into::<Error>::into(e.to_string()))
            .map(|c| c.try_into())?;
    }
}

impl TryFrom<GetCard200Response> for Card {
    type Error = Error;
    fn try_from(value: GetCard200Response) -> Result<Self, Self::Error> {
        Ok(Self {
            description: value
                .data
                .ok_or("card has no data")?
                .description
                .unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_find_card() -> testresult::TestResult {
        let mut server = mockito::Server::new_async().await;

        // Create a mock
        let mock = server
            .mock("GET", "/cards/4321")
            .with_status(200)
            .match_header("apikey", "api_key")
            .with_body(
                serde_json::to_string(&kanbanize_api::models::GetCard200Response {
                    data: Some(Box::new(kanbanize_api::models::Card {
                        card_id: Some(4321),
                        description: Some("Card description".to_string()),
                        custom_id: None,
                        board_id: None,
                        workflow_id: None,
                        title: None,
                        owner_user_id: None,
                        type_id: None,
                        color: None,
                        section: None,
                        column_id: None,
                        lane_id: None,
                        position: None,
                        size: None,
                        priority: None,
                        deadline: None,
                        reporter: None,
                        created_at: None,
                        revision: None,
                        last_modified: None,
                        in_current_position_since: None,
                        is_blocked: None,
                        block_reason: None,
                        child_card_stats: None,
                        finished_subtask_count: None,
                        unfinished_subtask_count: None,
                        attachments: None,
                        custom_fields: None,
                        stickers: None,
                        tag_ids: None,
                        co_owner_ids: None,
                        watchers_ids: None,
                        annotations: None,
                        outcomes: None,
                        subtasks: None,
                        linked_cards: None,
                    })),
                })
                .unwrap(),
            )
            .create_async()
            .await;

        // Create client
        let client = Kanbanize::new(server.url().as_str(), "api_key");
        let card = client.find_by_id(4321).await?;

        assert_eq!(card.description, "Card description".to_string());

        mock.assert_async().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_card_not_found() -> testresult::TestResult {
        let mut server = mockito::Server::new_async().await;

        // Create a mock
        let mock = server
            .mock("GET", "/cards/4321")
            .match_header("apikey", "api_key")
            .with_status(404)
            .create_async()
            .await;

        // Create client
        let client = Kanbanize::new(server.url().as_str(), "api_key");
        let card = client.find_by_id(4321).await;
        assert!(card.is_err());
        assert_eq!(
            card.err().unwrap().to_string(),
            "error in response: status code 404 Not Found"
        );

        mock.assert_async().await;
        Ok(())
    }
}
