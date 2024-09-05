"""
Provide utils for extracting data from the ISR and converting it to something usable.

Functions in this file each use their own export form the ISR, but share vertex data.
"""

import csv
from pathlib import Path

from src.extract.csv_from_raw import csv_from_raw
from src.extract.network_from_csv import generate_network
from src.extract.util import clean
from src.extract.vertices_from_csv import generate_missing_vertices


def _generate(keymap, center, in_file, out_file):
    csv_from_raw(in_file, "output/temp_isr_extracted.csv", keymap)
    with open("output/temp_isr_extracted.csv", "r", encoding="utf-8") as f:
        reader = list(csv.reader(f))
    tracks = [clean(row) for row in reader[1:]]
    Path("output/temp_isr_extracted.csv").unlink()
    vertices = generate_missing_vertices(tracks, center, "output/vertices.csv")
    return generate_network(vertices, tracks, out_file)


def generate_full():
    """
    Generate the largest possible network.
    """
    keymap = {
        "Streckennummer": 2463,
        "Streckenabschnitt": 2464,
        "Gleisart": 2471,
        "Länge": 2479,
        "Höchstgeschwindigkeit": 2483,
        "Bremsweg": 2546,
    }
    center = {"lat": 51.9, "lng": 6.7}
    _ = _generate(
        keymap, center, "input/isr_export_full.html", "output/network_full.json"
    )
