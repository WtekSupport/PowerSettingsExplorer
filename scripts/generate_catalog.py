# -*- coding: utf-8 -*-
import csv
import re
from pathlib import Path

from catalog_behaviors import explain_en as explain_detail_en_from_lib
from catalog_behaviors import explain_ru as explain_detail_from_lib
from catalog_manual_ru import MANUAL

ROOT = Path(__file__).resolve().parent.parent
CSV = ROOT / "settings_export.csv"
OUT = ROOT / "src" / "power" / "catalog.rs"

SUBGROUP_RU = {
    "fea3413e-7e05-4911-9a71-700331f1c294": "общие параметры схемы",
    "0012ee47-9041-4b5d-9b77-535fba8b1442": "жёсткий диск",
    "238c9fa8-0aad-41ed-83f4-97be242c8f20": "сон и гибернация",
    "2a737441-1930-4402-8d77-b2bebba308a3": "USB",
    "2e601130-5351-4d9d-8e04-252966bad054": "устойчивость при простое",
    "4f971e89-eebd-4455-a8de-9e59040e7347": "кнопки питания и крышка",
    "501a4d13-42af-4429-9fd1-a8218c268e20": "PCI Express",
    "54533251-82be-4824-96c1-47b60b740d00": "процессор",
    "7516b95f-f776-4464-8c53-06167f40cc99": "экран и видео",
    "e73a048d-bf27-4f12-9731-8b2076e8891f": "батарея",
    "9596fb26-9850-41fd-ac3e-f7c3c00afd4b": "мультимedia",
    "de830923-a562-41af-a086-e3a2c6bad2da": "энергосбережение",
    "8619b916-e004-4dd8-9b66-dae86f806698": "датчики присутствия",
    "48672f38-7a9a-4bb2-8bf8-3d85be19de4e": "прерывания (IRQ)",
    "44f3beca-a7c0-460e-9df2-bb8b99e0cba6": "графика Intel",
    "5fb4938d-1ee8-4b0f-9a3c-5036b0ab995c": "графика",
}

SUBGROUP_EN = {
    "fea3413e-7e05-4911-9a71-700331f1c294": "general",
    "0012ee47-9041-4b5d-9b77-535fba8b1442": "hard disk",
    "238c9fa8-0aad-41ed-83f4-97be242c8f20": "sleep",
    "2a737441-1930-4402-8d77-b2bebba308a3": "USB",
    "2e601130-5351-4d9d-8e04-252966bad054": "idle resiliency",
    "4f971e89-eebd-4455-a8de-9e59040e7347": "power buttons and lid",
    "501a4d13-42af-4429-9fd1-a8218c268e20": "PCI Express",
    "54533251-82be-4824-96c1-47b60b740d00": "processor",
    "7516b95f-f776-4464-8c53-06167f40cc99": "display",
    "e73a048d-bf27-4f12-9731-8b2076e8891f": "battery",
    "9596fb26-9850-41fd-ac3e-f7c3c00afd4b": "multimedia",
    "de830923-a562-41af-a086-e3a2c6bad2da": "energy saver",
    "8619b916-e004-4dd8-9b66-dae86f806698": "presence aware power",
    "48672f38-7a9a-4bb2-8bf8-3d85be19de4e": "interrupt steering",
    "44f3beca-a7c0-460e-9df2-bb8b99e0cba6": "Intel graphics",
    "5fb4938d-1ee8-4b0f-9a3c-5036b0ab995c": "graphics",
}

TITLE_OVERRIDES = {
    "4faab71a-92e5-4726-b531-224559672d19": "Политика простоя устройств",
    "68afb2d9-ee95-47a8-8f50-4115088073b1": "Режим ожидания при отключении от сети",
    "f15576e8-98b7-4186-b944-eafa664402d9": "Сеть в режиме сна",
    "6738e2c4-e8a5-4a42-b16a-e040e769756e": "Отключение жёсткого диска через",
    "0e796bdb-100d-47d6-a2d5-f7d2daa51f51": "Требовать пароль при пробуждении",
    "245d8541-3943-4422-b025-13a784f679b7": "Тип схемы электропитания",
    "be337238-0d82-4146-a960-4f3749d470c7": "Режим ускорения процессора (Turbo Boost)",
    "93b8b6dc-0698-4d1c-9ee4-0644e900c85d": "Политика планирования потоков (P/E-ядра)",
}


def clean_en(en: str) -> str:
    en = re.sub(r"^[-\d;,]+", "", en or "").strip()
    return en


def title_ru(en: str, guid: str) -> str:
    g = guid.lower()
    if g in MANUAL:
        return MANUAL[g][0]
    if g in TITLE_OVERRIDES:
        return TITLE_OVERRIDES[g]
    en = clean_en(en)
    if not en:
        return f"Параметр ({guid[:8]}…)"
    return en


def explain_detail(en: str, subgroup: str) -> tuple[str, str, str]:
    sg_name = SUBGROUP_RU.get(subgroup.lower(), "другие параметры")
    return explain_detail_from_lib(en, subgroup, sg_name)




def explain_detail_en(en: str, subgroup: str) -> tuple[str, str, str]:
    sg_name = SUBGROUP_EN.get(subgroup.lower(), "other settings")
    return explain_detail_en_from_lib(en, subgroup, sg_name)


