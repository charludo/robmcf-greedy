"""
Script to extract relevant column data from a raw HTML export from the ISR
"""

import csv
import html

from bs4 import BeautifulSoup

with open("isr_export.html", "r", encoding="utf-8") as f:
    content = html.unescape(f.read())

soup = BeautifulSoup(content, "html.parser")
rows = soup.find_all("tr", class_="x-grid-row")

# Open the CSV file to write the output
with open("output.csv", "w", newline="", encoding="utf-8") as csvfile:
    csvwriter = csv.writer(csvfile, quoting=csv.QUOTE_ALL)
    csvwriter.writerow(
        [
            "Streckennummer",
            "Streckenabschnitt",
            "Gleisart",
            "Länge",
            "Höchstgeschwindigkeit",
            "Bremsweg",
        ]
    )

    for row in rows:
        try:
            number = html.unescape(
                row.find("td", class_="x-grid-cell-gridcolumn-2463")
                .get_text(strip=True)
                .replace("\n", " ")
            )
            name = (
                row.find("td", class_="x-grid-cell-gridcolumn-2464")
                .get_text(strip=True)
                .replace("\n", " ")
            )
            track = (
                row.find("td", class_="x-grid-cell-gridcolumn-2471")
                .get_text(strip=True)
                .replace("\n", " ")
            )
            length = (
                row.find("td", class_="x-grid-cell-gridcolumn-2479")
                .get_text(strip=True)
                .replace("\n", " ")
            )
            speed = (
                row.find("td", class_="x-grid-cell-gridcolumn-2483")
                .get_text(strip=True)
                .replace("\n", " ")
            )
            brake_len = (
                row.find("td", class_="x-grid-cell-gridcolumn-2546")
                .get_text(strip=True)
                .replace("\n", " ")
            )
            csvwriter.writerow([number, name, track, length, speed, brake_len])
        except AttributeError:
            print(row)
