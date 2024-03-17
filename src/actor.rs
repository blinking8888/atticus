use async_trait::async_trait;

use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

use crate::error::Error;

/// This is returned by the [`Requestor::request`] method to indicate the result of the request.
pub type RequestResult<Rsp> = Result<Option<Rsp>, Error>;

type OptionalResponder<Rsp> = Option<oneshot::Sender<Option<Rsp>>>;

/// A [Requestor] can be passed around by cloning it so multiple requestors can send
/// requests to the [Actor]
#[repr(transparent)]
pub struct Requestor<Req, Rsp>(mpsc::Sender<(Req, OptionalResponder<Rsp>)>);

impl<Req, Rsp> Clone for Requestor<Req, Rsp>
where
    Req: Send,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Req, Rsp> Requestor<Req, Rsp>
where
    Req: Send + 'static,
    Rsp: Send + 'static,
{
    /// # Description
    /// Send a request message to the [Actor] instance and expecting a response.
    /// The return is a [`RequestResult`] to indicate if request was successfully sent.
    /// Check the [`Error`] module for the types of errors that can be thrown.
    ///
    /// # Errors
    /// - `Error::RequestError`: Problem with sending the request
    /// - `Error::ResponseError`: Problem with receiving the response
    pub async fn request(&self, request: Req) -> RequestResult<Rsp> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Option<Rsp>>();
        self.0
            .send((request, Some(rsp_tx)))
            .await
            .map_err(|_e| Error::RequestError)?;

        rsp_rx.await.map_err(|_e| Error::ResponseError)
    }

    /// Sends an event to the [`Actor`] instance and does not wait for a response.
    /// `send_event` returns a `JoinHandle` to allow the caller the option to wait for the event to be
    /// sent.  Typically, you don't have to...
    /// NOTE: An event is still a [`Actor::Request`] type.
    pub fn send_event(&self, event: Req) -> JoinHandle<Result<(), Error>> {
        let sender = self.0.clone();
        tokio::spawn(async move {
            sender
                .send((event, None))
                .await
                .map_err(|_e| Error::EventError)
        })
    }
}

/// This is a handle to the spawned [Actor] instance via [`run_actor`].  This enables the owner of the
/// `Handle<T>` to abort or wait for the `Actor` spawned instance.
/// It also contains a `requestor` field that can be cloned and passed around to allow multiple
/// clients to send requests to the `Actor`.
pub struct Handle<T>
where
    T: Actor + Send,
    <T as Actor>::Request: Send,
    <T as Actor>::Response: Send,
{
    /// A clonable `Requestor` for use in sending requests and events to the `Actor`
    pub requestor: Requestor<<T as Actor>::Request, <T as Actor>::Response>,
    /// The tokio JoinHandle to control the spawned task that runs the `Actor`
    pub handle: JoinHandle<()>,
}

impl<T> Handle<T>
where
    T: Actor + Send,
    <T as Actor>::Request: Send,
    <T as Actor>::Response: Send,
{
    /// Method to abort the actor message handling task
    pub fn abort(&self) {
        self.handle.abort();
    }

    /// Waits for the actor message handling task to complete.
    pub async fn join(self) {
        let _ = self.handle.await;
    }
}

/// This is the trait to create an `Actor` instance.
#[async_trait]
pub trait Actor: Send + 'static {
    /// The type of the request that the `Actor` could process.
    type Request;

    /// The type of the response that the `Actor` would return
    type Response;

    /// Method to handle the `Request` and expects to return an optional response.
    /// Return None if there really is a reason or data to return the requestor
    async fn handle(&mut self, message: Self::Request) -> Option<Self::Response>;
}

/// Spawns an [Actor] instance message handling loop.
/// It accepts an `actor` that implements an [Actor] trait.
/// `buffer` is the number of messages that can be kept in the channel.  Typically, you would only
/// need 1 but if the `Actor` takes a long time to process, a bigger buffer may be needed.
/// This method returns a [Handle] to control and send requests or events to the `Actor` instance.
#[allow(clippy::module_name_repetitions)]
pub fn run_actor<T>(mut actor: T, buffer: usize) -> Handle<T>
where
    T: Actor + Send,
    <T as Actor>::Request: Send,
    <T as Actor>::Response: Send,
{
    type Request<T> = <T as Actor>::Request;
    type Response<T> = <T as Actor>::Response;
    type RequestMessage<T> = (Request<T>, OptionalResponder<Response<T>>);

    let (tx, mut rx) = mpsc::channel::<RequestMessage<T>>(buffer);

    let handle = tokio::spawn(async move {
        while let Some((msg, rsp_tx)) = rx.recv().await {
            let response = actor.handle(msg).await;
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
