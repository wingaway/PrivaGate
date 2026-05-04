# Complex Text Task Report: complex_en_medical_claim_appeal_text_001

> Public test-data notice: this report was generated from synthetic and simulated test data only. Names, organizations, identifiers, accounts, addresses, medical facts, original inputs, projected views, model I/O, and restored fields in this file are artificial fixtures for evaluating ProofGate. They must not be interpreted as real personal, customer, patient, operational, or business data. The external API was used only as an OpenAI-compatible test endpoint, and no API keys or Authorization headers are recorded here.

## Test Task

- Case ID: `complex_en_medical_claim_appeal_text_001`
- Task profile: `healthcare_claim_appeal_review`
- Created at: `2026-05-04T09:14:20.549788+00:00`
- Gateway: `http://127.0.0.1:8080`
- External model: `deepseek-v4-flash`

## Original Synthetic Input

```text
Synthetic claim appeal narrative: On April 18, 2026, Robert Smith, member MBR-7788-2211 under policy POL-ACME-4401, contacted the oncology claims unit by phone at +1-212-555-0171 and by email at robert.smith.synthetic@example.test regarding denied claim CLM-2026-4401 and medical record MR-US-2026-3001. The member reported that the service was ordered by Dr. Emily Carter at Northbridge Oncology Center after a follow-up visit for high-risk oncology monitoring. The original denial stated that prior authorization was incomplete, although the clinical note says the authorization packet was faxed on April 12, 2026.

The appeal letter lists the member's date of birth as 1979-11-24, home address as 742 Evergreen Terrace, Springfield, IL 62704, and includes an internal case note marked synthetic-sensitive-oncology-note. The disputed claim amount is bucketed at 38000 USD. The reviewer should preserve the business context: oncology, denied, appeal, high-risk monitoring, prior authorization, missing documentation, fax timeline, and recommended next action. The reviewer must not infer or restore member identity, contact information, policy identifiers, claim identifiers, medical record identifiers, physician identity, facility identity, street address, date of birth, or internal notes.
```

## Sensitive Values Expected To Be Removed

| Type | Synthetic value | Expected mechanism |
|---|---|---|
| member name | `Robert Smith` | `hmac_token` |
| member id | `MBR-7788-2211` | `hmac_token` |
| policy id | `POL-ACME-4401` | `hmac_token` |
| claim id | `CLM-2026-4401` | `hmac_token` |
| medical record id | `MR-US-2026-3001` | `hmac_token` |
| phone | `+1-212-555-0171` | `hmac_token` |
| email | `robert.smith.synthetic@example.test` | `hmac_token` |
| physician | `Dr. Emily Carter` | `hmac_token` |
| facility | `Northbridge Oncology Center` | `hmac_token` |
| date of birth | `1979-11-24` | `suppress` |
| street address | `742 Evergreen Terrace, Springfield, IL 62704` | `address_generalize` |
| internal note | `synthetic-sensitive-oncology-note` | `suppress` |

## Utility Facts Expected To Be Preserved

| Utility fact | Purpose |
|---|---|
| `oncology` | Required for healthcare claim appeal review |
| `denied` | Required for healthcare claim appeal review |
| `appeal` | Required for healthcare claim appeal review |
| `high-risk` | Required for healthcare claim appeal review |
| `prior authorization` | Required for healthcare claim appeal review |
| `incomplete` | Required for healthcare claim appeal review |
| `faxed` | Required for healthcare claim appeal review |
| `April 12, 2026` | Required for healthcare claim appeal review |
| `38000 USD` | Required for healthcare claim appeal review |
| `recommended next action` | Required for healthcare claim appeal review |

## ProofGate Projection

- Audit ID: `3cd34aa3-527b-43b5-9f42-a8d3c7179b52`
- Input digest: `sha256:0994ff9a156590977817178bd868bcb9f4c7575a73e68018d390dd3aeb7abb32`
- External view digest: `sha256:1f94d779da0e4bcd774738095f6856e837b68258a35066b3544d0f6cabdddcec`
- External-view raw sensitive leaks: `0`

