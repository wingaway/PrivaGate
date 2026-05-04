# Evaluation Plan

## Goals

Evaluation should measure both privacy effectiveness and task utility. A run is useful only when it records the policy, inputs, external-visible view, reports, and reproducible commands.

## Data

Use synthetic data only. Synthetic cases should mimic realistic shapes without containing real people, customers, patients, accounts, contracts, devices, logs, or secrets.

Required coverage:

- structured JSON;
- free text;
- multi-table relations;
- RAG chunks;
- security logs;
- Chinese and English identifier formats;
- direct identifiers, quasi-identifiers, and synthetic secrets.

## Privacy Metrics

- Raw-value residue in `external_view`.
- Raw-value residue in external model output.
- Token mapping isolation.
- Output inspection pass rate.
- Differential-privacy budget and error bounds.
- Known residual leakage such as equality and frequency leakage.

## Utility Metrics

- Required field preservation.
- Relation preservation.
- Foreign-key validity.
- Event-order validity.
- Task label preservation.
- Utility term hit rate for synthetic simulation.
- Statistical error bounds for DP outputs.

## Complex Dataset Checks

Complex datasets must verify:

- stable entity replacement across tables;
- preserved relations;
- preserved event order;
- bounded numeric transformations;
- no raw identifiers in derived fields;
- no prompt or tool output bypass.

## Quality Gates

Before release:

```bash
./scripts/cargo.sh fmt --all -- --check
./scripts/cargo.sh test
./scripts/cargo.sh clippy --all-targets -- -D warnings
./scripts/prepublish-check.sh
```

External API simulation should pass all synthetic cases before being used as release evidence.
