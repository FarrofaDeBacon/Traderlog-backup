import json
import os

def sync_dict(base, target):
    """Recursively sync target dict with base dict keys."""
    for key, value in base.items():
        if key not in target:
            target[key] = value
            print(f"  + Added missing key: {key}")
        elif isinstance(value, dict) and isinstance(target[key], dict):
            sync_dict(value, target[key])
    
    # Remove extra keys
    keys_to_remove = [k for k in target if k not in base]
    for k in keys_to_remove:
        del target[k]
        print(f"  - Removed extra key: {k}")

def main():
    locales_dir = r'c:\PROJETOS\TraderLogPro\src\lib\i18n\locales'
    base_file = os.path.join(locales_dir, 'pt-BR.json')
    others = ['en-US.json', 'es-ES.json', 'fr-FR.json']

    with open(base_file, 'r', encoding='utf-8') as f:
        base_data = json.load(f)

    for other in others:
        other_path = os.path.join(locales_dir, other)
        if os.path.exists(other_path):
            print(f"\nSyncing {other}...")
            with open(other_path, 'r', encoding='utf-8') as f:
                target_data = json.load(f)
            
            sync_dict(base_data, target_data)
            
            with open(other_path, 'w', encoding='utf-8') as f:
                json.dump(target_data, f, ensure_ascii=False, indent=4)
            print(f"Done syncing {other}.")

if __name__ == "__main__":
    main()
