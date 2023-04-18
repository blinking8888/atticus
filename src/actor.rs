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
