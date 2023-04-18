use simple_actor::Actor;

pub struct TestActor;

#[allow(dead_code)]
pub enum Message {
    Echo(String),
    AddOne(i32),
    IgnoreThis,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResponseMsg {
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
