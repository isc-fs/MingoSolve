from ..core import formula, var
from ..data import data_dir, load

for _path in sorted((data_dir() / "formulas").glob("*.toml")):
    _d = load(f"formulas/{_path.name}")
    for _v in _d.get("var", []):
        var(_v["name"], _v["unit"], _v["desc"], positive=not _v.get("signed", False), default=_v.get("default"))
    for _f in _d.get("formula", []):
        formula(_f["key"], _f["title"], "\n".join(_f["eqs"]), " ".join(_f["tags"]), _f.get("notes", ""))
