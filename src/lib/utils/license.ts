export interface LicenseData {
    valid: boolean;
    plan: "Trial" | "Pro" | "Lifetime";
    expiration: string | null; // ISO Date
    createdAt?: string | null;  // ISO Date
    error?: string;
}

const SECRET_KEY = "TRADERLOGPRO_SECRET_KEY_2026"; // In prod, encrypt or obfuscate this!

// Web Crypto API Helper to Verify Signature
async function verifySignature(data: string, signatureHex: string): Promise<boolean> {
    const encoder = new TextEncoder();
    const keyData = encoder.encode(SECRET_KEY);
    const messageData = encoder.encode(data);

    const key = await crypto.subtle.importKey(
        "raw",
        keyData,
        { name: "HMAC", hash: "SHA-256" },
        false,
        ["verify"]
    );

    // Convert hex signature to Uint8Array
    const signatureBytes = new Uint8Array(signatureHex.match(/.{1,2}/g)?.map(byte => parseInt(byte, 16)) || []);

    return await crypto.subtle.verify(
        "HMAC",
        key,
        signatureBytes,
        messageData
    );
}

export async function computeCustomerId(data: { name: string, cpf: string, birthDate: string, hardwareId?: string }): Promise<string> {
    if (!data.name || !data.cpf || !data.birthDate) return "IDENTIDADE_NAO_PREENCHIDA";

    // Normalize data (lowercase name, numbers-only CPF)
    const normalizedName = data.name.trim().toLowerCase();
    const normalizedCpf = data.cpf.replace(/\D/g, '');
    const normalizedDob = data.birthDate.trim();
    const hwid = data.hardwareId || "NO_HWID";

    const raw = `TLP-ID-${normalizedName}-${normalizedCpf}-${normalizedDob}-${hwid}`;
    const msgUint8 = new TextEncoder().encode(raw);
    const hashBuffer = await crypto.subtle.digest('SHA-256', msgUint8);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('').toUpperCase();

    // Return a readable 12-char code
    return hashHex.slice(0, 12);
}

export async function validateLicenseKey(key: string, expectedCustomerId?: string): Promise<LicenseData> {
    try {
        // Sanitize: remove BOM and trim
        const sanitizedKey = key.replace(/^\uFEFF/, '').trim();

        // Format: TLP-v1-PAYLOADBASE64-SIGNATUREHEX
        const parts = sanitizedKey.split("-");
        if (parts.length !== 4 || parts[0] !== "TLP" || parts[1] !== "v1") {
            return { valid: false, plan: "Trial", expiration: null, error: "Formato de chave inválido." };
        }

        const [prefix, version, payloadB64, signature] = parts;
        const dataToVerify = `${prefix}-${version}-${payloadB64}`;

        // 1. Verify Signature
        const isValid = await verifySignature(dataToVerify, signature);
        if (!isValid) {
            return { valid: false, plan: "Trial", expiration: null, error: "Assinatura da chave inválida (Chave adulterada)." };
        }

        // 2. Decode Payload
        const payloadStr = atob(payloadB64);
        const payload = JSON.parse(payloadStr);
        console.log("[License] Decoded Payload:", payload);
        // 3. Check Customer ID (Identity Linkage)
        if (payload.cid && expectedCustomerId) {
            if (payload.cid !== expectedCustomerId) {
                return {
                    valid: false,
                    plan: payload.plan || "Pro",
                    expiration: payload.exp || null,
                    error: "Licença vinculada a outra identidade (ID incorreto)."
                };
            }
        } else if (payload.cid && !expectedCustomerId) {
            return {
                valid: false,
                plan: payload.plan || "Pro",
                expiration: payload.exp || null,
                error: "Esta licença exige identificação do usuário."
            };
        }

        // 4. Check Expiration
        if (payload.exp) {
            const expDate = new Date(payload.exp); // Timestamp or ISO
            if (expDate.getTime() < Date.now()) {
                return {
                    valid: false,
                    plan: payload.plan || "Pro",
                    expiration: expDate.toISOString(),
                    error: "Licença expirada."
                };
            }
            return {
                valid: true,
                plan: (payload.plan as any) || "Pro",
                expiration: expDate.toISOString(),
                createdAt: payload.created_at || null
            };
        }

        // Lifetime (no exp)
        return {
            valid: true,
            plan: (payload.plan as any) || "Lifetime",
            expiration: null,
            createdAt: payload.created_at || null
        };

    } catch (e) {
        console.error("License Validation Error:", e);
        return { valid: false, plan: "Trial", expiration: null, error: "Erro ao validar chave." };
    }
}
