use std::time;

pub fn format_time(t: &time::Duration) -> String {
    let seconds = t.as_secs() % 60;
    let minutes = (t.as_secs() / 60) % 60;
    let hours = (t.as_secs() / 60) / 60;
    format!("{}:{}:{}", hours, minutes, seconds)
}
