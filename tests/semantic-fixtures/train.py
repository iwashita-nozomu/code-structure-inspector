from cache import parse_count
from renderer import total

def summarize_rows(rows):
    counts = []
    for row in rows:
        counts.append(parse_count(row))
    return total(counts)
