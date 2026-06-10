<p align="center">
  <img src="geist_logo1.png" alt="Geist logo" width="200" />
</p>

<h1 align="center">Geist</h1>

<p align="center"><strong>Structure intent before execution.</strong></p>

Geist is a semantic execution compiler for fragmented markets — especially OTC and social-driven discovery. It ingests corpus signals, structures them into typed Semantic IR, scores execution risk, and commits a verifiable proof to Monad. It is **not** a broker, exchange, or trading bot.

## What it does

```text
Discovery (Reddit · Trends · News RSS · X)
  → select ticker
  → Parse → Semantic IR
  → Risk Engine
  → Coordination Rail (Monad proof)
```

| Stage | Purpose |
|-------|---------|
| **Discovery** | Ingest social and news feeds; surface tickers from corpus signals |
| **Semantic IR** | Convert raw intent into typed, structured meaning |
| **Risk Engine** | Deterministic scores for liquidity, spread, contention, route confidence |
| **Coordination Rail** | Register `signalHash`, `graphHash`, risk score, and metadata on Monad testnet |

The Monad transaction does **not** execute a trade. It anchors a public receipt that a specific compile occurred — hashes of the structured intent and execution plan, plus the risk score, signed by your wallet.

## Project structure

```text
geist/
├── apps/web/              # Next.js frontend
├── services/api/          # Rust / Axum API
├── contracts/             # ExecutionGraphRegistry (Solidity)
├── scripts/               # PyTrends helper for Google Trends
├── .env.example           # Environment template (copy to .env)
└── geist_logo1.png
```

## Prerequisites

- [Rust](https://rustup.rs/) (for the API)
- [Node.js](https://nodejs.org/) 18+ (for the web app)
- [Python](https://www.python.org/) 3.10+ (optional, for live Google Trends)
- MetaMask or another injected wallet (for Monad testnet commits)
- Testnet MON ([Monad faucet](https://faucet.monad.xyz))

## Setup

### 1. Environment

```bash
cp .env.example .env
```

Edit `.env` with your keys. **Never commit `.env`.**

| Variable | Required | Notes |
|----------|----------|-------|
| `OPENAI_API_KEY` | Optional | LLM correlate on discovery (off by default unless key is set) |
| `X_API_KEY` / `X_API_SECRET` / `X_ACCESS_TOKEN` / `X_ACCESS_TOKEN_SECRET` | Optional | X home timeline (5 posts) |
| `MASSIVE_API_KEY` | Optional | Live OTC quote data; falls back to mock |
| `NEXT_PUBLIC_EXECUTION_GRAPH_REGISTRY_ADDRESS` | For commits | Deployed registry contract on Monad testnet |
| `PRIVATE_KEY` | Deploy only | Server-side contract deploy — never expose to the browser |

`NEXT_PUBLIC_*` variables are exposed to the browser by design (RPC URL, chain ID, contract address).

### 2. Python (Google Trends)

```bash
pip install -r scripts/requirements.txt
```

### 3. Rust API

```powershell
# Windows — stop a stale process if port 8080 is locked
Stop-Process -Name geist-api -Force -ErrorAction SilentlyContinue

cd services/api
cargo run
```

API: `http://localhost:8080`

### 4. Frontend

```bash
cd apps/web
npm install
npm run dev
```

App: `http://localhost:3000`

## Using the app

1. **Scan markets** — select corpus sources (Reddit, Trends, News RSS, X).
2. **Pick a ticker** from any feed column.
3. Wait for **Semantic IR** and **Risk Engine** to compile.
4. In **Coordination Rail**, connect your wallet on **Monad Testnet** (chain `10143`).
5. Click **Commit execution proof → Monad** and confirm in your wallet.
6. Open the MonadVision link to view the transaction.

View your wallet history: [MonadVision testnet](https://testnet.monadvision.com)

## API endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Health check |
| `POST` | `/discover` | Corpus scan — `{ "seed": "", "sources": ["reddit","google_trends","news","x"] }` |
| `POST` | `/parse` | Text → Semantic IR |
| `POST` | `/analyze` | Semantic IR → risk scores + Monad proof payload |

Example:

```bash
curl http://localhost:8080/health

curl -X POST http://localhost:8080/discover \
  -H "Content-Type: application/json" \
  -d '{"seed":"","sources":["reddit","google_trends"]}'
```

## Smart contract

`ExecutionGraphRegistry` stores each commit as:

- `signalHash` — SHA-256 of Semantic IR
- `graphHash` — SHA-256 of execution DAG
- `score` — composite risk (0–100)
- `metadataURI` — Geist pointer (e.g. `geist://otc/intent/TICKER`)
- `creator` — wallet that signed the transaction
- `timestamp` — block time

Build:

```bash
cd contracts
forge build
```

Deploy to Monad testnet:

```bash
forge script script/Deploy.s.sol:Deploy \
  --rpc-url https://testnet-rpc.monad.xyz \
  --broadcast
```

Set the deployed address in `.env` as `NEXT_PUBLIC_EXECUTION_GRAPH_REGISTRY_ADDRESS`.

## Deploy to Netlify

Netlify only hosts the **Next.js frontend** (`apps/web`). The Rust API must run elsewhere (Railway, Fly.io, Render, etc.) if you want live discovery and compile — otherwise the UI falls back to mock data when the API is unreachable.

### 1. Connect the repo

1. [Netlify](https://app.netlify.com) → **Add new site** → **Import from Git** → select `main`.
2. Netlify reads `netlify.toml` at the repo root — **base directory** is already set to `apps/web`. Do not override it unless you know what you’re changing.

### 2. Set environment variables

In **Site configuration → Environment variables**, add (values from your local `.env`):

| Variable | Example |
|----------|---------|
| `NEXT_PUBLIC_API_URL` | `https://your-api-host.example.com` (omit or leave unset for mock-only demo) |
| `NEXT_PUBLIC_EXECUTION_GRAPH_REGISTRY_ADDRESS` | `0x4464Ad7D8145788D3061645E2f14e36aFeF542f3` |
| `NEXT_PUBLIC_CHAIN_ID` | `10143` |
| `NEXT_PUBLIC_MONAD_TESTNET_RPC` | `https://testnet-rpc.monad.xyz` |
| `NEXT_PUBLIC_MONAD_COORDINATION_WALLET` | your coordination wallet address |

Never put server secrets (`OPENAI_API_KEY`, `PRIVATE_KEY`, `X_BEARER_TOKEN`, etc.) in Netlify — those belong on the API host only.

### 3. Deploy

Push to `main` (or trigger **Deploy site**). A successful build log ends with `next build` completing and the Next.js runtime plugin publishing the app.

### Common failures

| Symptom | Fix |
|---------|-----|
| Build fails: “No package.json” | Base directory must be `apps/web` (use repo `netlify.toml` or set in Netlify UI). |
| Site loads but discovery/compile feel “fake” | `NEXT_PUBLIC_API_URL` points at `localhost:8080` or the API isn’t deployed — deploy `services/api` and set the public URL. |
| Monad commit button disabled | Set `NEXT_PUBLIC_EXECUTION_GRAPH_REGISTRY_ADDRESS` in Netlify env vars and redeploy. |

## Security

- Keep secrets in `.env` only — it is gitignored.
- Rotate any API keys or `PRIVATE_KEY` if they were ever shared or committed.
- Do not put secrets in `NEXT_PUBLIC_*` variables.

## License

MIT (choose your license)