### External View

```json
{
  "content_type": "text",
  "payload": "Synthetic claim appeal narrative: On April 18, 2026, <PERSON_08dcfc19269ec36a391c5b2c>, member <MEMBER_b716b76d94b29b38b438eda3> under policy <POLICY_8b70d0838e8162eb21ba1a1c>, contacted the oncology claims unit by phone at <PHONE_d93ad2f605f7092b53ce292d> and by email at <EMAIL_4a8a9930f8f98bea95fc77ab> regarding denied claim <CLAIM_0c460c8c59d21b27cd6461b4> and medical record <MEDICAL_RECORD_0ff157c98f781393745bf280>. The member reported that the service was ordered by <PERSON_dd2a014e7a821a5bed20cd1f> at <ORGANIZATION_6f2d0c1e11031ac928cc8ce6> after a follow-up visit for high-risk oncology monitoring. The original denial stated that prior authorization was incomplete, although the clinical note says the authorization packet was faxed on April 12, 2026.\n\nThe appeal letter lists the member's date of birth as [SUPPRESSED], home address as Springfield, IL, and includes an internal case note marked [SUPPRESSED]. The disputed claim amount is bucketed at 38000 USD. The reviewer should preserve the business context: oncology, denied, appeal, high-risk monitoring, prior authorization, missing documentation, fax timeline, and recommended next action. The reviewer must not infer or restore member identity, contact information, policy identifiers, claim identifiers, medical record identifiers, physician identity, facility identity, street address, date of birth, or internal notes."
}
```

### Privacy Report

