## Summary

Describe the change in one or two paragraphs.

## Type

- [ ] Bug fix
- [ ] Documentation
- [ ] Detector
- [ ] Projection mechanism
- [ ] Verification or benchmark
- [ ] Model adapter
- [ ] Deployment
- [ ] Other

## Privacy and Utility

- Privacy claim or impact:
- Utility claim or impact:
- Data that remains local:
- Data that may become externally visible:

## Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo test`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `scripts/prepublish-check.*`
- [ ] Synthetic test data added or updated
- [ ] Documentation updated

## Safety

- [ ] This PR contains synthetic data only.
- [ ] This PR does not include API keys, bearer tokens, mapping logs, audit logs, model input/output traces with real data, or generated artifacts.
- [ ] External model adapters, if changed, receive only `external_view`.

## RFC

- [ ] Not required.
- [ ] Required and linked:
