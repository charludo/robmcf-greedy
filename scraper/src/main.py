"""
Provide utils for extracting data from the ISR and converting it to something usable.
"""

import csv

# from src.extract.csv_from_raw import csv_from_raw
from src.extract.network_from_csv import generate_network
from src.extract.util import clean
from src.extract.vertices_from_csv import generate_missing_vertices

# This changes with every export!!
# keymap = {
#     "Streckennummer": 2463,
#     "Streckenabschnitt": 2464,
#     "Gleisart": 2471,
#     "Länge": 2479,
#     "Höchstgeschwindigkeit": 2483,
#     "Bremsweg": 2546,
# }
# csv_from_raw("input/isr_export.html", "output/isr_extracted.csv", keymap)

with open("output/isr_extracted.csv", "r", encoding="utf-8") as f:
    reader = list(csv.reader(f))
tracks = [clean(row) for row in reader[1:]]

vertices = generate_missing_vertices(tracks, "output/vertices.csv")
network = generate_network(vertices, tracks, "output/network.json")
