mod common;

mod abort_join {

    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use atticus::actor;

    use super::common::*;

    #[tokio::test]
    async fn returns_error_on_request() {
        let actor_handle = actor::run(TestActor::default(), 1);

        actor_handle.abort();

        let response = actor_handle.requestor.request(Message::IgnoreThis).await;

        assert!(response.is_err());
    }

    #[tokio::test]
    async fn joins_before_a_timeout() {
        let actor_handle = actor::run(TestActor::default(), 1);

        actor_handle.abort();

        let timed_out = Arc::new(Mutex::new(false));
        let timed_out2 = timed_out.clone();

        let timeout_handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let mut t = timed_out2.lock().unwrap();
            *t = true;
        });

        // Expect that the actor has exited before the timeout
        // We have to move it into this block so we relinquish the lock
        // for `timed_out`
        {
            actor_handle.join().await;

            assert!(!*timed_out.lock().unwrap());
        }

        timeout_handle.await.unwrap();
    }
}
