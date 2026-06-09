/** Monad testnet coordination rail — explorer + registry constants */

export const MONAD_CHAIN_ID = Number(process.env.NEXT_PUBLIC_CHAIN_ID || 10143);

export const MONAD_EXPLORER = "https://testnet.monadvision.com";

export const MONAD_COORDINATION_WALLET =
  process.env.NEXT_PUBLIC_MONAD_COORDINATION_WALLET ||
  "0x0AaA246300e261c6801b8c62397090Deb47310BA";

export function monadTxUrl(txHash: string): string {
  return `${MONAD_EXPLORER}/tx/${txHash}`;
}

export function monadAddressUrl(
  address: string,
  tab: "Transaction" | "Contract" = "Transaction"
): string {
  return `${MONAD_EXPLORER}/address/${address}?tab=${tab}`;
}

export function monadRegistryUrl(registry?: string): string | undefined {
  if (!registry) return undefined;
  return monadAddressUrl(registry, "Contract");
}

export const MONAD_WALLET_TX_URL = monadAddressUrl(MONAD_COORDINATION_WALLET, "Transaction");
