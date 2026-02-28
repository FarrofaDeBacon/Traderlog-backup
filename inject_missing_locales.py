import json
import os

def set_nested(d, path, value):
    keys = path.split('.')
    current = d
    for key in keys[:-1]:
        if key not in current or not isinstance(current[key], dict):
            current[key] = {}
        current = current[key]
    current[keys[-1]] = value

locales_dir = "src/lib/i18n/locales"

# API Form Modal Keys
api_form_pt = {
    "settings.api.form.editTitle": "Editar Configuração de API",
    "settings.api.form.newTitle": "Nova Configuração de API",
    "settings.api.form.description": "Configure uma nova chave de API para habilitar integrações no TraderLog Pro.",
    "settings.api.form.provider": "Provedor",
    "settings.api.form.apiKey": "Chave de API",
    "settings.api.form.apiKeyPlaceholder": "sk-...",
    "settings.api.form.test": "Testar Chave",
    "settings.api.form.dailyLimit": "Limite Diário (Avisos)",
    "settings.api.form.dailyLimitHint": "Notifica quando o uso atingir este limite.",
    "settings.api.form.enabled": "Habilitar esta API",
    "settings.api.form.enabledHint": "APIs desabilitadas não farão requisições.",
    "settings.api.form.noteTitle": "Mantenha suas chaves seguras.",
    "settings.api.form.noteBody": "As chaves de API são armazenadas localmente no seu dispositivo. Nunca as compartilhe.",
    "settings.api.form.getOpenAIApiKey": "Obter chave OpenAI",
    "settings.api.form.getGeminiApiKey": "Obter chave Gemini",
    "settings.api.form.getPolygonApiKey": "Obter chave Polygon",
    "settings.api.form.viewOpenAIPrices": "Ver preços OpenAI",
    "settings.api.form.viewPolygonPlans": "Ver planos Polygon",
    "settings.api.form.cancel": "Cancelar",
    "settings.api.form.save": "Salvar",
    "settings.api.form.testInfo": "Testando chave de API...",
    "settings.api.form.testSuccess": "Chave de API testada com sucesso!",
    "settings.api.form.enterKeyError": "Por favor, insira uma chave de API primeiro.",
    "settings.api.form.providerCustom": "Personalizado",
    "settings.api.form.providers.openai": "OpenAI",
    "settings.api.form.providers.google_gemini": "Google Gemini",
    "settings.api.form.providers.anthropic": "Anthropic Claude",
    "settings.api.form.providers.polygon": "Polygon.io",
    "settings.api.form.providers.alpha_vantage": "Alpha Vantage",
}

api_form_en = {
    "settings.api.form.editTitle": "Edit API Configuration",
    "settings.api.form.newTitle": "New API Configuration",
    "settings.api.form.description": "Configure a new API key to enable integrations in TraderLog Pro.",
    "settings.api.form.provider": "Provider",
    "settings.api.form.apiKey": "API Key",
    "settings.api.form.apiKeyPlaceholder": "sk-...",
    "settings.api.form.test": "Test Key",
    "settings.api.form.dailyLimit": "Daily Limit (Warnings)",
    "settings.api.form.dailyLimitHint": "Notifies you when usage reaches this limit.",
    "settings.api.form.enabled": "Enable this API",
    "settings.api.form.enabledHint": "Disabled APIs will not make requests.",
    "settings.api.form.noteTitle": "Keep your keys secure.",
    "settings.api.form.noteBody": "API keys are stored locally on your device. Never share them.",
    "settings.api.form.getOpenAIApiKey": "Get OpenAI key",
    "settings.api.form.getGeminiApiKey": "Get Gemini key",
    "settings.api.form.getPolygonApiKey": "Get Polygon key",
    "settings.api.form.viewOpenAIPrices": "View OpenAI pricing",
    "settings.api.form.viewPolygonPlans": "View Polygon plans",
    "settings.api.form.cancel": "Cancel",
    "settings.api.form.save": "Save",
    "settings.api.form.testInfo": "Testing API key...",
    "settings.api.form.testSuccess": "API key tested successfully!",
    "settings.api.form.enterKeyError": "Please enter an API key first.",
    "settings.api.form.providerCustom": "Custom",
    "settings.api.form.providers.openai": "OpenAI",
    "settings.api.form.providers.google_gemini": "Google Gemini",
    "settings.api.form.providers.anthropic": "Anthropic Claude",
    "settings.api.form.providers.polygon": "Polygon.io",
    "settings.api.form.providers.alpha_vantage": "Alpha Vantage",
}

