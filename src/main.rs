use chrono::{DateTime, ParseError};
use chrono_tz::Canada::Newfoundland;
use chrono_tz::Tz;
use scraper::{Html, Selector};

use serde::Serialize;

#[derive(Serialize, Debug)]
struct Outage {
    location: Option<String>,
    outage_type: Option<String>,
    estimated_start_time: Option<DateTime<Tz>>,
    estimated_restore_time: Option<DateTime<Tz>>,
    cause: Option<String>,
    customers_affected: Option<String>,
}

fn parse_time(time_str: &str) -> Result<DateTime<Tz>, ParseError> {
    let dt = DateTime::parse_from_str(time_str, "%a %b %d, %Y %I:%M %p")?;

    let dt = dt.with_timezone(&Newfoundland);

    Ok(dt)
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
            location: None,
            outage_type: None,
            estimated_start_time: None,
            estimated_restore_time: None,
            cause: None,
            customers_affected: None,
        };

        for row in element.select(&row_selector) {
            let div_selector = Selector::parse("div").unwrap();

            let label_div = row.select(&div_selector).next().unwrap();
            let label = label_div.text().collect::<Vec<_>>().join(" ");
            let value_div = row.select(&div_selector).nth(1).unwrap();
            let value = value_div.text().collect::<Vec<_>>().join(" ");

            match label.as_str() {
                "Outage Type:" => outage.outage_type = Some(value),
                "Location:" => outage.location = Some(value),
                "Est. Start:" => {
                    outage.estimated_start_time = Some(parse_time(value.as_str()).unwrap())
                }
                "Est. Restore:" => {
                    outage.estimated_restore_time = Some(parse_time(value.as_str()).unwrap())
                }
                "Cause:" => outage.cause = Some(value),
                "Cust. Affected:" => outage.customers_affected = Some(value),
                _ => panic!("Unknown label: {}", label),
            }
        }

        outages.push(outage);
    }

    let json = serde_json::to_string(&outages).unwrap();

    println!("{}", json);
}
