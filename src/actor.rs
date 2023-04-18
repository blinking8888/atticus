use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

use crate::error::Error;

pub struct Requestor<T>(
    mpsc::Sender<(
        <T as Actor>::Request,
        oneshot::Sender<Option<<T as Actor>::Response>>,
    )>,
)
where
    T: Actor + Sized,
    <T as Actor>::Request: Send;

impl<T> Clone for Requestor<T>
where
    T: Actor,
    <T as Actor>::Request: Send,
{
    fn clone(&self) -> Self {
        Requestor(self.0.clone())
    }
}

impl<T> Requestor<T>
where
    T: Actor + Sized,
    <T as Actor>::Request: Send,
    <T as Actor>::Response: Send,
{
    pub async fn request(
        &self,
        request: <T as Actor>::Request,
    ) -> Result<Option<<T as Actor>::Response>, Error> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Option<<T as Actor>::Response>>();
        self.0
            .send((request, rsp_tx))
            .await
            .map_err(|e| Error::RequestError(e.to_string()))?;

        rsp_rx
            .await
            .map_err(|e| Error::ResponseError(e.to_string()))
    }
}

pub struct Handle<T>
where
    T: Actor + Sized,
    <T as Actor>::Request: Send,
    <T as Actor>::Response: Send,
{
    pub requestor: Requestor<T>,
    pub handle: JoinHandle<()>,
}

impl<T> Handle<T>
where
    T: Actor + Sized,
    <T as Actor>::Request: Send,
    <T as Actor>::Response: Send,
{
    pub fn abort(&self) {
        self.handle.abort();
    }

    pub async fn join(self) {
        let _ = self.handle.await;
    }
}

pub trait Actor: Send + 'static {
    type Request;
    type Response;
    fn handle(&self, message: Self::Request) -> Option<Self::Response>;
}

pub fn run_actor<T>(actor: T, buffer: usize) -> Handle<T>
where
    T: Actor + Sized + Send,
    <T as Actor>::Request: Send,
    <T as Actor>::Response: Send,
{
    let (tx, mut rx) = mpsc::channel::<(
        <T as Actor>::Request,
        oneshot::Sender<Option<<T as Actor>::Response>>,
    )>(buffer);

    let handle = tokio::spawn(async move {
        while let Some((msg, rsp_tx)) = rx.recv().await {
            let response = actor.handle(msg);
            let _ = rsp_tx.send(response);
        }
    });

    Handle {
        handle,
        requestor: Requestor(tx),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestActor;

    enum Message {
        Echo(String),
        AddOne(i32),
        IgnoreThis,
    }

    #[derive(Debug, PartialEq, Eq)]
    enum ResponseMsg {
        Echo(String),
        Number(i32),
    }

    impl Actor for TestActor {
        type Request = Message;
        type Response = ResponseMsg;

        fn handle(&self, message: Self::Request) -> Option<Self::Response> {
            use Message::*;
            match message {
                Echo(s) => Some(ResponseMsg::Echo(s)),
                AddOne(i) => Some(ResponseMsg::Number(i + 1)),
                IgnoreThis => None,
            }
        }
    }

    #[tokio::test]
    async fn request_echo() {
        let actor_handle = run_actor(TestActor {}, 1);
        let response = actor_handle
            .requestor
            .request(Message::Echo("ping".to_owned()))
            .await
            .unwrap();
        assert_eq!(response.unwrap(), ResponseMsg::Echo("ping".to_owned()));
    }

    #[tokio::test]
    async fn request_add_one() {
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

    mod aborted_actor {
        use std::{
            sync::{Arc, Mutex},
            time::Duration,
        };

        use super::*;

        #[tokio::test]
        async fn returns_error_on_request() {
            const NUMBER: i32 = 6;
            let actor_handle = run_actor(TestActor {}, 1);

            actor_handle.abort();

            let response = actor_handle.requestor.request(Message::IgnoreThis).await;

            assert!(response.is_err());
        }

        #[tokio::test]
        async fn joins_before_a_timeout() {
            const NUMBER: i32 = 6;
            let actor_handle = run_actor(TestActor {}, 1);

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

                let t = timed_out.lock().unwrap();
                assert!(!*t);
            }

            timeout_handle.await.unwrap();
        }
    }
}
