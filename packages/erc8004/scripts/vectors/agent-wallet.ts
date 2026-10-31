import { hashTypedData } from "viem";
import type { Address } from "viem";

const IDENTITY: Address = "0x8004A169FB4a3325136EB29fA0ceB6D2e539a432";
const NEW_WALLET: Address = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed";
const OWNER: Address = "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359";

const TYPES = {
  AgentWalletSet: [
    { name: "agentId", type: "uint256" },
    { name: "newWallet", type: "address" },
    { name: "owner", type: "address" },
    { name: "deadline", type: "uint256" },
  ],
} as const;

function hash(
  chainId: number,
  message: { agentId: bigint; newWallet: Address; owner: Address; deadline: bigint },
): string {
  return hashTypedData({
    domain: { name: "ERC8004IdentityRegistry", version: "1", chainId, verifyingContract: IDENTITY },
    types: TYPES,
    primaryType: "AgentWalletSet",
    message: {
      agentId: message.agentId,
      newWallet: message.newWallet,
      owner: message.owner,
      deadline: message.deadline,
    },
  });
}

export function build(): { capability: string; cases: unknown[] } {
  const cases: Array<Record<string, unknown>> = [];
  const push = (
    chainId: number,
    agentId: bigint,
    deadline: bigint,
    opts: { newWallet?: Address; owner?: Address } = {},
  ) => {
    const message = {
      agentId: agentId.toString(),
      newWallet: opts.newWallet ?? NEW_WALLET,
      owner: opts.owner ?? OWNER,
      deadline: deadline.toString(),
    };
    cases.push({
      op: "hash",
      input: { chainId, identityRegistry: IDENTITY, message },
      output: hash(chainId, {
        agentId,
        newWallet: message.newWallet,
        owner: message.owner,
        deadline,
      }),
    });
  };

  push(1, 1n, 1735689600n);
  push(84532, 42n, 1735689600n);
  push(1, 0n, 0n);
  push(
    1,
    115792089237316195423570985008687907853269984665640564039457584007913129639935n,
    18446744073709551615n,
  );
  push(1, 7n, 999999n, { newWallet: "0x0000000000000000000000000000000000000000" });

  const deadlineCases: Array<[string, string, string | undefined]> = [
    ["100", "100", undefined],
    ["100", "99", undefined],
    ["100", "101", "DeadlineExpired"],
    ["400", "100", undefined],
    ["402", "101", "DeadlineTooFar"],
    ["401", "100", "DeadlineTooFar"],
    ["0", "0", undefined],
    ["0", "1", "DeadlineExpired"],
    ["18446744073709551615", "18446744073709551615", undefined],
  ];
  for (const [deadline, now, error] of deadlineCases) {
    cases.push({
      op: "deadline",
      input: { deadline, now },
      ...(error === undefined ? { output: null } : { error }),
    });
  }

  return { capability: "identity.agent-wallet", cases };
}
