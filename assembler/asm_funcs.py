def include_bytes(file: str) -> bytes:
    with open(file, 'rb') as f:
        return f.read()

def include_str(file: str) -> bytes:
    with open(file, 'r') as f:
        return f.read()