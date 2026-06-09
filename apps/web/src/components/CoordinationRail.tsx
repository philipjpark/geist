"use client";

import { useEffect, useState } from "react";
import {
  useAccount,
  useChainId,
  useConnect,
  useDisconnect,
  useSendTransactionSync,
  useSwitchChain,
} from "wagmi";
import { encodeFunctionData } from "viem";
import type { ExecutionDAG, MonadProof as MonadProofData } from "@/lib/ir";
import { EXECUTION_GRAPH_REGISTRY_ABI, getRegistryAddress } from "@/lib/contract";
import {
  MONAD_CHAIN_ID,
  MONAD_COORDINATION_WALLET,
  MONAD_WALLET_TX_URL,
  monadAddressUrl,
  monadRegistryUrl,
  monadTxUrl,
} from "@/lib/monad";
import { TermHint } from "@/components/TermHint";

export function CoordinationRail({
  proof,
  dag,
  compositeScore,
  ready,
}: {
  proof: MonadProofData | null;
  dag: ExecutionDAG | null;
  compositeScore?: number;
  ready: boolean;
}) {
  const { address, isConnected } = useAccount();
  const chainId = useChainId();
  const onMonad = chainId === MONAD_CHAIN_ID;
  const { connect, connectors, isPending: connecting } = useConnect();
  const { disconnect } = useDisconnect();
  const { switchChain, isPending: switching } = useSwitchChain();
  const { sendTransactionSyncAsync, isPending } = useSendTransactionSync();
  const [txHash, setTxHash] = useState<string | null>(null);
  const [blockNumber, setBlockNumber] = useState<bigint | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [spawned, setSpawned] = useState(false);

  const registryAddress = getRegistryAddress();
  const isCoordinationWallet =
    address?.toLowerCase() === MONAD_COORDINATION_WALLET.toLowerCase();

  useEffect(() => {
    if (ready && proof) {
      setSpawned(true);
    }
    if (!ready) {
      setSpawned(false);
      setTxHash(null);
      setBlockNumber(null);
      setError(null);
    }
  }, [ready, proof]);

  async function commitToRail() {
    if (!proof) return;
    setError(null);

    if (!registryAddress) {
      setError("Set NEXT_PUBLIC_EXECUTION_GRAPH_REGISTRY_ADDRESS in .env");
      return;
    }

    if (!onMonad) {
      setError("Switch your wallet to Monad Testnet before committing.");
      return;
    }

    try {
      const data = encodeFunctionData({
        abi: EXECUTION_GRAPH_REGISTRY_ABI,
        functionName: "registerGraph",
        args: [
          proof.signal_hash as `0x${string}`,
          proof.graph_hash as `0x${string}`,
          BigInt(proof.score),
          proof.metadata_uri,
        ],
      });

      const receipt = await sendTransactionSyncAsync({
        to: registryAddress,
        data,
        chainId: MONAD_CHAIN_ID,
      });
      setTxHash(receipt.transactionHash);
      setBlockNumber(receipt.blockNumber);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Coordination commit failed");
    }
  }

  if (!ready || !proof) {
    return (
      <section className="rounded-lg border border-dashed border-rh-border bg-rh-surface p-4">
        <RailHeader />
        <p className="mt-2 font-mono text-[11px] text-rh-muted">
          Awaiting compile — coordination rail activates after risk engine completes.
        </p>
      </section>
    );
  }

  const canCommit = isConnected && onMonad && !isPending && !txHash;

  return (
    <section
      className={`rounded-lg border bg-rh-surface p-4 transition-all duration-500 ${
        spawned ? "border-rh-green shadow-[0_0_0_1px_var(--rh-green)]" : "border-rh-border"
      }`}
    >
      <RailHeader spawned={spawned} />

      <details className="mt-3 rounded-lg border border-rh-border bg-rh-canvas" open>
        <summary className="cursor-pointer px-3 py-2 font-mono text-[10px] font-semibold uppercase tracking-widest text-rh-muted">
          What gets stored on Monad
        </summary>
        <div className="space-y-2 border-t border-rh-border px-3 py-3 font-mono text-[10px] leading-relaxed text-rh-muted">
          <p>
            Pressing <strong className="text-rh-ink">Commit execution proof → Monad</strong> sends
            one transaction to the{" "}
            <strong className="text-rh-ink">ExecutionGraphRegistry</strong> contract. It does not
            execute a trade — it anchors a proof that this compile happened.
          </p>
          <ul className="list-inside list-disc space-y-1 text-rh-ink/90">
            <li>
              <span className="text-rh-green-bright">signalHash</span> — SHA-256 of your Semantic
              IR (structured intent)
            </li>
            <li>
              <span className="text-rh-green-bright">graphHash</span> — SHA-256 of the compiled
              execution DAG
            </li>
            <li>
              <span className="text-rh-green-bright">score</span> — composite risk score (0–100)
              {compositeScore != null ? ` · this compile: ${compositeScore}` : ""}
            </li>
            <li>
              <span className="text-rh-green-bright">metadataURI</span> — compact Geist pointer (
              {proof.metadata_uri})
            </li>
            <li>
              <span className="text-rh-green-bright">creator</span> — your connected wallet (
              {address ? `${address.slice(0, 6)}…${address.slice(-4)}` : "connect below"})
            </li>
            <li>
              <span className="text-rh-green-bright">timestamp</span> — block time on Monad
              testnet
            </li>
          </ul>
          <p className="pt-1">
            Registry contract:{" "}
            <a
              href={registryAddress ? monadRegistryUrl(registryAddress) : undefined}
              target="_blank"
              rel="noopener noreferrer"
              className="text-rh-green-bright underline-offset-2 hover:underline"
            >
              {registryAddress ?? "not configured"}
            </a>
          </p>
          <p>
            Coordination wallet{" "}
            <a
              href={monadAddressUrl(MONAD_COORDINATION_WALLET)}
              target="_blank"
              rel="noopener noreferrer"
              className="text-rh-green-bright underline-offset-2 hover:underline"
            >
              {MONAD_COORDINATION_WALLET}
            </a>{" "}
            — Geist reference account on Monad testnet. Connect this address in your browser wallet
            to sign commits from that account; otherwise your connected wallet is recorded as{" "}
            <em>creator</em>.
          </p>
        </div>
      </details>

      <div className="mt-3 space-y-1.5 rounded-lg bg-rh-canvas p-3 font-mono text-[10px]">
        <Row label="signalHash" value={proof.signal_hash} accent />
        <Row label="graphHash" value={proof.graph_hash} accent />
        <Row label="verification score" value={`${proof.score}/100`} />
        <Row label="metadata" value={proof.metadata_uri} />
      </div>

      <div className="mt-3 flex flex-wrap items-center gap-2">
        {!isConnected ? (
          <button
            type="button"
            onClick={() => connect({ connector: connectors[0], chainId: MONAD_CHAIN_ID })}
            disabled={connecting}
            className="rounded-full border border-rh-green px-3 py-1 text-xs font-semibold text-rh-green transition hover:bg-rh-green hover:text-rh-on-green"
          >
            Connect wallet
          </button>
        ) : (
          <>
            <button
              type="button"
              onClick={() => disconnect()}
              className={`rounded-full border px-3 py-1 font-mono text-[10px] ${
                isCoordinationWallet
                  ? "border-rh-green text-rh-green"
                  : "border-rh-border text-rh-muted hover:text-rh-ink"
              }`}
            >
              {address?.slice(0, 6)}…{address?.slice(-4)}
              {isCoordinationWallet ? " · coordination wallet" : ""}
            </button>
            {!onMonad && (
              <button
                type="button"
                onClick={() => switchChain({ chainId: MONAD_CHAIN_ID })}
                disabled={switching}
                className="rounded-full border border-rh-warning px-3 py-1 font-mono text-[10px] text-rh-warning"
              >
                {switching ? "Switching…" : "Switch to Monad Testnet"}
              </button>
            )}
          </>
        )}
      </div>

      <button
        type="button"
        onClick={commitToRail}
        disabled={!canCommit}
        className="mt-3 w-full rounded-full bg-rh-green py-2.5 text-sm font-bold text-rh-on-green transition hover:bg-rh-green-hover disabled:opacity-50"
      >
        {isPending
          ? "Committing to coordination rail…"
          : txHash
            ? "Committed on Monad"
            : !isConnected
              ? "Connect wallet to commit"
              : !onMonad
                ? "Switch to Monad Testnet"
                : "Commit execution proof → Monad"}
      </button>

      {txHash && (
        <div className="mt-3 space-y-2 rounded-lg border border-rh-green/30 bg-rh-green/5 p-3">
          <div className="font-mono text-[9px] font-bold uppercase tracking-widest text-rh-green">
            Coordination commit confirmed
          </div>
          <p className="font-mono text-[10px] text-rh-muted">
            Stored on registry · creator {address?.slice(0, 6)}…{address?.slice(-4)}
          </p>
          <a
            href={monadTxUrl(txHash)}
            target="_blank"
            rel="noopener noreferrer"
            className="block break-all font-mono text-[11px] text-rh-green-bright underline-offset-2 hover:underline"
          >
            View transaction on MonadVision →
          </a>
          {blockNumber != null && (
            <p className="font-mono text-[10px] text-rh-muted">block {blockNumber.toString()}</p>
          )}
          {registryAddress && monadRegistryUrl(registryAddress) && (
            <a
              href={monadRegistryUrl(registryAddress)}
              target="_blank"
              rel="noopener noreferrer"
              className="block font-mono text-[10px] text-rh-muted underline-offset-2 hover:text-rh-green hover:underline"
            >
              View registry contract on MonadVision →
            </a>
          )}
        </div>
      )}

      {!txHash && (
        <a
          href={MONAD_WALLET_TX_URL}
          target="_blank"
          rel="noopener noreferrer"
          className="mt-3 block font-mono text-[10px] text-rh-muted underline-offset-2 hover:text-rh-green hover:underline"
        >
          View coordination wallet history ({MONAD_COORDINATION_WALLET.slice(0, 6)}…) →
        </a>
      )}

      {error && <p className="mt-2 font-mono text-[10px] text-rh-danger">{error}</p>}
    </section>
  );
}

function RailHeader({ spawned }: { spawned?: boolean }) {
  return (
    <div className="flex items-center justify-between gap-3">
      <div>
        <span className="font-mono text-[10px] font-bold uppercase tracking-widest text-rh-muted">
          <TermHint term="coordination_rail" />
        </span>
        <p className="mt-0.5 font-mono text-[9px] uppercase tracking-wider text-rh-muted/80">
          Monad proof substrate · not the product
        </p>
      </div>
      {spawned && (
        <span className="rounded-full bg-rh-green/15 px-2 py-0.5 font-mono text-[9px] font-bold uppercase tracking-wide text-rh-green">
          ready to commit
        </span>
      )}
    </div>
  );
}

function Row({ label, value, accent }: { label: string; value: string; accent?: boolean }) {
  return (
    <div className="flex justify-between gap-3">
      <span className="text-rh-muted">{label}</span>
      <span className={`truncate ${accent ? "text-rh-green-bright" : "text-rh-ink"}`}>{value}</span>
    </div>
  );
}
