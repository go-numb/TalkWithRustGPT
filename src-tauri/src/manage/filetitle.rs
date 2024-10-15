use chrono::format;

pub fn to_title(s: &str) -> String {
    // get tags
    let str = s.split_whitespace().collect::<Vec<&str>>();
    let tags = str
        .iter()
        .filter(|&x| x.starts_with("#"))
        .collect::<Vec<&&str>>();

    let date = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    if !tags.is_empty() {
        let s = tags.first().unwrap();
        // delete   #  from title
        let title = s.chars().skip(1).collect::<String>();
        format!("memo-{}-{}.txt", date, title)
    } else {
        let date = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let filename = format!("memo-{}.txt", date);
        filename
    }
}
