use bouyomi4rs::{BouyomiClient, TalkConfig};

pub fn say(voice_id: i16, msg: &str) -> Result<(), String> {
    let mut config = TalkConfig::default();
    config
        .set_voice(voice_id)
        .set_volume(110)
        .set_speed(88)
        .set_tone(105);
    let client = BouyomiClient::new().set_config(config);

    match client.talk(msg) {
        Ok(_) => {
            println!("bouyomi4rs: success");
            Ok(())
        }
        Err(e) => Err(format!(
            "bouyomi4rs: it is possible that bouyomi-chan is not activated.: {}",
            e
        )),
    }
}
