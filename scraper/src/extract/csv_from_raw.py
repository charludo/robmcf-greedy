"""
Script to extract relevant column data from a raw HTML export from the ISR
"""

import csv
import html

from bs4 import BeautifulSoup


def csv_from_raw(in_file, out_file, keymap):
    """
    Reads the raw HTML table copied from the ISR and returns only the relevant data as CSV
    """

    with open(in_file, "r", encoding="utf-8") as f:
        content = html.unescape(f.read())

    soup = BeautifulSoup(content, "html.parser")
    rows = soup.find_all("tr", class_="x-grid-row")

    with open(out_file, "w", newline="", encoding="utf-8") as csvfile:
        csvwriter = csv.writer(csvfile, quoting=csv.QUOTE_ALL)
        csvwriter.writerow(keymap.keys())

        for row in rows:
            row_data = []
            try:
                for _, value in keymap.items():
                    row_data.append(
                        html.unescape(
                            row.find("td", class_=f"x-grid-cell-gridcolumn-{value}")
                            .get_text(strip=True)
                            .replace("\n", " ")
                        )
                    )
                csvwriter.writerow(row_data)
            except AttributeError:
                print(row)
