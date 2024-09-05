"""
Util functions used by multiple modules
"""

import math
from enum import Enum


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
    res["type"] = TrackType(track_type)

    km, m = (10, 0) if length in ["auf Anfrage"] else length.split(",")
    length = max(100, int(km) * 1000 + int(m))  # prevents 0 length
    speed = 50 if speed in ["auf Anfrage"] else int(speed)
    res["cost"] = math.ceil(60 * length / (speed * 1000))

    brake_ln = (
        700
        if brake_ln in ["Kein Dokument vorhanden", "auf Anfrage"]
        else int(brake_ln.split(" ")[0])
    )
    simultaneous_usage = 0.5 * (length / brake_ln)
    traversals_per_hour = 60 / res["cost"]
    res["capacity"] = math.floor(simultaneous_usage * traversals_per_hour)

    return res
