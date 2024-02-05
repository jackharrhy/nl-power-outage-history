from dataclasses import dataclass
from datetime import datetime
import json

import httpx
from bs4 import BeautifulSoup

url = "https://www.newfoundlandpower.com/api/sitecore/iFrameMap/DisplayOutageList"
headers = {"User-Agent": "github.com/jackharrhy/nl-power-outage-history"}
params = {
    "planned": "true",
    "unplanned": "true",
}


@dataclass
class Outage:
    outage_type: str | None = None
    location: str | None = None
    estimated_start_time: datetime | None = None
    estimated_restore_time: datetime | None = None
    cause: str | None = None
    customers_affected: int | None = None


def to_json(object) -> str:
    def default(obj):
        if isinstance(obj, datetime):
            return obj.isoformat()

        return obj.__dict__

    return json.dumps(object, default=default, indent=4)


class NoOutages:
    pass


def get_outages() -> NoOutages | list[Outage]:
    response = httpx.get(url, params=params)
    soup = BeautifulSoup(response.text, "html.parser")

    info_contents = soup.find_all("div", class_="info-content")

    if len(info_contents) == 0:
        return NoOutages()

    outages: list[Outage] = []

    for info_content in info_contents:
        rows = info_content.find_all("div", class_="row")
        outage = Outage()

        for row in rows:
            [description, value] = row.find_all("div")
            description = description.text[:-1]

            match description:
                case "Outage Type":
                    outage.outage_type = value.text
                case "Location":
                    outage.location = value.text
                case "Est. Start":
                    try:
                        outage.estimated_start_time = datetime.strptime(
                            value.text, "%a %b %d, %Y %I:%M %p"
                        )
                    except ValueError:
                        pass
                case "Est. Restore":
                    try:
                        outage.estimated_restore_time = datetime.strptime(
                            value.text, "%a %b %d, %Y %I:%M %p"
                        )
                    except ValueError:
                        pass
                case "Cause":
                    outage.cause = value.text
                case "Cust. Affected":
                    outage.customers_affected = int(value.text)

        outages.append(outage)

    return outages


def test():
    response = get_outages()
    print(response)


def as_json():
    response = get_outages()
    print(to_json(response))
