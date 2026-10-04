"""Past FS-Quiz questions solved with fsq, from data/examples.toml: (id, what it asks, fsq command, official answer)."""

from .data import load

EXAMPLES = [(e["id"], e["what"], e["cmd"], e["answer"]) for e in load("examples.toml")["example"]]
