use tokio::sync::{mpsc, oneshot};

use crate::error::Error;

pub struct Handle<T>(
    mpsc::Sender<(
        <T as Actor>::Request,
        oneshot::Sender<Option<<T as Actor>::Response>>,
    )>,
)
where
    T: Actor + Sized,
    <T as Actor>::Request: Send;

impl<T> Handle<T>
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
    tokio::spawn(async move {
        while let Some((msg, rsp_tx)) = rx.recv().await {
            let response = actor.handle(msg);
            let _ = rsp_tx.send(response);
        }
    });

    Handle(tx)
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
            .request(Message::Echo("ping".to_owned()))
            .await
            .unwrap();
        assert_eq!(response.unwrap(), ResponseMsg::Echo("ping".to_owned()));
    }

    #[tokio::test]
    async fn request_add_one() {
        const NUMBER: i32 = 6;
        let actor_handle = run_actor(TestActor {}, 1);
        let response = actor_handle.request(Message::AddOne(NUMBER)).await.unwrap();
        assert_eq!(response.unwrap(), ResponseMsg::Number(NUMBER + 1));
    }

    #[tokio::test]
    async fn respond_with_none() {
        const NUMBER: i32 = 6;
        let actor_handle = run_actor(TestActor {}, 1);
        let response = actor_handle.request(Message::IgnoreThis).await.unwrap();
        assert_eq!(response, None);
    }
}
