import { build as buildAgentWallet } from "./agent-wallet.ts";
import { build as buildEvents } from "./events.ts";
import { finish, vectorFile } from "./lib.ts";
import { build as buildRevert } from "./revert.ts";

const GENERATORS = [
  ["vectors/core/revert.json", buildRevert, "packages/erc8004/scripts/vectors/revert.ts"],
  [
    "vectors/identity/agent-wallet.json",
    buildAgentWallet,
    "packages/erc8004/scripts/vectors/agent-wallet.ts",
  ],
  ["vectors/events/decode.json", buildEvents, "packages/erc8004/scripts/vectors/events.ts"],
] as const;

for (const [path, build, generator] of GENERATORS) {
  const { capability, cases } = build();
  vectorFile(path, capability, cases, generator);
}
finish();