```json
{
  "report_type": "privacy_proof",
  "report_id": "54725205-6652-4d74-a363-22aea27baaa2",
  "created_at": "2026-05-04T09:13:50.330680200Z",
  "policy_version": "2026.05.03-production-sample",
  "input_digest": "sha256:0994ff9a156590977817178bd868bcb9f4c7575a73e68018d390dd3aeb7abb32",
  "external_view_digest": "sha256:1f94d779da0e4bcd774738095f6856e837b68258a35066b3544d0f6cabdddcec",
  "mechanisms": [
    {
      "field_name": "access_token",
      "field_type": "secret",
      "mechanism": "suppress",
      "key_domain": null,
      "token_count": 1
    },
    {
      "field_name": "address",
      "field_type": "address",
      "mechanism": "address_generalize",
      "key_domain": null,
      "token_count": 1
    },
    {
      "field_name": "claim_id",
      "field_type": "claim",
      "mechanism": "hmac_token",
      "key_domain": "local-kms/proofgate-gateway/hmac/v1",
      "token_count": 1
    },
    {
      "field_name": "company_name",
      "field_type": "organization",
      "mechanism": "hmac_token",
      "key_domain": "local-kms/proofgate-gateway/hmac/v1",
      "token_count": 1
    },
    {
      "field_name": "contact_name",
      "field_type": "person",
      "mechanism": "hmac_token",
      "key_domain": "local-kms/proofgate-gateway/hmac/v1",
      "token_count": 1
    },
    {
      "field_name": "contact_name",
      "field_type": "person",
      "mechanism": "hmac_token",
      "key_domain": "local-kms/proofgate-gateway/hmac/v1",
      "token_count": 1
    },
    {
      "field_name": "date_of_birth",
      "field_type": "date_of_birth",
      "mechanism": "suppress",
      "key_domain": null,
      "token_count": 1
    },
    {
      "field_name": "email",
      "field_type": "email",
      "mechanism": "hmac_token",
      "key_domain": "local-kms/proofgate-gateway/hmac/v1",
      "token_count": 1
    },
    {
      "field_name": "medical_record_id",
      "field_type": "medical_record",
      "mechanism": "hmac_token",
      "key_domain": "local-kms/proofgate-gateway/hmac/v1",
      "token_count": 1
    },
    {
      "field_name": "member_id",
      "field_type": "member",
      "mechanism": "hmac_token",
      "key_domain": "local-kms/proofgate-gateway/hmac/v1",
      "token_count": 1
    },
    {
      "field_name": "phone",
      "field_type": "phone",
      "mechanism": "hmac_token",
      "key_domain": "local-kms/proofgate-gateway/hmac/v1",
      "token_count": 1
    },
    {
      "field_name": "policy_id",
      "field_type": "policy",
      "mechanism": "hmac_token",
      "key_domain": "local-kms/proofgate-gateway/hmac/v1",
      "token_count": 1
    }
  ],
  "privacy_budget": {
    "epsilon": 2.25,
    "delta": 0.0,
    "consumed": true
  },
  "declared_leakage": [
    "account:equality",
    "account:frequency",
    "account:type",
    "claim:equality",
    "claim:frequency",
    "claim:type",
    "contract:equality",
    "contract:frequency",
    "contract:type",
    "customer:equality",
    "customer:frequency",
    "customer:type",
    "device:equality",
    "device:frequency",
    "device:type",
    "employee:equality",
    "employee:frequency",
    "employee:type",
    "host:equality",
    "host:frequency",
    "host:type",
    "invoice:equality",
    "invoice:frequency",
    "invoice:type",
    "ip_address:equality",
    "ip_address:frequency",
    "ip_address:type",
    "logistics:equality",
    "logistics:frequency",
    "logistics:type",
    "medical_record:equality",
    "medical_record:frequency",
    "medical_record:type",
    "member:equality",
    "member:frequency",
    "member:type",
    "order:equality",
    "order:frequency",
    "order:type",
    "organization:equality",
    "organization:frequency",
    "organization:type",
    "patient:equality",
    "patient:frequency",
    "patient:type",
    "person:equality",
    "person:frequency",
    "person:type",
    "policy:equality",
    "policy:frequency",
    "policy:type",
    "ticket:equality",
    "ticket:frequency",
    "ticket:type",
    "user:equality",
    "user:frequency",
    "user:type"
  ],
  "residual_risks": [
    "external_linkage_attack_on_unsuppressed_quasi_identifiers",
    "contextual_inference_from_remaining_business_fields",
    "frequency_analysis_on_deterministic_tokens"
  ],
  "verification_results": [
    {
      "check": "direct_identifier_residue",
      "passed": true,
      "details": "residual_direct_identifier_count=0"
    }
  ]
}
```

### Utility Report

```json
{
  "report_type": "utility_proof",
  "report_id": "7bb21571-e0c0-4e8a-8f15-3b772a2ca8d9",
  "task_profile": "contract_risk_review",
  "external_view_digest": "sha256:1f94d779da0e4bcd774738095f6856e837b68258a35066b3544d0f6cabdddcec",
  "entity_preservation": {
    "required": 0,
    "preserved": 0,
    "ratio": 1.0
  },
  "relation_preservation": {
    "required": 1,
    "preserved": 1,
    "ratio": 1.0
  },
  "constraint_results": [
    {
      "check": "foreign_key_validity",
      "passed": true,
      "details": "contracts.customer_id -> customers.customer_id, source_values=0, missing=0"
    },
    {
      "check": "time_order_validity",
      "passed": true,
      "details": "events.event_time, values=0, violations=0"
    },
    {
      "check": "relation_preservation",
      "passed": true,
      "details": "relations(source, predicate, target), rows=0, incomplete=0"
    }
  ],
  "statistical_error_bounds": [
    "contract_count: mechanism=LaplaceCount epsilon=0.5 delta=0 beta=0.01 sensitivity=1 absolute_error_bound=9.210340",
    "event_status_histogram: mechanism=LaplaceHistogram epsilon=0.75 delta=0 beta=0.01 sensitivity=1 absolute_error_bound=6.140227",
    "payment_amount_mean: mechanism=LaplaceMean epsilon=1 delta=0 beta=0.01 sensitivity=10000 absolute_error_bound=46051.701860"
  ],
  "task_loss_bounds": []
}
```

