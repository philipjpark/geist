"use client";

import { useState } from "react";
import { useAccount, useConnect, useDisconnect, useSendTransactionSync } from "wagmi";
import { encodeFunctionData } from "viem";
import type { MonadProof as MonadProofData } from "@/lib/ir";
import { EXECUTION_GRAPH_REGISTRY_ABI, getRegistryAddress } from "@/lib/contract";

export function MonadProof({ proof }: { proof: MonadProofData | null }) {
  const { address, isConnected } = useAccount();
  const { connect, connectors, isPending: connecting } = useConnect();
  const { disconnect } = useDisconnect();
  const { sendTransactionSyncAsync, isPending } = useSendTransactionSync();
  const [txHash, setTxHash] = useState<string | null>(proof?.tx_hash ?? null);
  const [blockNumber, setBlockNumber] = useState<bigint | null>(null);
  const [error, setError] = useState<string | null>(null);

  const registryAddress = getRegistryAddress();

  async function registerOnMonad() {
    if (!proof) return;
    setError(null);

    if (!registryAddress) {
      setError("Set NEXT_PUBLIC_EXECUTION_GRAPH_REGISTRY_ADDRESS in .env");
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

      const receipt = await sendTransactionSyncAsync({ to: registryAddress, data });
      setTxHash(receipt.transactionHash);
      setBlockNumber(receipt.blockNumber);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Transaction failed");
    }
  }

  if (!proof) {
    return (
      <section className="rounded-lg border border-dashed border-rh-border bg-rh-surface p-4">
        <span className="font-mono text-[10px] uppercase tracking-widest text-rh-muted">Monad Proof</span>
      </section>
    );
  }

  return (
    <section className="rounded-lg border border-rh-border bg-rh-surface p-4">
      <div className="flex items-center justify-between gap-3">
        <span className="font-mono text-[10px] font-bold uppercase tracking-widest text-rh-muted">
          Monad Proof
        </span>
        {!isConnected ? (
          <button
            type="button"
            onClick={() => connect({ connector: connectors[0] })}
            disabled={connecting}
            className="rounded-full border border-rh-green px-3 py-1 text-xs font-semibold text-rh-green transition hover:bg-rh-green hover:text-rh-on-green"
          >
            Connect
          </button>
        ) : (
          <button
            type="button"
            onClick={() => disconnect()}
            className="rounded-full border border-rh-border px-3 py-1 font-mono text-[10px] text-rh-muted hover:text-rh-ink"
          >
            {address?.slice(0, 6)}…{address?.slice(-4)}
          </button>
        )}
      </div>

      <div className="mt-3 space-y-1.5 rounded-lg bg-rh-canvas p-3 font-mono text-[10px]">
        <Row label="signalHash" value={proof.signal_hash} accent />
        <Row label="graphHash" value={proof.graph_hash} accent />
        <Row label="score" value={`${proof.score}/100`} />
        <Row label="metadata" value={proof.metadata_uri} />
      </div>

      <button
        type="button"
        onClick={registerOnMonad}
        disabled={!isConnected || isPending}
        className="mt-3 w-full rounded-full bg-rh-green py-2.5 text-sm font-bold text-rh-on-green transition hover:bg-rh-green-hover disabled:opacity-50"
      >
        {isPending ? "Registering…" : "Register on Monad"}
      </button>

      {txHash && (
        <p className="mt-2 break-all font-mono text-[10px] text-rh-green-bright">
          tx {txHash}
          {blockNumber != null ? ` · blk ${blockNumber.toString()}` : ""}
        </p>
      )}
      {error && <p className="mt-2 font-mono text-[10px] text-rh-danger">{error}</p>}
    </section>
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
