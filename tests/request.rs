mod common;

mod request {
    use simple_actor::run_actor;

    use super::common::*;

    #[tokio::test]
    async fn echo() {
        let actor_handle = run_actor(TestActor::default(), 1);
        let response = actor_handle
            .requestor
            .request(Message::Echo("ping".to_owned()))
            .await
            .unwrap();
        assert_eq!(response.unwrap(), ResponseMsg::Echo("ping".to_owned()));
    }

    #[tokio::test]
    async fn add_one() {
        const NUMBER: i32 = 6;
        let actor_handle = run_actor(TestActor::default(), 1);
        let response = actor_handle
            .requestor
            .request(Message::AddOne(NUMBER))
            .await
            .unwrap();
        assert_eq!(response.unwrap(), ResponseMsg::Number(NUMBER + 1));
    }

    #[tokio::test]
    async fn respond_with_none() {
        let actor_handle = run_actor(TestActor::default(), 1);
        let response = actor_handle
            .requestor
            .request(Message::IgnoreThis)
            .await
            .unwrap();
        assert_eq!(response, None);
    }

    #[tokio::test]
    async fn multiple_requestors() {
        const NUMBER: i32 = 6;
        let actor_handle = run_actor(TestActor::default(), 1);

        let requestor2 = actor_handle.requestor.clone();

        let req2 =
            tokio::spawn(async move { requestor2.request(Message::AddOne(NUMBER)).await.unwrap() });

        let response = actor_handle
            .requestor
            .request(Message::Echo("ping!".to_string()))
            .await
            .unwrap();

        assert_eq!(response, Some(ResponseMsg::Echo("ping!".to_string())));
        assert_eq!(req2.await.unwrap(), Some(ResponseMsg::Number(7)));
    }

    #[tokio::test]
    async fn send_an_event() {
        let actor_handle = run_actor(TestActor::default(), 1);
        let event_task_handle = actor_handle.requestor.send_event(Message::IgnoreThis);

        // Let's do an await here to make sure that the actor has received the event before
        // proceding with the next request.
        // In actual practice, this is not really needed but may be used for niche cases.
        event_task_handle.await.unwrap().unwrap();

        let last_request = actor_handle
            .requestor
            .request(Message::GetLastRequest)
            .await
            .unwrap();

        assert_eq!(
            last_request.unwrap(),
            ResponseMsg::LastRequest(Some(Message::IgnoreThis))
        );
    }
}
