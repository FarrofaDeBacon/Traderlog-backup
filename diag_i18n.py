import json
import os

def get_nested_keys(data, prefix=''):
    keys = set()
    if isinstance(data, dict):
        for k, v in data.items():
            new_key = f"{prefix}.{k}" if prefix else k
            keys.add(new_key)
            keys.update(get_nested_keys(v, new_key))
    return keys

def compare_locales(base_file, target_file):
    with open(base_file, 'r', encoding='utf-8') as f:
        base_data = json.load(f)
    with open(target_file, 'r', encoding='utf-8') as f:
        target_data = json.load(f)
    
    base_keys = get_nested_keys(base_data)
    target_keys = get_nested_keys(target_data)
    
    missing = base_keys - target_keys
    extra = target_keys - base_keys
    
    return missing, extra, len(base_keys), len(target_keys)

locales_dir = r'c:\PROJETOS\TraderLogPro\src\lib\i18n\locales'
base_locale = os.path.join(locales_dir, 'pt-BR.json')
others = ['en-US.json', 'es-ES.json', 'fr-FR.json']

print(f"Base Locale: pt-BR.json")
for other in others:
    other_path = os.path.join(locales_dir, other)
    if os.path.exists(other_path):
        missing, extra, base_count, target_count = compare_locales(base_locale, other_path)
        print(f"\n--- Comparing with {other} ---")
        print(f"Total Keys (Base): {base_count}")
        print(f"Total Keys ({other}): {target_count}")
        print(f"Missing Keys: {len(missing)}")
        print(f"Extra Keys: {len(extra)}")
        if missing:
            print("First 10 missing:")
            for m in list(missing)[:10]:
                print(f"  - {m}")
    else:
        print(f"\nFile {other} not found.")
