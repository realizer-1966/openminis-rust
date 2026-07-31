// 일일 로그 헬퍼

use chrono::{Datelike, Local};

pub fn today_filename() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

pub fn yesterday_filename() -> String {
    let yesterday = Local::now() - chrono::Duration::days(1);
    yesterday.format("%Y-%m-%d").to_string()
}
