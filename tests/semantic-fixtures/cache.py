def parse_count(text):
    cleaned = text.strip()
    if not cleaned.isdecimal():
        raise ValueError("expected unsigned decimal")
    return int(cleaned)
