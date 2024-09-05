"""
Use the extracted CSV data to create a valid network
"""

import csv
import os
from pathlib import Path

import googlemaps

from .util import clean


def init_maps_api():
    """
    Init the GMaps API. Provide the key as an env variable.
    """
    api_key = os.environ["GMAPS_API_KEY"]
    return googlemaps.Client(key=api_key)


def search_location(client, s):
    """
    Use google maps to find the coordinates of a location.

    Ignore anything up to and including "-", since google, in its infinite wisdom,
    mistunderstands this as "I want a route from _ to _", instead of recognizing this as a suburb.

    Ignore anything after a ",", since that is DB-specific information which will confuse google.
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


def generate_missing_vertices(tracks, out_file):
    """
    Uses google maps to generate coordinates for all so-far unknown origin vertices.
    For vertices starting with "StrUeb", use the target vertices, since these track segments are
    usually short and their location is not publicly accessible.

    If no result can be found, ask the user for search input.

    Returns a list of all vertices.
    """
    gmaps = init_maps_api()
    centered_on = {
        "lat": 51.9,
        "lng": 6.7,
    }  # Just so the graph representation isn't way off to one side

    vertices = {}

    if Path(out_file).is_file():
        with open(out_file, "r", encoding="utf-8") as f:
            vertices = {
                x[0]: {"name": x[0], "x": x[1], "y": x[2]} for x in list(csv.reader(f))
            }

    for track in [(track["s"], track["t"]) for track in tracks]:
        missing = []
        if track[0] not in vertices:
            missing.append(track)
        if track[1] not in vertices:
            missing.append((track[1], track[0]))

        for s, t in missing:
            print(f"Locating {s}...")

            name = t if s.startswith("StrUeb") else s
            found = False
            while not found:
                try:
                    geolocation = search_location(gmaps, name)
                    found = True
                except IndexError:
                    name = input(f"Not found. Please enter search term for {s}: ")

            vertices[s] = {
                "name": s,
                "x": str(round((geolocation["lat"] - centered_on["lat"]) * 100)),
                "y": str(round((geolocation["lng"] - centered_on["lng"]) * 100)),
            }

            with open(out_file, "a", encoding="utf-8") as f:
                writer = csv.writer(f, quoting=csv.QUOTE_ALL)
                writer.writerow(vertices[s].values())

    return vertices.values()
