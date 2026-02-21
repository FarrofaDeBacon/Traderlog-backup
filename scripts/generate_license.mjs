import crypto from 'node:crypto';
import fs from 'node:fs';

// CONFIG: Must match src/lib/utils/license.ts
const SECRET_KEY = "TRADERLOGPRO_SECRET_KEY_2026";

function generateKey(daysRaw, plan = "Pro", cid = null) {
    let payload = {
        plan: plan,
        exp: null,
        cid: cid,
        created_at: new Date().toISOString()
    };

    if (daysRaw !== "lifetime") {
        const days = parseInt(daysRaw);
        if (isNaN(days) || days <= 0) {
            console.error("Invalid days. Use a positive number or 'lifetime'.");
            process.exit(1);
        }
        const expDate = new Date();
        expDate.setDate(expDate.getDate() + days);
        payload.exp = expDate.toISOString();
    }

    // 1. Prepare Data
    const payloadStr = JSON.stringify(payload);
    const payloadB64 = Buffer.from(payloadStr).toString('base64');
    const version = "v1";
    const prefix = "TLP";

    // TLP-v1-PAYLOADBASE64
    const dataToSign = `${prefix}-${version}-${payloadB64}`;

    // 2. Sign
    const hmac = crypto.createHmac('sha256', SECRET_KEY);
    hmac.update(dataToSign);
    const signatureHex = hmac.digest('hex');

    // 3. Assemble Key
    const licenseKey = `${dataToSign}-${signatureHex}`; // Or just append signature
    // Match validator logic: TLP-v1-Payload-Sig
    // Wait, validator splits by "-".
    // dataToSign has 2 dashes "-". So total 3 dashes.
    // TLP-v1-B64-SIG

    return {
        key: `${prefix}-${version}-${payloadB64}-${signatureHex}`,
        details: payload
    };
}

// CLI Args
const args = process.argv.slice(2);
const days = args[0] || "7"; // Default 7 days
const plan = args[1] || "Pro";
const cid = args[2] || null;
const outFile = args[3] || null;

console.log(`\n--- TRADERLOGPRO LICENSE GENERATOR ---\n`);
console.log(`Config: ${days} days, Plan: ${plan}, CID: ${cid || "None"}, Output: ${outFile || "Console only"}`);

try {
    const result = generateKey(days, plan, cid);
    console.log(`\nLicense Details:`);
    console.log(`  Plan: ${result.details.plan}`);
    console.log(`  Expires: ${result.details.exp ? new Date(result.details.exp).toLocaleString() : "Never (Lifetime)"}`);
    console.log(`\nYour License Key:\n`);
    console.log(`\x1b[32m${result.key}\x1b[0m`); // Green color

    if (outFile) {
        fs.writeFileSync(outFile, result.key, 'utf8');
        console.log(`\n[SUCCESS] License saved to: ${outFile}`);
    }

    console.log(`\n--------------------------------------\n`);
} catch (e) {
    console.error("Error generating key:", e);
}
