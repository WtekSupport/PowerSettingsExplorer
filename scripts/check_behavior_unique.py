# -*- coding: utf-8 -*-
import re
from collections import Counter
from pathlib import Path

text = Path(__file__).resolve().parent.parent / "src" / "power" / "catalog.rs"
raw = text.read_text(encoding="utf-8")
behaviors: list[str] = []
for m in re.finditer(r'body_ru: "(.*?)",\s*\n\s*body_en:', raw, re.DOTALL):
    body = m.group(1).replace("\\n", "\n").replace('\\"', '"')
    if "Поведение\n\n" in body:
        b = body.split("Поведение\n\n", 1)[1].split("\n\n", 1)[0]
        behaviors.append(b)

c = Counter(behaviors)
dups = [(k, v) for k, v in c.items() if v > 1]
print(f"entries={len(behaviors)} unique_behaviors={len(c)} duplicate_groups={len(dups)}")
for k, v in sorted(dups, key=lambda x: -x[1])[:10]:
    print(f"  x{v}: {k[:160]}")
print("where_block=", raw.count("Где задать"))
print("visibility_block=", raw.count("Видимость в этой программе"))
