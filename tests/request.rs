mod common;

mod request {
    use atticus::actor;

    use super::common::*;

    #[tokio::test]
    async fn echo() {
        let actor_handle = actor::run(TestActor::default(), 1);
        let response = actor_handle
            .request(Message::Echo("ping".to_owned()))
            .await
            .unwrap();
        assert_eq!(response, ResponseMsg::Echo("ping".to_owned()));
    }

    #[tokio::test]
    async fn add_one() {
        const NUMBER: i32 = 6;
        let actor_handle = actor::run(TestActor::default(), 1);
        let response = actor_handle
            .request(Message::AddOne(NUMBER))
            .await
            .unwrap();
        assert_eq!(response, ResponseMsg::Number(NUMBER + 1));
    }

    #[tokio::test]
    async fn respond_with_none() {
        let actor_handle = actor::run(TestActor::default(), 1);
        let response = actor_handle
            .request(Message::IgnoreThis)
            .await
            .unwrap();
        assert_eq!(response, ResponseMsg::None);
    }

    #[tokio::test]
    async fn multiple_requestors() {
        const NUMBER: i32 = 6;
        let actor_handle = actor::run(TestActor::default(), 1);

        let requestor2 = actor_handle.addr.clone();

        let req2 =
            tokio::spawn(async move { requestor2.request(Message::AddOne(NUMBER)).await.unwrap() });

        let response = actor_handle
            .request(Message::Echo("ping!".to_string()))
            .await
            .unwrap();

        assert_eq!(response, ResponseMsg::Echo("ping!".to_string()));
        assert_eq!(req2.await.unwrap(), ResponseMsg::Number(7));
    }

    #[tokio::test]
    async fn send_an_event() {
        let actor_handle = actor::run(TestActor::default(), 1);
        actor_handle.event(Message::IgnoreThis).await.unwrap();

        let last_request = actor_handle
            .request(Message::GetLastRequest)
            .await
            .unwrap();

        assert_eq!(
            last_request,
            ResponseMsg::LastRequest(Some(Message::IgnoreThis))
        );
    }
}
