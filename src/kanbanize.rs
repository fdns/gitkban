use kanbanize_api::models::GetCard200Response;

#[derive(Debug)]
pub struct Kanbanize {
    config: kanbanize_api::apis::configuration::Configuration,
}

impl Kanbanize {
    pub fn new(base_path: &str, api_key: &str) -> Self {
        Kanbanize {
            config: Self::new_config(base_path, api_key),
        }
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

    #[tracing::instrument]
    pub async fn find_by_id(
        &self,
        id: i32,
    ) -> Result<GetCard200Response, Box<dyn std::error::Error>> {
        let result = kanbanize_api::apis::cards_api::get_card(&self.config, id).await;
        tracing::debug!("Response: {:?}", result);
        return result.map_err(|e| e.to_string().into());
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

        let card = card.data.ok_or("No card datar")?;
        assert_eq!(card.card_id.unwrap_or(0), 4321);
        assert_eq!(
            card.description.unwrap_or("".to_string()),
            "Card description".to_string()
        );

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
