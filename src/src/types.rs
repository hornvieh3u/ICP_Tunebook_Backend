use candid::{CandidType, Deserialize};

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct Profile {
    pub principal: String,
    pub username: String,
    pub avatar: Vec<u8>,
    pub pob: String,
    pub instruments: String,
    pub friends: Vec<String>,
    pub incoming_fr: Vec<Friend>,
    pub outcoming_fr: Vec<Friend>
}

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct Tune {
    pub origin: bool,
    pub title: String,
    pub tune_data: String,
    pub timestamp: u64,
    pub principals: Vec<String>,
}

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct Session {
    pub id: u32,
    pub principal: String,
    pub name: String,
    pub location: String,
    pub daytime: String,
    pub contact: String,
    pub comment: String,
}

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct Friend {
    pub principal: String,
    pub avatar: Vec<u8>,
    pub username: String
}

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct Tuneinfo {
    pub title: String,
    pub tune_data: String
}