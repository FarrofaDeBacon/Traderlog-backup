
import json
import os

def restructure_json(file_path):
    with open(file_path, 'r', encoding='utf8') as f:
        data = json.load(f)

    # 1. Prepare segments
    settings = data.get('settings', {})
    api = settings.get('api', {})
    integrations = api.get('integrations', {})
    
    # Extract from integrations
    assets = integrations.pop('assets', None)
    asset_types = integrations.pop('assetTypes', None)
    modalities = integrations.pop('modalities', None)
    psychology_data = integrations.pop('psychology', None)

    # 2. Extract from root or forms
    accounts = data.pop('accounts', None)
    currencies = data.pop('currencies', None)
    fees = data.pop('fees', None)
    
    # Handle forms (might be at root or under settings)
    forms_root = data.pop('forms', {})
    forms_settings = settings.pop('forms', {})
    forms = {**forms_root, **forms_settings}
    
    # Extract risk (might be at root or in forms)
    risk = data.pop('risk', None)
    if risk is None:
        risk = forms.pop('risk', None)

    # 3. Rebuild settings
    new_settings = {
        "nav": settings.get('nav'),
        "general": settings.get('general'),
        "profile": settings.get('profile'),
        "api": {
            "form": api.get('form'),
            "integrations": integrations
        },
        "assets": assets,
        "assetTypes": asset_types,
        "accounts": accounts,
        "currencies": currencies,
        "fees": fees,
        "risk": risk,
        "modalities": modalities,
        "forms": forms
    }

    # 4. Final structure reassembly in preferred order
    final_data = {
        "general": data.get('general'),
        "filters": data.get('filters'),
        "strategy": data.get('strategy'),
        "sidebar": data.get('sidebar'),
        "auth": data.get('auth'),
        "home": data.get('home'),
        "journal": data.get('journal'),
        "trades": data.get('trades'),
        "finance": data.get('finance'),
        "psychology": psychology_data,
        "settings": new_settings
    }

    # Remove None values
    final_data = {k: v for k, v in final_data.items() if v is not None}
    
    with open(file_path, 'w', encoding='utf8') as f:
        json.dump(final_data, f, indent=4, ensure_ascii=False)
    
    print(f"Restructured {file_path}")

paths = [
    r'c:\PROJETOS\TraderLogPro\src\lib\i18n\locales\pt-BR.json',
    r'c:\PROJETOS\TraderLogPro\src\lib\i18n\locales\en-US.json'
]

for p in paths:
    if os.path.exists(p):
        restructure_json(p)
    else:
        print(f"File not found: {p}")
