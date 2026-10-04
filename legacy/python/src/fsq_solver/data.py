import os
import tomllib
from pathlib import Path


def data_dir() -> Path:
    if env := os.environ.get("FSQ_DATA"):
        return Path(env)
    for p in Path(__file__).resolve().parents:
        if (p / "data" / "formulas").is_dir():
            return p / "data"
    raise FileNotFoundError("shared data/ folder not found (set FSQ_DATA)")


def load(rel: str) -> dict:
    with open(data_dir() / rel, "rb") as f:
        return tomllib.load(f)