## External Model Input

```json
{
  "model": "deepseek-v4-flash",
  "messages": [
    {
      "role": "system",
      "content": "You are a healthcare claim appeal reviewer. You will receive only a ProofGate external_view. Produce a professional claim-review memo. Preserve clinical context, administrative context, denial rationale, evidence conflict, claim amount bucket, and recommended next actions. Do not infer, reconstruct, or restore tokenized, generalized, or suppressed sensitive values. Keep token markers unchanged when referring to tokenized entities."
    },
    {
      "role": "user",
      "content": "{\"task_profile\": \"healthcare_claim_appeal_review\", \"task_instruction\": \"Return a structured English memo with: 1. case summary; 2. denial rationale; 3. evidence conflict; 4. risk assessment; 5. recommended next actions. Do not restore or guess any token, generalized address, or suppressed field.\", \"utility_terms_to_preserve\": [\"oncology\", \"denied\", \"appeal\", \"high-risk\", \"prior authorization\", \"incomplete\", \"faxed\", \"April 12, 2026\", \"38000 USD\", \"recommended next action\"], \"external_view\": {\"content_type\": \"text\", \"payload\": \"Synthetic claim appeal narrative: On April 18, 2026, <PERSON_08dcfc19269ec36a391c5b2c>, member <MEMBER_b716b76d94b29b38b438eda3> under policy <POLICY_8b70d0838e8162eb21ba1a1c>, contacted the oncology claims unit by phone at <PHONE_d93ad2f605f7092b53ce292d> and by email at <EMAIL_4a8a9930f8f98bea95fc77ab> regarding denied claim <CLAIM_0c460c8c59d21b27cd6461b4> and medical record <MEDICAL_RECORD_0ff157c98f781393745bf280>. The member reported that the service was ordered by <PERSON_dd2a014e7a821a5bed20cd1f> at <ORGANIZATION_6f2d0c1e11031ac928cc8ce6> after a follow-up visit for high-risk oncology monitoring. The original denial stated that prior authorization was incomplete, although the clinical note says the authorization packet was faxed on April 12, 2026.\\n\\nThe appeal letter lists the member's date of birth as [SUPPRESSED], home address as Springfield, IL, and includes an internal case note marked [SUPPRESSED]. The disputed claim amount is bucketed at 38000 USD. The reviewer should preserve the business context: oncology, denied, appeal, high-risk monitoring, prior authorization, missing documentation, fax timeline, and recommended next action. The reviewer must not infer or restore member identity, contact information, policy identifiers, claim identifiers, medical record identifiers, physician identity, facility identity, street address, date of birth, or internal notes.\"}}"
    }
  ],
  "temperature": 0
}
```

## External Model Output

```text
**CLAIM REVIEW MEMO – ONCOLOGY APPEAL**

**Case Summary**
On April 18, 2026, <PERSON_08dcfc19269ec36a391c5b2c> (member <MEMBER_b716b76d94b29b38b438eda3> under policy <POLICY_8b70d0838e8162eb21ba1a1c>) contacted the oncology claims unit regarding denied claim <CLAIM_0c460c8c59d21b27cd6461b4> and associated medical record <MEDICAL_RECORD_0ff157c98f781393745bf280>. The service was ordered by <PERSON_dd2a014e7a821a5bed20cd1f> at <ORGANIZATION_6f2d0c1e11031ac928cc8ce6> following a follow-up visit for high-risk oncology monitoring. The disputed claim amount is bucketed at **38000 USD**.

**Denial Rationale**
The original denial was based on a determination that prior authorization was **incomplete**.

**Evidence Conflict**
The clinical note for the service indicates that the authorization packet was **faxed** on **April 12, 2026**. This assertion directly conflicts with the denial rationale that the prior authorization was incomplete. No fax confirmation or additional documentation has been provided in the record to resolve this discrepancy.

**Risk Assessment**
This is a **high-risk** oncology **appeal** involving a substantial claim value (38000 USD). If the denial is upheld absent clear evidence of incomplete submission, the member may face significant financial liability and potential disruption of ongoing oncology monitoring. The conflict between the denial statement and the clinical note creates a material risk of an improper denial.

**Recommended Next Action**
The **recommended next action** is to request and review the fax confirmation or transmission log for the authorization packet dated April 12, 2026. If the fax was successfully sent and completes the prior authorization requirements, the denial should be overturned and the claim reprocessed. If evidence remains inconclusive, a formal request for the missing authorization documents should be made to the ordering provider.
```

