use crate::types;
use crate::types::Friend;
use crate::types::OriginTune;
use ic_cdk;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::BTreeMap;

type ProfileStore = BTreeMap<String, types::Profile>;
type TuneDB = BTreeMap<String, String>;
type UserTuneStore = BTreeMap<String, Vec<types::Tune>>;
type SessionDB = BTreeMap<u32, types::Session>;

thread_local! {
    pub static PROFILE_STORE: RefCell<ProfileStore> = RefCell::default();
    pub static TUNE_DB: RefCell<TuneDB> = RefCell::default();
    pub static USER_TUNE_STORE: RefCell<UserTuneStore> = RefCell::default();
    pub static SESSION_STORE: RefCell<SessionDB> = RefCell::default();
}

const TURN_DB_INIT: &str = include_str!("./tune_db.json");

pub async fn init() {
    ic_cdk::setup();
    let parsed: Value = serde_json::from_str(TURN_DB_INIT).expect("parse error!");
    TUNE_DB.with(|tune_db| {
        let btree_map: BTreeMap<String, String> = if let Value::Object(obj) = parsed {
            obj.into_iter()
                .map(|(k, v)| (k, v.as_str().unwrap().to_string()))
                .collect()
        } else {
            eprintln!("Expected a JSON object");
            BTreeMap::new() // Return an empty map if not an object
        };
        *tune_db.borrow_mut() = btree_map;
    });
}

pub fn authentication(principal: String) -> Option<types::Profile> {
    PROFILE_STORE.with(|profile_store| {
        if profile_store.borrow().get(&principal).is_some() {
            Some(profile_store.borrow().get(&principal).unwrap().clone())
        } else {
            None
        }
    })
}

pub async fn update_profile(
    principal: String,
    username: String,
    avatar: Vec<u8>,
    pob: String,
    instruments: String,
) -> types::Profile {
    PROFILE_STORE.with(|profile_store| {
        if profile_store.borrow().get(&principal).is_some() {
            let mut new_profile = profile_store.borrow().get(&principal).unwrap().clone();
            new_profile.username = username;
            new_profile.avatar = avatar;
            new_profile.pob = pob;
            new_profile.instruments = instruments;
            profile_store
                .borrow_mut()
                .insert(principal, new_profile.clone());
            new_profile
        } else {
            let new_profile = types::Profile {
                principal: principal.clone(),
                username,
                avatar,
                pob,
                instruments,
                friends: vec![],
                incoming_fr: vec![],
                outcoming_fr: vec![],
            };
            profile_store
                .borrow_mut()
                .insert(principal, new_profile.clone());
            new_profile
        }
    })
}

pub fn get_original_tune_list(page_number: i32) -> (Vec<String>, i32) {
    TUNE_DB.with(|tune_db| {
        let res: Vec<String> = tune_db
            .borrow()
            .iter()
            .skip(page_number as usize * 15)
            .enumerate()
            .filter(|(index, _)| index.clone() < 15)
            .map(|(_, (_, data))| data.clone())
            .collect();
        (res, tune_db.borrow().len() as i32)
    })
}

pub fn get_original_tune(title: String) -> String {
    TUNE_DB.with(|tune_db| {
        if tune_db.borrow().get(&title).is_some() {
            tune_db.borrow().get(&title).unwrap().clone()
        } else {
            "not found".to_string()
        }
    })
}

pub fn get_user_tune_list(principal: String, page_number: i32) -> (Vec<types::UserTune>, i32) {
    USER_TUNE_STORE.with(|user_tune_store| {
        if user_tune_store.borrow().get(&principal).is_some() {
            let user_tunes = user_tune_store.borrow().get(&principal).unwrap().clone();
            let res = user_tunes
                .iter()
                .skip(page_number as usize * 15)
                .enumerate()
                .filter(|(index, _)| index.clone() < 15)
                .map(|(_, tune)| {
                    let user_tune = types::UserTune {
                        id: tune.id.clone(),
                        title: tune.title.clone(),
                        thumbnail: tune.thumbnail.clone(),
                    };
                    user_tune
                })
                .collect();
            (res, user_tunes.len() as i32)
        } else {
            (vec![], 0)
        }
    })
}

pub fn get_user_tune(principal: String, title: String) -> String {
    USER_TUNE_STORE.with(|user_tune_store| {
        let user_tunebook = user_tune_store.borrow().get(&principal).unwrap().clone();
        let tune = user_tunebook
            .iter()
            .find(|tune| tune.title == title)
            .unwrap();
        let tune_data = tune.clone().tune_data;
        if tune_data.is_some() {
            tune_data.unwrap()
        } else {
            TUNE_DB.with(|tune_db| tune_db.borrow().get(&title).unwrap().clone())
        }
    })
}

