use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

use crate::error::Error;

type Request<T> = <T as Actor>::Request;
type Response<T> = <T as Actor>::Response;
type RequestMessage<T> = (Request<T>, Option<oneshot::Sender<OptionalResponse<T>>>);
type RequestResult<T> = Result<Option<Response<T>>, Error>;
type OptionalResponse<T> = Option<Response<T>>;

pub struct Requestor<T>(mpsc::Sender<RequestMessage<T>>)
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
    pub async fn request(&self, request: Request<T>) -> RequestResult<T> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<OptionalResponse<T>>();
        self.0
            .send((request, Some(rsp_tx)))
            .await
            .map_err(|e| Error::RequestError(e.to_string()))?;

        rsp_rx
            .await
            .map_err(|e| Error::ResponseError(e.to_string()))
    }

    pub fn send_event(&self, event: Request<T>) -> JoinHandle<Result<(), Error>> {
        let sender = self.0.clone();
        tokio::spawn(async move {
            sender
                .send((event, None))
                .await
                .map_err(|e| Error::EventError(e.to_string()))
        })
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
    fn handle(&mut self, message: Self::Request) -> Option<Self::Response>;
}

pub fn run_actor<T>(mut actor: T, buffer: usize) -> Handle<T>
where
    T: Actor + Sized + Send,
    <T as Actor>::Request: Send,
    <T as Actor>::Response: Send,
{
    let (tx, mut rx) = mpsc::channel::<RequestMessage<T>>(buffer);

    let handle = tokio::spawn(async move {
        while let Some((msg, rsp_tx)) = rx.recv().await {
            let response = actor.handle(msg);
            if let Some(rsp_tx) = rsp_tx {
                let _ = rsp_tx.send(response);
            }
        }
    });

    Handle {
        handle,
        requestor: Requestor(tx),
    }
}
