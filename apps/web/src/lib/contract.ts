export const EXECUTION_GRAPH_REGISTRY_ABI = [
  {
    type: "function",
    name: "registerGraph",
    stateMutability: "nonpayable",
    inputs: [
      { name: "signalHash", type: "bytes32" },
      { name: "graphHash", type: "bytes32" },
      { name: "score", type: "uint256" },
      { name: "metadataURI", type: "string" },
    ],
    outputs: [{ name: "id", type: "uint256" }],
  },
  {
    type: "function",
    name: "recordCount",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint256" }],
  },
  {
    type: "event",
    name: "GraphRegistered",
    inputs: [
      { name: "creator", type: "address", indexed: true },
      { name: "signalHash", type: "bytes32", indexed: true },
      { name: "graphHash", type: "bytes32", indexed: true },
      { name: "score", type: "uint256", indexed: false },
      { name: "metadataURI", type: "string", indexed: false },
      { name: "timestamp", type: "uint256", indexed: false },
    ],
  },
] as const;

export function getRegistryAddress(): `0x${string}` | undefined {
  const addr = process.env.NEXT_PUBLIC_EXECUTION_GRAPH_REGISTRY_ADDRESS;
  if (!addr || addr.length < 10) return undefined;
  return addr as `0x${string}`;
}
