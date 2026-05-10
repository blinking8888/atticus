use async_trait::async_trait;
use atticus::Actor;

#[derive(Default)]
pub struct TestActor {
    last_message: Option<Message>,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Message {
    Echo(String),
    AddOne(i32),
    IgnoreThis,
    GetLastRequest,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ResponseMsg {
    Echo(String),
    Number(i32),
    LastRequest(Option<Message>),
    None,
}

#[async_trait]
impl Actor for TestActor {
    type Request = Message;
    type Response = ResponseMsg;

    async fn handle(&mut self, message: Self::Request) -> Self::Response {
        use Message::*;

        println!("handling message");

        let response = match &message {
            Echo(s) => ResponseMsg::Echo(s.clone()),
            AddOne(i) => ResponseMsg::Number(i + 1),
            IgnoreThis => ResponseMsg::None,
            GetLastRequest => ResponseMsg::LastRequest(self.last_message.clone()),
        };

        self.last_message = Some(message);

        response
    }
}
