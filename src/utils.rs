use std::time;

pub fn format_time(t: &time::Duration) -> String {
    let seconds = t.as_secs() % 60;
    let minutes = (t.as_secs() / 60) % 60;
    let hours = (t.as_secs() / 60) / 60;
    format!("{}:{}:{}", hours, minutes, seconds)
}

pub fn read_to_string<T: std::io::Read>(r: &mut T) -> String {
    let mut s_buf = String::with_capacity(512);
    let mut s = String::new();
    while let Ok(size) = r.read_to_string(&mut s_buf) {
        if size == 0 {
            break;
        }
        s.push_str(&s_buf[0..size]);
    }
    s
}
