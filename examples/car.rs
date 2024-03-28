use async_trait::async_trait;
use atticus::{actor, Actor};

pub enum CarRequest {
    Park,
    Drive,
    Brake,
    Reverse,
}

pub type CarResponse = String;

#[async_trait]
pub trait Car {
    async fn park(&self) -> CarResponse;
    async fn drive(&self) -> CarResponse;
    async fn brake(&self) -> CarResponse;
    async fn reverse(&self) -> CarResponse;
}

pub struct Sedan;

#[async_trait]
impl Car for Sedan {
    async fn park(&self) -> CarResponse {
        String::from("Sedan: park")
    }
    async fn drive(&self) -> CarResponse {
        String::from("Sedan: drive")
    }
    async fn brake(&self) -> CarResponse {
        String::from("Sedan: brake")
    }
    async fn reverse(&self) -> CarResponse {
        String::from("Sedan: reverse")
    }
}

#[async_trait]
impl Actor for Sedan {
    type Request = CarRequest;
    type Response = String;

    async fn handle(&mut self, message: Self::Request) -> Option<Self::Response> {
        match message {
            CarRequest::Park => Some(self.park().await),
            CarRequest::Drive => Some(self.drive().await),
            CarRequest::Brake => Some(self.brake().await),
            CarRequest::Reverse => Some(self.reverse().await),
        }
    }
}

impl Sedan {
    pub fn new() -> Self {
        Self
    }
}

pub struct Suv;

#[async_trait]
impl Car for Suv {
    async fn park(&self) -> CarResponse {
        String::from("SUV: park")
    }
    async fn drive(&self) -> CarResponse {
        String::from("SUV: drive")
    }
    async fn brake(&self) -> CarResponse {
        String::from("SUV: brake")
    }
    async fn reverse(&self) -> CarResponse {
        String::from("SUV: reverse")
    }
}

#[async_trait]
impl Actor for Suv {
    type Request = CarRequest;
    type Response = String;

    async fn handle(&mut self, message: Self::Request) -> Option<Self::Response> {
        match message {
            CarRequest::Park => Some(self.park().await),
            CarRequest::Drive => Some(self.drive().await),
            CarRequest::Brake => Some(self.brake().await),
            CarRequest::Reverse => Some(self.reverse().await),
        }
    }
}

impl Suv {
    pub fn new() -> Self {
        Self
    }
}

async fn run_car<T>(car: T)
where
    T: Car + Actor,
    T: Actor<Request = CarRequest>,
    T: Actor<Response = String>,
    <T as Actor>::Request: Send + Sync,
    <T as Actor>::Response: Send + Sync,
{
    let actor_handle = actor::run(car, 1);
    let requestor = actor_handle.requestor;

    let response = requestor.request(CarRequest::Park).await.unwrap();
    println!("{}", response.unwrap());

    let response = requestor.request(CarRequest::Drive).await.unwrap();
    println!("{}", response.unwrap());

    let response = requestor.request(CarRequest::Brake).await.unwrap();
    println!("{}", response.unwrap());

    let response = requestor.request(CarRequest::Reverse).await.unwrap();
    println!("{}", response.unwrap());
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    run_car(Sedan::new()).await;
    run_car(Suv::new()).await;
}
