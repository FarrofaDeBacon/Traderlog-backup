const dateStr = "2026-02-28T02:00:00.000Z";
const normalized = dateStr.includes(" ") ? dateStr.replace(" ", "T") : dateStr;
const d = new Date(normalized);

const year = d.getFullYear();
const month = String(d.getMonth() + 1).padStart(2, "0");
const day = String(d.getDate()).padStart(2, "0");

console.log(`${year}-${month}-${day}`);
