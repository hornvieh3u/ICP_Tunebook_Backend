use ic_cdk;
use crate::types;
use candid::{Decode, Encode};
use serde_json::Value;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::BTreeMap;
use regex::Regex;

use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};

type Memory = VirtualMemory<DefaultMemoryImpl>;

type ProfileStore = StableBTreeMap<String, types::Profile, Memory>;
type TuneDB = StableBTreeMap<String, types::Tune, Memory>;
type SessionDB = StableBTreeMap<u32, types::Session, Memory>;

impl Storable for types::Profile {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 2000000, // Replace with the actual max size
        is_fixed_size: false,
    };
}

impl Storable for types::Tune {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 2000000, // Replace with the actual max size
        is_fixed_size: false,
    };
}

impl Storable for types::Session {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    
    const BOUND: Bound = Bound::Bounded {
        max_size: 2000000, // Replace with the actual max size
        is_fixed_size: false,
    };
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    pub static PROFILE_STORE: RefCell<ProfileStore> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
    ));

    pub static TUNE_STORE: RefCell<TuneDB> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    pub static SESSION_STORE: RefCell<SessionDB> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));
}

const TUNE_DB_INIT: &str = include_str!("./tune_db.json");

pub async fn init() {
    ic_cdk::setup();
    let parsed: Value = serde_json::from_str(TUNE_DB_INIT).expect("parse error!");
    TUNE_STORE.with(|tune_store| {
        if tune_store.borrow().len() == 0 {
            let btree_map = if let Value::Object(obj) = parsed {
                obj.into_iter()
                    .map(|(k, v)| (k, v.as_str().unwrap().to_string()))
                    .collect()
            } else {
                eprintln!("Expected a JSON object");
                BTreeMap::new() // Return an empty map if not an object
            };

            for (key, value) in btree_map.iter() {
                let new_tune = types::Tune {
                    origin: true,
                    title: key.clone(),
                    tune_data: value.clone(),
                    timestamp: ic_cdk::api::time(),
                    principals: vec![]
                };
                tune_store.borrow_mut().insert(key.clone(), new_tune);
            } 
        }
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
    TUNE_STORE.with(|tune_store| {
        let res: Vec<String> = tune_store
            .borrow()
            .iter()
            .skip(page_number as usize * 15)
            .enumerate()
            .filter(|(index, _)| index.clone() < 15)
            .map(|(_, (_, data))| data.title.clone())
            .collect();
        (res, tune_store.borrow().len() as i32)
    })
}

pub fn get_original_tune(title: String) -> String {
    TUNE_STORE.with(|tune_store| {
        if tune_store.borrow().get(&title).is_some() {
            // tune_store.borrow().get(&title).unwrap().tune_data.clone()
            let selected_tune = tune_store.borrow_mut().get(&title).unwrap();
            selected_tune.tune_data.clone()
        } else {
            String::from("not found")
        }
    })
}

pub fn get_user_tune_list(principal: String, page_number: i32) -> (Vec<types::Tuneinfo>, i32) {
    TUNE_STORE.with(|tune_store| {
        let user_tunes: Vec<types::Tuneinfo> = tune_store
            .borrow()
            .iter()
            .filter(|(_, tune_info)| tune_info.principals.contains(&principal))
            .map(|(_, tune_info)| {
                let user_tune = types::Tuneinfo {
                    title: tune_info.title.clone(),
                    tune_data: tune_info.tune_data.clone()
                };
                user_tune
            })
            .collect();

            if page_number == -1 {
                return (user_tunes.clone(), user_tunes.len() as i32);
            }

            let res = user_tunes
                .iter()
                .skip(page_number as usize * 15)
                .enumerate()
                .filter(|(index, _)| index.clone() < 15)
                .map(|(_, tune_info)| tune_info.clone())
                .collect();

            return (res, user_tunes.len() as i32);
    })
}

pub fn get_user_tune(principal: String, title: String) -> String {
    TUNE_STORE.with(|tune_store| {
        let user_tune = tune_store
            .borrow()
            .get(&title)
            .unwrap()
            .clone();
        
        if user_tune.principals.contains(&principal) {
            user_tune.tune_data.clone()
        } else {
            String::new()
        }
    })
}

pub async fn add_tune(
    principal: String,
    title: String,
    tune_data: String,
    origin: bool,
) -> bool {
    TUNE_STORE.with(|tune_store| {
        let mut principals: Vec<String> = vec![];
        if tune_store.borrow().get(&title).is_some() {
            let prev_tune = tune_store.borrow().get(&title).unwrap().clone();
            if prev_tune.principals.contains(&principal) {
                return false;
            }

            principals = prev_tune.principals;
        }

        principals.push(principal);

        let new_tune = types::Tune {
            origin,
            title,
            tune_data,
            timestamp: ic_cdk::api::time(),
            principals
        };
        tune_store.borrow_mut().insert(new_tune.title.clone(), new_tune);
        true
    })
}

pub async fn update_tune(
    principal: String,
    title: String,
    tune_data: String,
    origin: bool,
) -> bool {
    TUNE_STORE.with(|tune_store| {
        if tune_store.borrow().get(&title).is_none() {
            return false;
        }
        let prev_tune = tune_store.borrow().get(&title).unwrap().clone();
        if !prev_tune.principals.contains(&principal) {
            return false;
        }

        let updated_tune = types::Tune {
            origin,
            title,
            tune_data,
            timestamp: ic_cdk::api::time(),
            principals: prev_tune.principals
        };
        tune_store.borrow_mut().insert(updated_tune.title.clone(), updated_tune);
        true
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

            let outcoming_principals: Vec<String> = sender_profile.outcoming_fr
                .iter()
                .map(|friend| friend.principal.clone())
                .collect();

            let incoming_principals: Vec<String> = sender_profile.incoming_fr
                .iter()
                .map(|friend| friend.principal.clone())
                .collect();
            
            if sender_profile.friends.contains(&receiver) || outcoming_principals.contains(&receiver) || incoming_principals.contains(&receiver) {
                return None;
            }

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
) -> (Vec<types::Tuneinfo>, i32) {
    TUNE_STORE.with(|tune_store| {
        let binding = tune_store.borrow();
        let res: Vec<types::Tuneinfo> = binding
            .iter()
            .filter(|(title, tune_info)| {
                let regex_rythm = Regex::new(&format!(r"R:\s*{}", rithm)).unwrap();
                let regex_key = Regex::new(&format!(r"K:\s*{}", key)).unwrap();
                title.to_lowercase().contains(&sub_title.to_lowercase())
                    && (rithm == "all" || regex_rythm.is_match(&tune_info.tune_data.clone()))
                    && (key == "all" || regex_key.is_match(&tune_info.tune_data.clone()))
            })
            .map(|(title, tune_info)| {
                let tune = types::Tuneinfo {
                    title: title.clone(),
                    tune_data: tune_info.tune_data.clone(),
                };
                tune
            })
            .collect();
        let result: Vec<types::Tuneinfo> = res
            .iter()
            .skip(page_num as usize * 15 as usize)
            .enumerate()
            .filter(|(index, _)| index.clone() < 15 as usize)
            .map(|(_, tune)| tune.clone())
            .collect();
        (result, res.len() as i32)
    })
}

pub fn browse_people(my_principal: String, filter: String, page_num: i32) -> (Vec<types::Friend>, i32) {
    PROFILE_STORE.with(|profile_store| {
        let my_profile = profile_store.borrow().get(&my_principal).unwrap().clone();
        let outcoming_principals: Vec<String> = my_profile.outcoming_fr
            .iter()
            .map(|friend| friend.principal.clone())
            .collect();

        let incoming_principals: Vec<String> = my_profile.incoming_fr
            .iter()
            .map(|friend| friend.principal.clone())
            .collect();

        let res: Vec<types::Friend> = profile_store
            .borrow()
            .iter()
            .filter(|(_, profile)| 
                profile.username.contains(filter.as_str()) &&
                profile.principal != my_profile.principal &&
                !my_profile.friends.contains(&profile.principal) &&
                !outcoming_principals.contains(&profile.principal) &&
                !incoming_principals.contains(&profile.principal)
            )
            .map(|(principal, profile)| {
                let user = types::Friend {
                    principal: principal.clone(),
                    avatar: profile.avatar.clone(),
                    username: profile.username.clone(),
                };
                user
            })
            .collect();
        let result: Vec<types::Friend> = res
            .iter()
            .skip(page_num as usize * 15 as usize)
            .enumerate()
            .filter(|(index, _)| index.clone() < 15 as usize)
            .map(|(_, user)| user.clone())
            .collect();
        (result, res.len() as i32)
    })
}

pub fn get_new_tunes_from_friends(_principal: String) -> Vec<types::Tune> {
    // let friends = PROFILE_STORE.with(|profile_store| {
    //     let binding = profile_store.borrow();
    //     if binding.get(&principal).is_some() {
    //         binding.get(&principal).unwrap().friends.clone()
    //     } else {
    //         vec![]
    //     }
    // });
    TUNE_STORE.with(|tune_store| {
        tune_store
            .borrow()
            .iter()
            .filter(|(_, tune_info)| ic_cdk::api::time() - tune_info.timestamp < 604800000000000)
            .map(|(_, tune)| tune.clone())
            .collect()
    })
    // USER_TUNE_STORE.with(|user_tune_store| {
    //     let binding = user_tune_store.borrow();
    //     friends.iter().for_each(|friend| {
    //         let frined_tunes = binding.get(friend).unwrap_or(&vec![]).clone();
    //         frined_tunes
    //             .iter()
    //             .filter(|tune| ic_cdk::api::time() - tune.timestamp < 604800000000000)
    //             .for_each(|tune| result.push(tune.clone()));
    //     });
    // });
}

pub fn get_sessions(sub_name: &str, page_num: i32) -> (Vec<types::Session>, i32) {
    SESSION_STORE.with(|session_store| {
        let res: Vec<types::Session> = session_store
            .borrow()
            .iter()
            .filter(|(_, session)| session.name.contains(sub_name) || session.location.contains(sub_name))
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

pub fn update_session(id: u32, principal: String, name: String, location: String, daytime: String, contact: String, comment: String) -> bool {
    SESSION_STORE.with(|session_store| {
        if session_store.borrow().get(&id).is_none() {
            return false;
        }

        let mut updated_session = session_store.borrow().get(&id).unwrap().clone();
        if updated_session.principal != principal {
            return false;
        }

        updated_session.name = name;
        updated_session.location = location;
        updated_session.daytime = daytime;
        updated_session.contact = contact;
        updated_session.comment = comment;
        session_store.borrow_mut().insert(id, updated_session);
        true
    })
}