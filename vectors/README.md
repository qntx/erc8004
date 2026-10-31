# vectors/

Shared cross-language test vectors. The TypeScript tests
(`packages/erc8004/tests/vectors/`) are the reference run; `crates/erc8004`
runs the same files through its test suite. Both sides must match exactly:
shared error cases name a code (`InvalidChainId`, `BadChecksum`,
`TooManyDecimals`, ...) that equals the Rust error variant name and the
TypeScript error `code`.

## Layout

```text
vectors/<area>/<name>.json    one file per capability, name = capability id
                              without the "<area>." prefix, dots as hyphens
vectors/<area>/<name>.<ext>   official external vectors keep their original
                              format
```

`scripts/parity/check.ts` asserts that every file here is referenced by at
least one capability in `parity.json`.

## Schema

JSON files share a single envelope:

```json
{
  "schema": 1,
  "capability": "registry.parse",
  "source": { "kind": "generated", "generator": "@qntx/erc8004", "version": "0.5.0" },
  "cases": []
}
```

- `schema` — envelope version, currently always `1`.
- `capability` — the `parity.json` capability id this file exercises.
- `source.kind` — `official` (upstream vector, verbatim), `reference`
  (produced by an independent reference implementation; record its name and
  version in `source`), or `generated` (frozen output of the TS
  implementation; `generator` names the producing script/package and
  `version` the package version that produced it).
- `cases` — always an array. Success cases carry inputs and expected outputs;
  failure cases carry an `error` field naming the shared error code.
  Runners branch on the case fields; a file may mix case shapes.
