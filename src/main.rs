use chrono::{DateTime, NaiveDateTime, ParseError, TimeZone};
use chrono_tz::Canada::Newfoundland;
use chrono_tz::Tz;
use scraper::{Html, Selector};

use serde::Serialize;

#[derive(Serialize, Debug)]
struct Outage {
    outage_type: Option<String>,
    locations: Option<Vec<String>>,
    estimated_start_time: Option<DateTime<Tz>>,
    estimated_restore_time: Option<DateTime<Tz>>,
    cause: Option<String>,
    customers_affected: Option<String>,
    crew_status: Option<String>,
}

fn parse_time(time_str: &str) -> Result<Option<DateTime<Tz>>, ParseError> {
    let time_str = time_str.split_whitespace().collect::<Vec<_>>().join(" ");

    // if time string is "Unknown", return None

    if time_str == "Unknown" {
        return Ok(None);
    }

    let dt = NaiveDateTime::parse_from_str(time_str.as_str(), "%a %b %e, %Y %l:%M %p")?;

    let dt = Newfoundland.from_local_datetime(&dt).unwrap();

    Ok(Some(dt))
}

fn main() {
    let response = reqwest::blocking::get(
        "https://www.newfoundlandpower.com/api/sitecore/iFrameMap/DisplayOutageList?planned=true&unplanned=true"
    )
    .unwrap()
    .text()
    .unwrap();

    let document = Html::parse_document(&response);

    let selector = Selector::parse(".info-content").unwrap();

    let mut outages = Vec::new();

    for element in document.select(&selector) {
        // find each div with class row
        let row_selector = Selector::parse(".row").unwrap();

        let mut outage = Outage {
            locations: None,
            outage_type: None,
            estimated_start_time: None,
            estimated_restore_time: None,
            cause: None,
            customers_affected: None,
            crew_status: None,
        };

        for row in element.select(&row_selector) {
            let div_selector = Selector::parse("div").unwrap();

            let label_div = row.select(&div_selector).next().unwrap();
            let label = label_div.text().collect::<Vec<_>>().join(" ");
            let value_div = row.select(&div_selector).nth(1).unwrap();
            let value = value_div.text().collect::<Vec<_>>().join(" ");

            match label.as_str() {
                "Outage Type:" => outage.outage_type = Some(value),
                "Location:" => {
                    outage.locations =
                        Some(value.split(",").map(|s| s.trim().to_string()).collect())
                }
                "Est. Start:" => outage.estimated_start_time = parse_time(value.as_str()).unwrap(),
                "Est. Restore:" => {
                    outage.estimated_restore_time = parse_time(value.as_str()).unwrap()
                }
                "Cause:" => outage.cause = Some(value),
                "Cust. Affected:" => outage.customers_affected = Some(value),
                "Crew Status:" => outage.crew_status = Some(value),
                _ => panic!("Unknown label: {}", label),
            }
        }

        outages.push(outage);
    }

    let json = serde_json::to_string(&outages).unwrap();

    println!("{}", json);
}
