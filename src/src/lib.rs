mod utils;
mod types;

#[ic_cdk::init]
fn init(time: u64) {
    ic_cdk_timers::set_timer(std::time::Duration::from_secs(time), || {
        ic_cdk::spawn(update_data())
    });
}

#[ic_cdk::post_upgrade]
fn post_upgrade(time: u64) {
    init(time)
}

#[ic_cdk::update]
async fn update_data() {
    utils::init().await
}

#[ic_cdk::query]
fn authentication(principal: String) -> Option<types::Profile> {
    utils::authentication(principal)
}

#[ic_cdk::update]
async fn update_profile(principal: String, username: String, pob: String, instrument: String, avatar: Vec<u8>) -> types::Profile {
    utils::update_profile(principal, username, avatar, pob, instrument).await
}

#[ic_cdk::query]
fn get_original_tune_list(page_number: i32) -> (Vec<String>, i32) {
    utils::get_original_tune_list(page_number)
}

#[ic_cdk::query]
fn get_original_tune(title: String) -> String {
    utils::get_original_tune(title)
}

#[ic_cdk::query]
fn get_user_tune_list(principal: String, page_number: i32) -> (Vec<types::Tuneinfo>, i32) {
    utils::get_user_tune_list(principal, page_number)
}

#[ic_cdk::query]
fn get_user_tune(principal: String, title: String) -> String {
    utils::get_user_tune(principal, title)
}

#[ic_cdk::update]
async fn add_tune(principal: String, title: String, tune_data: String, origin: bool) -> bool {
    utils::add_tune(principal, title, tune_data, origin).await
}

#[ic_cdk::update]
async fn update_tune(principal: String, title: String, tune_data: String, origin: bool) -> bool {
    utils::update_tune(principal, title, tune_data, origin).await
}

#[ic_cdk::query]
pub fn get_friends(principal: String) -> Vec<types::Friend> {
    utils::get_friends(principal)
}

#[ic_cdk::update]
pub async fn send_friend_request(sender: String, receiver: String) -> Option<types::Friend> {
    utils::send_friend_request(sender, receiver).await
}

#[ic_cdk::update]
pub async fn accept_friend_request(sender: String, receiver: String) -> bool {
    utils::accept_friend_request(sender, receiver).await
}

#[ic_cdk::query]
pub fn filter_tunes(title:String, rithm: String, key: String, page_num: i32) -> (Vec<types::Tuneinfo>, i32) {
    utils::filter_tunes(title.as_str(), rithm.as_str(), key.as_str(), page_num)
}

#[ic_cdk::query]
pub fn browse_people(principal: String, filter: String, page_num:i32) -> (Vec<types::Friend>, i32) {
    utils::browse_people(principal, filter, page_num)
}

#[ic_cdk::query]
pub fn get_new_tunes_from_friends(principal: String) -> Vec<types::Tune> {
    utils::get_new_tunes_from_friends(principal)
}

#[ic_cdk::query]
pub fn get_sessions(sub_name: String, page_num: i32) -> (Vec<types::Session>, i32) {
    utils::get_sessions(sub_name.as_str(), page_num)
}

#[ic_cdk::update]
pub fn add_session(principal: String, name: String, location: String, daytime: String, contact: String, comment: String) -> bool {
    utils::add_session(principal, name, location, daytime, contact, comment)
}

#[ic_cdk::update]
pub fn update_session(id: u32, principal: String, name: String, location: String, daytime: String, contact: String, comment: String) -> bool {
    utils::update_session(id, principal, name, location, daytime, contact, comment)
}