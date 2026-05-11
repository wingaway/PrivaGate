# RFC Process

RFCs are used for changes that affect PrivaGate's privacy boundary, public policy format, report semantics, external model adapters, or benchmark methodology.

Use an RFC when a change:

- Introduces a new redaction, generalization, suppression, tokenization, or differential-privacy mechanism.
- Adds an external model adapter or changes what data an adapter can receive.
- Changes policy schema, report schema, audit semantics, mapping semantics, or restoration semantics.
- Adds a benchmark that will be used to compare project quality.
- Makes a new privacy or utility claim.

## Steps

1. Open an issue using the Research Question template.
2. Copy `docs/rfcs/0000-template.md` into `docs/rfcs/NNNN-title.md`.
3. Fill in motivation, threat model, design, verification method, and compatibility notes.
4. Discuss counterexamples and failure modes before implementation.
5. After maintainer agreement, submit implementation PRs in small parts.

## Required Evidence

An RFC should describe:

- privacy claim;
- utility claim;
- threat assumptions;
- data that remains local;
- data that may become externally visible;
- verification method;
- tests or benchmark cases;
- known limitations.
