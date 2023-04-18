mod common;

mod request {
    use simple_actor::run_actor;

    use super::common::*;

    #[tokio::test]
    async fn echo() {
        let actor_handle = run_actor(TestActor {}, 1);
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
        let actor_handle = run_actor(TestActor {}, 1);
        let response = actor_handle
            .requestor
            .request(Message::AddOne(NUMBER))
            .await
            .unwrap();
        assert_eq!(response.unwrap(), ResponseMsg::Number(NUMBER + 1));
    }

    #[tokio::test]
    async fn respond_with_none() {
        const NUMBER: i32 = 6;
        let actor_handle = run_actor(TestActor {}, 1);
        let response = actor_handle
            .requestor
            .request(Message::IgnoreThis)
            .await
            .unwrap();
        assert_eq!(response, None);
    }
}