# License Management Keys
license_pt = {
    "settings.license.management.title": "Gerenciamento de Licenças",
    "settings.license.management.subtitle": "Gere e gerencie licenças baseadas em identidade para proteger sua venda.",
    "settings.license.management.benefits.local_first.title": "Local First",
    "settings.license.management.benefits.local_first.desc": "Validação 100% offline via criptografia Ed25519.",
    "settings.license.management.benefits.identity_lock.title": "Travamento por Identidade",
    "settings.license.management.benefits.identity_lock.desc": "Amarrado ao Nome e CPF. Impossível compartilhar.",
    "settings.license.management.benefits.hardware_lock.title": "Amarrado ao Dispositivo e CPF",
    "settings.license.management.benefits.hardware_lock.desc": "Anti-pirataria extremo baseado na máquina e documentos.",
    "settings.license.management.steps.1.title": "Coletar Dados",
    "settings.license.management.steps.1.desc": "O cliente informa Nome, CPF e Data de Nascimento no onboard.",
    "settings.license.management.steps.2.title": "Gerar Licença",
    "settings.license.management.steps.2.desc": "Você usa a chave privada para gerar a licença oficial.",
    "settings.license.management.steps.3.title": "Validar Offline",
    "settings.license.management.steps.3.desc": "O app valida a assinatura contra a chave pública embutida sem internet.",
    "settings.license.management.new_license.button": "Cadastrar Nova Licença",
    "settings.license.management.new_license.modalTitle": "Criar Nova Licença",
    "settings.license.management.new_license.modalDesc": "Insira os dados do cliente para gerar uma nova chave.",
    "settings.license.management.new_license.name": "Nome Completo",
    "settings.license.management.new_license.cpf": "CPF (Sem pontuação)",
    "settings.license.management.new_license.birthDate": "Data de Nascimento",
    "settings.license.management.new_license.customerEmail": "Email do Cliente (Opcional)",
    "settings.license.management.table.client": "Cliente",
    "settings.license.management.table.key": "Chave de Licença",
    "settings.license.management.table.status": "Status",
    "settings.license.management.table.expires": "Expira em",
    "settings.license.management.table.actions": "Ações",
    "settings.license.management.table.empty": "Nenhuma licença cadastrada.",
    "settings.license.management.status.active": "Ativa",
    "settings.license.management.status.expired": "Expirada",
    "settings.license.management.status.revoked": "Revogada",
    "settings.license.management.actions.copy": "Copiar Chave",
    "settings.license.management.actions.revoke": "Revogar",
    "settings.license.management.actions.delete": "Excluir",
    "settings.license.management.delete.title": "Excluir Licença",
    "settings.license.management.delete.desc": "Tem certeza que deseja excluir permanentemente esta licença do banco de dados? O cliente perderá acesso na próxima validação online (se houver).",
    "settings.license.management.toasts.created": "Licença criada com sucesso!",
    "settings.license.management.toasts.deleted": "Licença excluída com sucesso!",
    "settings.license.management.toasts.copied": "Licença copiada para a área de transferência!",
    "settings.license.management.toasts.revoked": "Licença revogada com sucesso."
}

license_en = {
    "settings.license.management.title": "License Management",
    "settings.license.management.subtitle": "Generate and manage identity-based licenses to protect your sales.",
    "settings.license.management.benefits.local_first.title": "Local First",
    "settings.license.management.benefits.local_first.desc": "100% offline validation via Ed25519 cryptography.",
    "settings.license.management.benefits.identity_lock.title": "Identity Lock",
    "settings.license.management.benefits.identity_lock.desc": "Bound to Name and ID. Impossible to share.",
    "settings.license.management.benefits.hardware_lock.title": "Tied to Device and ID",
    "settings.license.management.benefits.hardware_lock.desc": "Extreme anti-piracy based on machine and documents.",
    "settings.license.management.steps.1.title": "Collect Data",
    "settings.license.management.steps.1.desc": "The client provides Name, ID, and Birth Date in onboarding.",
    "settings.license.management.steps.2.title": "Generate License",
    "settings.license.management.steps.2.desc": "You use the private key to generate the official license.",
    "settings.license.management.steps.3.title": "Validate Offline",
    "settings.license.management.steps.3.desc": "The app validates the signature against the built-in public key without internet.",
    "settings.license.management.new_license.button": "Register New License",
    "settings.license.management.new_license.modalTitle": "Create New License",
    "settings.license.management.new_license.modalDesc": "Enter customer data to generate a new key.",
    "settings.license.management.new_license.name": "Full Name",
    "settings.license.management.new_license.cpf": "National ID (CPF)",
    "settings.license.management.new_license.birthDate": "Birth Date",
    "settings.license.management.new_license.customerEmail": "Customer Email (Optional)",
    "settings.license.management.table.client": "Client",
    "settings.license.management.table.key": "License Key",
    "settings.license.management.table.status": "Status",
    "settings.license.management.table.expires": "Expires On",
    "settings.license.management.table.actions": "Actions",
    "settings.license.management.table.empty": "No licenses registered.",
    "settings.license.management.status.active": "Active",
    "settings.license.management.status.expired": "Expired",
    "settings.license.management.status.revoked": "Revoked",
    "settings.license.management.actions.copy": "Copy Key",
    "settings.license.management.actions.revoke": "Revoke",
    "settings.license.management.actions.delete": "Delete",
    "settings.license.management.delete.title": "Delete License",
    "settings.license.management.delete.desc": "Are you sure you want to permanently delete this license from the database? The client will lose access on the next online validation.",
    "settings.license.management.toasts.created": "License successfully created!",
    "settings.license.management.toasts.deleted": "License successfully deleted!",
    "settings.license.management.toasts.copied": "License copied to clipboard!",
    "settings.license.management.toasts.revoked": "License successfully revoked."
}

def inject(lang, dict_pt, dict_en):
    file_path = os.path.join(locales_dir, f"{lang}.json")
    if not os.path.exists(file_path):
        return
    with open(file_path, 'r', encoding='utf-8') as f:
        data = json.load(f)
    
    source = dict_pt if lang == 'pt-BR' else dict_en
    for k, v in source.items():
        set_nested(data, k, v)
        
    with open(file_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=4, ensure_ascii=False)
        print(f"Injected into {lang}")

inject('pt-BR', api_form_pt, api_form_en)
inject('en-US', api_form_pt, api_form_en)
inject('es-ES', api_form_pt, api_form_en)
inject('fr-FR', api_form_pt, api_form_en)

inject('pt-BR', license_pt, license_en)
inject('en-US', license_pt, license_en)
inject('es-ES', license_pt, license_en)
inject('fr-FR', license_pt, license_en)

print("Done injecting locs.")
