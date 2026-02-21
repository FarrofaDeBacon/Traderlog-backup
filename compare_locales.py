import json
import os

files = [
    r"c:\PROJETOS\TraderLogPro\src\lib\i18n\locales\pt-BR.json",
    r"c:\PROJETOS\TraderLogPro\src\lib\i18n\locales\en-US.json",
    r"c:\PROJETOS\TraderLogPro\src\lib\i18n\locales\es-ES.json",
    r"c:\PROJETOS\TraderLogPro\src\lib\i18n\locales\fr-FR.json"
]

def get_keys(d, parent_key=''):
    keys = set()
    for k, v in d.items():
        new_key = f"{parent_key}.{k}" if parent_key else k
        keys.add(new_key)
        if isinstance(v, dict):
            keys.update(get_keys(v, new_key))
    return keys

all_data = {}
all_keys_per_file = {}
superset_keys = set()

for f in files:
    with open(f, 'r', encoding='utf-8') as jf:
        data = json.load(jf)
        all_data[f] = data
        keys = get_keys(data)
        all_keys_per_file[f] = keys
        superset_keys.update(keys)

print(f"Total unique keys across all files: {len(superset_keys)}")

for f in files:
    missing = superset_keys - all_keys_per_file[f]
    # Filter missing keys: if a parent key is missing, its children will also be missing.
    # We only care about leaf keys or specific missing nodes.
    # Actually, let's just show top-level or significant missing keys.
    if missing:
        print(f"\nMissing keys in {os.path.basename(f)} ({len(missing)}):")
        # Sort and group to make it readable
        sorted_missing = sorted(list(missing))
        for m in sorted_missing[:20]: # show first 20
            print(f"  - {m}")
        if len(sorted_missing) > 20:
            print(f"  ... and {len(sorted_missing) - 20} more.")
    else:
        print(f"\nNo keys missing in {os.path.basename(f)}")
