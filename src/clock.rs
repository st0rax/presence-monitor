//! UTC timestamps without chrono (rein-Rust `time` crate).

use time::macros::format_description;
use time::OffsetDateTime;

const LOG_FMT: &[time::format_description::FormatItem<'_>] =
    format_description!("[year]-[month padding:zero]-[day padding:zero]T[hour padding:zero]:[minute padding:zero]:[second padding:zero]");

const STAMP_FMT: &[time::format_description::FormatItem<'_>] = format_description!(
    "[year][month padding:zero][day padding:zero]_[hour padding:zero][minute padding:zero][second padding:zero][subsecond digits:3]"
);

pub fn now_utc() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

pub fn log_timestamp() -> String {
    now_utc()
        .format(LOG_FMT)
        .unwrap_or_else(|_| "unknown".to_string())
}

pub fn file_stamp() -> String {
    now_utc()
        .format(STAMP_FMT)
        .unwrap_or_else(|_| "unknown".to_string())
}

pub fn rfc3339_now() -> String {
    now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string())
}