pub async fn add_tune(
    principal: String,
    title: String,
    tune_data: String,
    origin: bool,
    thumbnail: Vec<u8>,
) -> bool {
    USER_TUNE_STORE.with(|user_tune_store| {
        let mut user_tunebook: Vec<types::Tune> = vec![];
        if user_tune_store.borrow().get(&principal).is_some() {
            user_tunebook = user_tune_store.borrow().get(&principal).unwrap().clone();

            let same_tunes: Vec<&types::Tune> = user_tunebook
                .iter()
                .filter(|&tune| tune.clone().title == title)
                .collect();

            if same_tunes.len() > 0 {
                return false;
            }
        }
        let new_tune = types::Tune {
            id: ic_cdk::api::time() as u32,
            origin,
            title,
            tune_data: Some(tune_data),
            timestamp: ic_cdk::api::time(),
            thumbnail,
        };
        user_tunebook.push(new_tune);
        user_tune_store
            .borrow_mut()
            .insert(principal, user_tunebook.clone());
        true
    })
}

pub async fn update_tune(
    tune_id: u32,
    principal: String,
    title: String,
    tune_data: String,
    origin: bool,
    thumbnail: Vec<u8>,
) -> bool {
    USER_TUNE_STORE.with(|user_tune_store| {
        if user_tune_store.borrow().get(&principal).is_some() {
            let user_tunebook = user_tune_store.borrow().get(&principal).unwrap().clone();
            let mut new_tunebook: Vec<types::Tune> = user_tunebook
                .iter()
                .filter(|tune| tune.id != tune_id)
                .map(|tune| tune.clone())
                .collect();
            let new_tune = types::Tune{
                id: tune_id,
                title,
                tune_data: Some(tune_data),
                origin,
                timestamp: ic_cdk::api::time(),
                thumbnail
            };
            new_tunebook.push(new_tune);
            user_tune_store.borrow_mut().insert(principal, new_tunebook);
            true
        } else {
            false
        }
    })
}

pub fn get_friends(principal: String) -> Vec<types::Friend> {
    PROFILE_STORE.with(|profile_store| {
        let binding = profile_store.borrow();
        if binding.get(&principal).is_some() {
            let friend_principals = profile_store
                .borrow()
                .get(&principal)
                .unwrap()
                .friends
                .clone();
            let result: Vec<types::Friend> = friend_principals
                .iter()
                .map(|friend_principal| {
                    let friend_profile = binding.get(friend_principal).unwrap();
                    let friend = types::Friend {
                        principal: friend_principal.clone(),
                        avatar: friend_profile.avatar.clone(),
                        username: friend_profile.username.clone(),
                    };
                    friend
                })
                .collect();
            result
        } else {
            vec![]
        }
    })
}

pub async fn send_friend_request(sender: String, receiver: String) -> Option<types::Friend> {
    PROFILE_STORE.with(|profile_store| {
        let mut binding = profile_store.borrow_mut();
        if binding.get(&sender).is_some() && binding.get(&receiver).is_some() {
            let mut sender_profile = binding.get(&sender).unwrap().clone();
            let mut receiver_profile = binding.get(&receiver).unwrap().clone();
            let incoming_request = types::Friend {
                principal: sender.clone(),
                username: sender_profile.username.clone(),
                avatar: sender_profile.avatar.clone(),
            };
            let outcoming_request = types::Friend {
                principal: receiver.clone(),
                username: receiver_profile.username.clone(),
                avatar: receiver_profile.avatar.clone(),
            };
            sender_profile.outcoming_fr.push(outcoming_request.clone());
            receiver_profile.incoming_fr.push(incoming_request);
            binding.insert(sender, sender_profile);
            binding.insert(receiver, receiver_profile);
            Some(outcoming_request)
        } else {
            None
        }
    })
}

