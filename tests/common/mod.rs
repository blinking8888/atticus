use simple_actor::Actor;

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
}

impl Actor for TestActor {
    type Request = Message;
    type Response = ResponseMsg;

    fn handle(&mut self, message: Self::Request) -> Option<Self::Response> {
        use Message::*;

        println!("handling message");

        let response = match &message {
            Echo(s) => Some(ResponseMsg::Echo(s.clone())),
            AddOne(i) => Some(ResponseMsg::Number(i + 1)),
            IgnoreThis => None,
            GetLastRequest => Some(ResponseMsg::LastRequest(self.last_message.clone())),
        };

        self.last_message = Some(message);

        response
    }
}
