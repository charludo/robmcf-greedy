"""
Use the extracted CSV data to create a valid network
"""

import csv
import math
from enum import Enum
from pathlib import Path

import googlemaps


class TrackType(Enum):
    """Available track types"""

    SIMPLEX = "Richtungsgleis"
    DUPLEX = "Gegengleis"
    DUPLEX_HALFED = "eingleisig"


def clean(row):
    """
    Convert raw data from the csv into a usable dict.
    If no values are given, assumes the following defaults:

    length: 10 km
    speed: 50 km/h
    brake length: 1km
    """
    (_, s, track_type, length, speed, brake_ln) = row

    res = {}
    res["s"], res["t"] = s.split(" - ", 1)
    res["track_type"] = TrackType(track_type)

    km, m = (10, 0) if length in ["auf Anfrage"] else length.split(",")
    length = int(km) * 1000 + int(m) + 1  # prevents 0 length
    speed = 50 if speed in ["auf Anfrage"] else int(speed)
    res["cost"] = math.ceil(60 * length / (speed * 1000))

    brake_ln = (
        1000
        if brake_ln in ["Kein Dokument vorhanden", "auf Anfrage"]
        else int(brake_ln.split(" ")[0])
    )
    res["capacity"] = math.floor((60 / res["cost"]) * 0.5 * (length / brake_ln))

    return res


def search_location(client, s):
    """
    Use google maps to find the coordinates of a location
    """
    searchable_name = s
    if "-" in searchable_name:
        searchable_name = s.split("-")[1]
    if "," in searchable_name:
        searchable_name = s.split(",")[0]
    return client.find_place(
        searchable_name,
        "textquery",
        fields=["geometry", "geometry/viewport/southwest"],
    )["candidates"][0]["geometry"]["location"]


with open("output.csv", "r", encoding="utf-8") as f:
    reader = list(csv.reader(f))

gmaps = googlemaps.Client(key="")
centered_on = {"lat": 51.9, "lng": 6.7}

tracks = [clean(row) for row in reader[1:]]
vertices = {}

if Path("vertices.csv").is_file():
    with open("vertices.csv", "r", encoding="utf-8") as f:
        vertices = {
            x[0]: {"name": x[0], "x": x[1], "y": x[2]} for x in list(csv.reader(f))
        }
        print(vertices)

for track in tracks:
    if track["s"] not in vertices:
        print(f"Locating {track['s']}...")

        name = track["t"] if track["s"].startswith("StrUeb") else track["s"]
        FOUND = False
        while not FOUND:
            try:
                geolocation = search_location(gmaps, name)
                FOUND = True
            except IndexError:
                name = input(f"Not found. Please enter search term for {track['s']}: ")

        vertices[track["s"]] = {
            "name": track["s"],
            "x": str(round((geolocation["lat"] - centered_on["lat"]) * 100)),
            "y": str(round((geolocation["lng"] - centered_on["lng"]) * 100)),
        }

        with open("vertices.csv", "a", encoding="utf-8") as f:
            writer = csv.writer(f, quoting=csv.QUOTE_ALL)
            writer.writerow(vertices[track["s"]].values())
