use bouyomi4rs::{BouyomiClient, TalkConfig};

pub fn say(msg: &str) {
    let mut config = TalkConfig::default();
    config
        .set_voice(10002)
        .set_volume(110)
        .set_speed(88)
        .set_tone(105);
    let client = BouyomiClient::new().set_config(config);

    match client.talk(msg) {
        Ok(_) => println!("bouyomi4rs: success"),
        Err(e) => println!("bouyomi4rs: it is possible that bouyomi-chan is not activated.: {}", e),
    };
}
