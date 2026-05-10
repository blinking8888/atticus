use std::fmt;
use std::ops;

use async_trait::async_trait;

use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

use crate::error::Error;

/// This is returned by the [`Addr::request`] method to indicate the result of the request.
pub type RequestResult<Rsp> = Result<Rsp, Error>;

type OptionalResponder<Rsp> = Option<oneshot::Sender<Rsp>>;

/// An [Addr] can be passed around by cloning it so multiple requestors can send
/// requests to the [Actor]
#[repr(transparent)]
#[derive(Debug)]
pub struct Addr<Req, Rsp>(mpsc::Sender<(Req, OptionalResponder<Rsp>)>);

impl<Req, Rsp> Clone for Addr<Req, Rsp>
where
    Req: Send,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Req, Rsp> ops::Deref for Addr<Req, Rsp> {
    type Target = mpsc::Sender<(Req, OptionalResponder<Rsp>)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Req, Rsp> Addr<Req, Rsp>
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
        let (rsp_tx, rsp_rx) = oneshot::channel::<Rsp>();
        self.0
            .send((request, Some(rsp_tx)))
            .await
            .map_err(|_e| Error::RequestError)?;

        rsp_rx.await.map_err(|_e| Error::ResponseError)
    }

    /// Sends an event to the [`Actor`] instance and does not wait for a response.
    /// NOTE: An event is still a [`Actor::Request`] type.
    ///
    /// # Errors
    /// - `Error::EventError`: Problem with sending the event
    pub async fn event(&self, event: Req) -> Result<(), Error> {
        self.0
            .send((event, None))
            .await
            .map_err(|_e| Error::EventError)
    }
}

/// This is a handle to the spawned [Actor] instance via [`actor::run`].
///
/// This enables the owner of the `Handle<T>` to abort or wait for the `Actor` spawned instance.
/// It also contains an `addr` field that can be cloned and passed around to allow multiple
/// clients to send requests to the `Actor`.
pub struct Handle<T>
where
    T: Actor + Send,
    <T as Actor>::Request: Send,
    <T as Actor>::Response: Send,
{
    /// A clonable `Addr` for use in sending requests and events to the `Actor`
    pub addr: Addr<<T as Actor>::Request, <T as Actor>::Response>,
    /// The tokio `JoinHandle` to control the spawned task that runs the `Actor`
    pub handle: JoinHandle<()>,
}

impl<T> fmt::Debug for Handle<T>
where
    T: Actor + Send + fmt::Debug,
    <T as Actor>::Request: Send + fmt::Debug,
    <T as Actor>::Response: Send + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Handle")
            .field("addr", &self.addr)
            .field("handle", &self.handle)
            .finish()
    }
}

impl<T> ops::Deref for Handle<T>
where
    T: Actor + Send,
    <T as Actor>::Request: Send,
    <T as Actor>::Response: Send,
{
    type Target = Addr<<T as Actor>::Request, <T as Actor>::Response>;

    fn deref(&self) -> &Self::Target {
        &self.addr
    }
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
    type Request: Send;

    /// The type of the response that the `Actor` would return
    type Response: Send;

    /// Method to handle the `Request` and returns a response.
    async fn handle(&mut self, message: Self::Request) -> Self::Response;

    /// Method to handle events.  An event is a request that does not expect a response.
    /// The default implementation calls `handle` and ignores the response.
    async fn handle_event(&mut self, message: Self::Request) {
        let _ = self.handle(message).await;
    }
}

/// Spawns an [Actor] instance message handling loop.
///
/// It accepts an `actor` that implements an [Actor] trait.
/// `buffer` is the number of messages that can be kept in the channel.  Typically, you would only
/// need 1 but if the `Actor` takes a long time to process, a bigger buffer may be needed.
/// This method returns a [Handle] to control and send requests or events to the `Actor` instance.
pub fn run<T>(mut actor: T, buffer: usize) -> Handle<T>
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
            if let Some(rsp_tx) = rsp_tx {
                let response = actor.handle(msg).await;
                let _ = rsp_tx.send(response);
            } else {
                actor.handle_event(msg).await;
            }
        }
    });

    Handle {
        handle,
        addr: Addr(tx),
    }
}