## Output Inspection

```json
{
  "audit_id": "3cd34aa3-527b-43b5-9f42-a8d3c7179b52",
  "passed": true,
  "unauthorized_sensitive_output_count": 0,
  "findings": []
}
```

## Local Restoration Test

### Restoration On Actual External Output

```json
{
  "audit_id": "3cd34aa3-527b-43b5-9f42-a8d3c7179b52",
  "restored_output": "**CLAIM REVIEW MEMO – ONCOLOGY APPEAL**\n\n**Case Summary**  \nOn April 18, 2026, Robert Smith (member MBR-7788-2211 under policy POL-ACME-4401) contacted the oncology claims unit regarding denied claim CLM-2026-4401 and associated medical record MR-US-2026-3001. The service was ordered by Dr. Emily Carter at Northbridge Oncology Center following a follow-up visit for high-risk oncology monitoring. The disputed claim amount is bucketed at **38000 USD**.\n\n**Denial Rationale**  \nThe original denial was based on a determination that prior authorization was **incomplete**.\n\n**Evidence Conflict**  \nThe clinical note for the service indicates that the authorization packet was **faxed** on **April 12, 2026**. This assertion directly conflicts with the denial rationale that the prior authorization was incomplete. No fax confirmation or additional documentation has been provided in the record to resolve this discrepancy.\n\n**Risk Assessment**  \nThis is a **high-risk** oncology **appeal** involving a substantial claim value (38000 USD). If the denial is upheld absent clear evidence of incomplete submission, the member may face significant financial liability and potential disruption of ongoing oncology monitoring. The conflict between the denial statement and the clinical note creates a material risk of an improper denial.\n\n**Recommended Next Action**  \nThe **recommended next action** is to request and review the fax confirmation or transmission log for the authorization packet dated April 12, 2026. If the fax was successfully sent and completes the prior authorization requirements, the denial should be overturned and the claim reprocessed. If evidence remains inconclusive, a formal request for the missing authorization documents should be made to the ordering provider.",
  "replacements": 7
}
```

### Restoration Probe With Tokens From External View

```text
Restoration probe: <CLAIM_0c460c8c59d21b27cd6461b4> <EMAIL_4a8a9930f8f98bea95fc77ab> <MEDICAL_RECORD_0ff157c98f781393745bf280> <MEMBER_b716b76d94b29b38b438eda3> <ORGANIZATION_6f2d0c1e11031ac928cc8ce6> <PERSON_08dcfc19269ec36a391c5b2c> <PERSON_dd2a014e7a821a5bed20cd1f> <PHONE_d93ad2f605f7092b53ce292d> <POLICY_8b70d0838e8162eb21ba1a1c>
```

```json
{
  "audit_id": "3cd34aa3-527b-43b5-9f42-a8d3c7179b52",
  "restored_output": "Restoration probe: CLM-2026-4401 robert.smith.synthetic@example.test MR-US-2026-3001 MBR-7788-2211 Northbridge Oncology Center Robert Smith Dr. Emily Carter +1-212-555-0171 POL-ACME-4401",
  "replacements": 9
}
```

## Conclusion

- Privacy report passed: `true`
- Output inspection passed: `true`
- External-view sensitive leak count: `0`
- External-output sensitive leak count: `0`
- Utility score: `1.00`
- Overall passed: `true`
