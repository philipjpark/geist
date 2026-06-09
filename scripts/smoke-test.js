const API = process.env.API_URL || process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";

const cases = [
  { query: "AI compute demand surge", mode: "balanced", market_type: "cross_asset" },
  { query: "CYDY", mode: "conservative", market_type: "otc" },
];

for (const body of cases) {
  const res = await fetch(`${API}/analyze`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });

  if (!res.ok) {
    console.error(`Smoke test failed for ${body.query}: ${res.status}`);
    process.exit(1);
  }

  const json = await res.json();
  console.log(`\n--- ${body.query} (${body.market_type}) ---`);
  console.log(JSON.stringify(json, null, 2));
}

console.log("\nSmoke test passed.");