pub async fn accept_friend_request(sender: String, receiver: String) -> bool {
    PROFILE_STORE.with(|profile_store| {
        let mut binding = profile_store.borrow_mut();
        if binding.get(&sender).is_some() && binding.get(&receiver).is_some() {
            let mut sender_profile = binding.get(&sender).unwrap().clone();
            let mut receiver_profile = binding.get(&receiver).unwrap().clone();
            let in_position = sender_profile
                .incoming_fr
                .iter()
                .position(|ifr| ifr.principal == receiver.clone());
            if in_position.is_some() {
                sender_profile.incoming_fr.remove(in_position.unwrap());
            }
            let out_position = receiver_profile
                .outcoming_fr
                .iter()
                .position(|ofr| ofr.principal == sender.clone());
            if out_position.is_some() {
                receiver_profile.outcoming_fr.remove(out_position.unwrap());
            }

            sender_profile.friends.push(receiver.clone());
            receiver_profile.friends.push(sender.clone());
            binding.insert(sender, sender_profile);
            binding.insert(receiver, receiver_profile);
            true
        } else {
            false
        }
    })
}

pub fn filter_tunes(
    sub_title: &str,
    rithm: &str,
    key: &str,
    page_num: i32,
) -> (Vec<types::OriginTune>, i32) {
    TUNE_DB.with(|tune_db| {
        let binding = tune_db.borrow();
        let res: Vec<OriginTune> = binding
            .iter()
            .filter(|(title, tune_data)| {
                title.contains(sub_title)
                    && (rithm == "all" || tune_data.contains(format!("R: {}", rithm).as_str()))
                    && (key == "all" || tune_data.contains(format!("K: {}", key).as_str()))
            })
            .map(|(title, tune_data)| {
                let tune = types::OriginTune {
                    title: title.clone(),
                    tune_data: tune_data.clone(),
                };
                tune
            })
            .collect();
        let result: Vec<OriginTune> = res
            .iter()
            .skip(page_num as usize * 15 as usize)
            .enumerate()
            .filter(|(index, _)| index.clone() < 15 as usize)
            .map(|(_, tune)| tune.clone())
            .collect();
        (result, res.len() as i32)
    })
}

pub fn browse_people(filter: String, page_num: i32) -> (Vec<types::Friend>, i32) {
    PROFILE_STORE.with(|profile_store| {
        let res: Vec<Friend> = profile_store
            .borrow()
            .iter()
            .filter(|(_, profile)| profile.username.contains(filter.as_str()))
            .map(|(principal, profile)| {
                let user: Friend = types::Friend {
                    principal: principal.clone(),
                    avatar: profile.avatar.clone(),
                    username: profile.username.clone(),
                };
                user
            })
            .collect();
        let result: Vec<Friend> = res
            .iter()
            .skip(page_num as usize * 15 as usize)
            .enumerate()
            .filter(|(index, _)| index.clone() < 15 as usize)
            .map(|(_, user)| user.clone())
            .collect();
        (result, res.len() as i32)
    })
}

pub fn get_new_tunes_from_friends(principal: String) -> Vec<types::Tune> {
    let mut result: Vec<types::Tune> = vec![];
    let friends = PROFILE_STORE.with(|profile_store| {
        let binding = profile_store.borrow();
        if binding.get(&principal).is_some() {
            binding.get(&principal).unwrap().friends.clone()
        } else {
            vec![]
        }
    });
    USER_TUNE_STORE.with(|user_tune_store| {
        let binding = user_tune_store.borrow();
        friends.iter().for_each(|friend| {
            let frined_tunes = binding.get(friend).unwrap_or(&vec![]).clone();
            frined_tunes
                .iter()
                .filter(|tune| ic_cdk::api::time() - tune.timestamp < 604800000000000)
                .for_each(|tune| result.push(tune.clone()));
        });
    });
    result
}

pub fn get_sessions(sub_name: &str, page_num: i32) -> (Vec<types::Session>, i32) {
    SESSION_STORE.with(|session_store| {
        let res: Vec<types::Session> = session_store
            .borrow()
            .iter()
            .filter(|(_, session)| session.name.contains(sub_name))
            .map(|(_, session)| session.clone())
            .collect();

        let result: Vec<types::Session> = res
            .iter()
            .skip(page_num as usize * 15 as usize)
            .enumerate()
            .filter(|(index, _)| index.clone() < 15 as usize)
            .map(|(_, session)| session.clone())
            .collect();

        (result, res.len() as i32)
    })
}

pub fn add_session(principal: String, name: String, location: String, daytime: String, contact: String, comment: String) -> bool {
    SESSION_STORE.with(|session_store| {
        let new_session = types::Session {
            id: ic_cdk::api::time() as u32,
            principal,
            name,
            location,
            daytime,
            contact,
            comment
        };

        session_store.borrow_mut().insert(new_session.id.clone(), new_session);
        true
    })
}