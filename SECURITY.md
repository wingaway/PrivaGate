# Security Policy

ProofGate is designed to reduce the information sent from a local trust boundary to an external LLM API. It does not make external providers trustworthy, and it does not make unsafe data handling safe by itself.

Please report security issues privately to the repository maintainers. Do not open public issues for suspected leaks, bypasses, secret exposure, or real-data incidents.

Security-sensitive material that must not be committed:

- Real datasets, prompts, transcripts, model input/output traces, mapping logs, audit logs, or evaluation reports containing real data.
- API keys, bearer tokens, local model credentials, database URLs with passwords, HMAC keys, SSH keys, or cloud credentials.
- Production policies that reveal internal schemas, customer identifiers, account formats, device naming schemes, or operational secrets.

Use `scripts/prepublish-check.*` before release. The scan is heuristic and does not replace manual review.
