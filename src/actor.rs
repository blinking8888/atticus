use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

use crate::error::Error;

/// Shorthand for the [Actor] Request type
pub type Request<T> = <T as Actor>::Request;
/// Shorthand for the [Actor] Response type
pub type Response<T> = <T as Actor>::Response;
/// Shorthand for the tuple containing the Request and response channel (oneshot)
pub type RequestMessage<T> = (Request<T>, Option<oneshot::Sender<OptionalResponse<T>>>);
/// This is returned by the [Requestor::request] method to indicate the result of the request.
pub type RequestResult<T> = Result<Option<Response<T>>, Error>;
/// Shorthand for an `Option<Response<T>`
pub type OptionalResponse<T> = Option<Response<T>>;

/// A [Requestor] can be passed around by cloning it so multiple requestors can send
/// requests to the [Actor]
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
    /// Send a request message to the [Actor] instance and expecting a response.
    /// The return is a [RequestResult] to indicate if request was successfully sent.
    /// Check the [Error] module for the types of errors that can be thrown.
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

    /// Sends an event to the [Actor] instance and does not wait for a response.
    /// `send_event` returns a JoinHandle to allow the caller the option to wait for the event to be
    /// sent.  Typically, you don't have to...
    /// NOTE: An event is still a [Actor::Request] type.
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

/// This is a handle to the spawned [Actor] instance via [run_actor].  This enables the owner of the
/// `Handle<T>` to abort or wait for the `Actor` spawned instance.
/// It also contains a `requestor` field that can be cloned and passed around to allow multiple
/// clients to send requests to the `Actor`.
pub struct Handle<T>
where
    T: Actor + Sized,
    <T as Actor>::Request: Send,
    <T as Actor>::Response: Send,
{
    /// A clonable `Requestor` for use in sending requests and events to the `Actor`
    pub requestor: Requestor<T>,
    /// The tokio JoinHandle to control the spawned task that runs the `Actor`
    pub handle: JoinHandle<()>,
}

impl<T> Handle<T>
where
    T: Actor + Sized,
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
pub trait Actor: Send + 'static {
    /// The type of the request that the `Actor` could process.
    type Request;

    /// The type of the response that the `Actor` would return
    type Response;

    /// Method to handle the `Request` and expects to return an optional response.
    /// Return None if there really is a reason or data to return the requestor
    fn handle(&mut self, message: Self::Request) -> Option<Self::Response>;
}

/// Spawns an [Actor] instance message handling loop.
/// It accepts an `actor` that implements an [Actor] trait.
/// `buffer` is the number of messages that can be kept in the channel.  Typically, you would only
/// need 1 but if the `Actor` takes a long time to process, a bigger buffer may be needed.
/// This method returns a [Handle] to control and send requests or events to the `Actor` instance.
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
