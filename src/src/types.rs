use candid::{ CandidType, Deserialize};

#[derive(CandidType, Clone, Debug)]
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
pub struct Friend {
    pub principal: String,
    pub avatar: Vec<u8>,
    pub username: String
}

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct Tune {
    pub id: u32,
    pub origin: bool,
    pub title: String,
    pub tune_data: Option<String>,
    pub timestamp: u64,
    pub thumbnail: Vec<u8>
}

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct UserTune {
    pub id: u32,
    pub title: String,
    pub thumbnail: Vec<u8>
}

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct OriginTune {
    pub title: String,
    pub tune_data: String
}
