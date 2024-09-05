"""
Generate a network in the format expected by the solver from the preprocessed data
"""

import json
import math

from .util import TrackType


def generate_network(vertices, tracks, out_file):
    """
    Using the pre-processed tracks and vertices, generate a network.
    Dump this as JSON in the format the solver understands.

    Costs are averaged over capacities when multiple tracks with the same source and sink exist.
    """
    vertex_ids = {v["name"]: i for i, v in enumerate(vertices)}

    capacities = [[0 for _ in vertex_ids] for _ in vertex_ids]
    costs = [[0 for _ in vertex_ids] for _ in vertex_ids]

    for track in tracks:
        s = vertex_ids[track["s"]]
        t = vertex_ids[track["t"]]

        capacity_to = 0
        capacity_rev = 0
        if track["type"] == TrackType.SIMPLEX:
            capacity_to += track["capacity"]
        if track["type"] == TrackType.DUPLEX:
            capacity_to += track["capacity"]
            capacity_rev += track["capacity"]
        if track["type"] == TrackType.DUPLEX_HALFED:
            capacity_to += math.ceil(track["capacity"] / 2)
            capacity_rev += math.ceil(track["capacity"] / 2)

        average_cost_to = round(
            (capacities[s][t] * costs[s][t])
            + (capacity_to * track["cost"]) / (capacities[s][t] + capacity_to)
        )

        average_cost_rev = (
            round(
                (capacities[t][s] * costs[t][s])
                + (capacity_rev * track["cost"]) / (capacities[t][s] + capacity_rev)
            )
            if capacity_rev > 0
            else 0
        )

        capacities[s][t] += capacity_to
        capacities[t][s] += capacity_rev

        costs[s][t] += average_cost_to
        costs[t][s] += average_cost_rev

    network = {
        "vertices": list(vertices),
        "capacities": capacities,
        "costs": costs,
        "balances": [],  # todo: how to generate these?
        "fixed_arcs": [],
    }

    with open(out_file, "w", encoding="utf-8") as f:
        f.write(json.dumps(network))

    return network
