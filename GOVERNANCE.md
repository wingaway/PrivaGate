# Governance

ProofGate uses a lightweight maintainer model until the contributor base grows.

## Roles

- **Maintainers** review changes, manage releases, triage issues, and protect the project boundary.
- **Contributors** propose issues, submit pull requests, improve docs, add tests, and share reproducible evaluations.
- **Reviewers** may be invited by maintainers for domain-specific review, especially privacy, security, deployment, or language coverage.

## Decision Rules

Routine changes may be merged after one maintainer approval and passing CI. Routine changes include documentation fixes, synthetic test cases, detector improvements with tests, and small implementation fixes.

The following changes require an RFC or design issue before implementation:

- New redaction mechanisms or privacy claims.
- New external model adapters.
- Policy format changes.
- Audit, mapping, or restoration semantics.
- Benchmark methodology changes.
- Any change that broadens what can be sent to an external model.

Maintainers should prefer small, reversible changes. If reviewers disagree, the decision should be resolved by written evidence: threat model, tests, counterexamples, and reproducible evaluation.

## Merge Criteria

Pull requests should satisfy:

- CI passes.
- No real data, secrets, or generated artifacts are committed.
- Privacy and utility impact is described.
- New behavior has focused tests or an explicit test gap.
- Documentation is updated when public behavior changes.

## Release Criteria

Releases should include:

- A concise change summary.
- Compatibility notes for policies, APIs, and reports.
- Test and prepublish-check status.
- Known limitations and unresolved risks.
