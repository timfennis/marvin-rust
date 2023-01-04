use std::num::ParseIntError;

use chrono::NaiveDate;
use ical::parser::ParserError;
use tracing::{debug, warn};

pub enum CalendarError {
    ReqwestError(reqwest::Error),
    ParseError(ParserError),
}

#[derive(Debug)]
pub struct Event {
    pub uid: String,
    pub name: String,
    pub date: NaiveDate,
}

pub async fn fetch_calendar_info(url: &str) -> Result<Vec<Event>, CalendarError> {
    debug!(url = url, "fetching calendar information from url");

    let text = reqwest::get(url)
        .await
        .map_err(|e| CalendarError::ReqwestError(e))?
        .text()
        .await
        .map_err(|e| CalendarError::ReqwestError(e))?;

    // let response = response.text().await
    let calendars = ical::IcalParser::new(text.as_bytes());
    let mut events_out = Vec::new();

    for calendar in calendars {
        let calendar = calendar.map_err(|e| CalendarError::ParseError(e))?;
        for event in calendar.events {
            // dbg!(&event);
            let mut uid = None;
            let mut name = None;
            let mut date = None;

            for property in event.properties {
                match (property.name.as_str(), property.value) {
                    ("UID", value) => {
                        uid = value;
                    }
                    ("SUMMARY", value) => {
                        name = value;
                    }
                    ("DTSTART", Some(value)) => date = date_from_string(&value).ok(),
                    _ => {}
                };
            }

            match (uid, name, date) {
                (Some(uid), Some(name), Some(date)) => {
                    debug!(uid, name, date = date.to_string(), "found a valid event");
                    events_out.push(Event {
                        uid: uid,
                        name: name,
                        date: date,
                    });
                }
                _ => {
                    warn!("skipping event that could not be parsed");
                }
            }
        }
    }

    return Ok(events_out);
}

#[derive(Debug, PartialEq)]
enum ParseDateError {
    ParseIntError(ParseIntError),
    InvalidLength(usize),
    NothingParsed(),
}

fn date_from_string(value: &str) -> Result<NaiveDate, ParseDateError> {
    if value.len() != 8 {
        return Err(ParseDateError::InvalidLength(value.len()));
    }

    let year = value[0..=3]
        .parse::<i32>()
        .map_err(|e| ParseDateError::ParseIntError(e));

    let month = value[4..=5]
        .parse::<u32>()
        .map_err(|e| ParseDateError::ParseIntError(e));

    let day = value[6..=7]
        .parse::<u32>()
        .map_err(|e| ParseDateError::ParseIntError(e));

    return match (year, month, day) {
        (Ok(year), Ok(month), Ok(day)) => {
            NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| ParseDateError::NothingParsed())
        }
        _ => Err(ParseDateError::NothingParsed()),
    };
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;

    use super::{date_from_string, ParseDateError};

    #[test]
    fn it_parses_dates_successfully() {
        let date = date_from_string("19910131").unwrap();
        assert_eq!(date.year_ce(), (true, 1991));
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 31);
    }

    #[test]
    fn it_errors_if_the_input_string_has_invalid_length() {
        assert_eq!(
            date_from_string("202301010"),
            Err(ParseDateError::InvalidLength(9))
        );
        assert_eq!(
            date_from_string("2013"),
            Err(ParseDateError::InvalidLength(4))
        );
    }
}