def section(title: str, text: str) -> str | None:
    t = (text or "").strip()
    if not t:
        return None
    return f"{title}\n\n{t}"


def compose_body_ru(title: str, en: str, sg_name: str, purpose: str, behavior: str, tips: str) -> str:
    intro = f"«{title}» · раздел «{sg_name}»."
    if en and en.strip() and title != en:
        intro += f"\nИмя в Windows: «{en}»."

    blocks: list[str] = [intro]
    for block in (
        section("Назначение", purpose),
        section("Поведение", behavior),
        section("На практике", tips),
    ):
        if block:
            blocks.append(block)
    return "\n\n".join(blocks)


def compose_body_en(title: str, en: str, sg_name: str, purpose: str, behavior: str, tips: str) -> str:
    intro = f"“{title}” · section “{sg_name}”."
    if en and en.strip() and title != en:
        intro += f"\nWindows name: “{en}”."

    blocks: list[str] = [intro]
    for block in (
        section("Purpose", purpose),
        section("Behavior", behavior),
        section("In practice", tips),
    ):
        if block:
            blocks.append(block)
    return "\n\n".join(blocks)


def title_en(en: str, guid: str) -> str:
    g = guid.lower()
    en = clean_en(en)
    if g in MANUAL:
        return en or MANUAL[g][0]
    if not en:
        return f"Setting ({guid[:8]}…)"
    return en


def body_ru(en: str, guid: str, subgroup: str) -> str:
    g = guid.lower()
    if g in MANUAL:
        return MANUAL[g][1]

    en = clean_en(en)
    title = title_ru(en, guid)
    sg_name = SUBGROUP_RU.get(subgroup.lower(), "другие параметры")

    if not en:
        return compose_body_ru(
            f"Параметр ({g[:8]}…)",
            "",
            sg_name,
            "Служебный или OEM-параметр без отображаемого имени в экспорте.",
            f"Идентификатор {g} — в списке дополнительных параметров отображается без FriendlyName; "
            f"семантика задаётся драйвером или прошивкой OEM, не универсальным описанием Windows.",
            "Меняйте только по документации производителя; для диагностики сравните значения «От сети» и «От батареи».",
        )

    purpose, behavior, tips = explain_detail(en, subgroup)
    return compose_body_ru(title, en, sg_name, purpose, behavior, tips)


def body_en(en: str, guid: str, subgroup: str) -> str:
    g = guid.lower()
    en = clean_en(en)
    title = title_en(en, guid)
    sg_name = SUBGROUP_EN.get(subgroup.lower(), "other settings")

    if not en:
        return compose_body_en(
            f"Setting ({g[:8]}…)",
            "",
            sg_name,
            "A service or OEM setting without a display name in the export.",
            "It may belong to a driver or a vendor-specific capability.",
            "Change it only if you have vendor documentation.",
        )

    purpose, behavior, tips = explain_detail_en(en, subgroup)
    return compose_body_en(title, en, sg_name, purpose, behavior, tips)


def esc(s: str) -> str:
    return (
        s.replace("\\", "\\\\")
        .replace('"', '\\"')
        .replace("\r", "")
        .replace("\n", "\\n")
    )


def main():
    seen = {}
    rows = []
    with CSV.open(encoding="utf-8-sig", newline="") as f:
        for row in csv.DictReader(f):
            g = row["Setting"].lower()
            if g in seen:
                continue
            seen[g] = True
            rows.append(row)

    lines = [
        "// Auto-generated by scripts/generate_catalog.py",
        "",
        "use crate::i18n::Lang;",
        "",
        "pub struct CatalogEntry {",
        "    pub guid: &'static str,",
        "    pub title_ru: &'static str,",
        "    pub title_en: &'static str,",
        "    pub body_ru: &'static str,",
        "    pub body_en: &'static str,",
        "}",
        "",
        "impl CatalogEntry {",
        "    pub fn title(&self, lang: Lang) -> &'static str {",
        "        match lang {",
        "            Lang::Ru => self.title_ru,",
        "            Lang::En => self.title_en,",
        "        }",
        "    }",
        "",
        "    pub fn body(&self, lang: Lang) -> &'static str {",
        "        match lang {",
        "            Lang::Ru => self.body_ru,",
        "            Lang::En => self.body_en,",
        "        }",
        "    }",
        "}",
        "",
        "pub static ENTRIES: &[CatalogEntry] = &[",
    ]
    for row in rows:
        g = row["Setting"].lower()
        sg = row.get("Subgroup", "")
        en_name = row["EN"]
        tr = esc(title_ru(en_name, g))
        te = esc(title_en(en_name, g))
        br = esc(body_ru(en_name, g, sg))
        be = esc(body_en(en_name, g, sg))
        lines.extend(
            [
                "    CatalogEntry {",
                f'        guid: "{g}",',
                f'        title_ru: "{tr}",',
                f'        title_en: "{te}",',
                f'        body_ru: "{br}",',
                f'        body_en: "{be}",',
                "    },",
            ]
        )
    lines.extend(
        [
            "];",
            "",
            "pub fn lookup(setting_guid: &str) -> Option<&'static CatalogEntry> {",
            "    let g = setting_guid.to_lowercase();",
            "    ENTRIES.iter().find(|e| e.guid == g)",
            "}",
            "",
        ]
    )
    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"Wrote {len(rows)} bilingual entries to {OUT}")


if __name__ == "__main__":
    main()
