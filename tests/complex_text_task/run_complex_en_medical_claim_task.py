import argparse
import datetime as dt
import json
import os
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tests" / "external_api_simulation"))

from run_external_api_simulation import (  # noqa: E402
    DEFAULT_GATEWAY,
    approve_manual_review_if_needed,
    chat_completion,
    chat_completion_request,
    inspect_output,
    project,
    restore_output,
    start_gateway_if_needed,
)


CASE_ID = "complex_en_medical_claim_appeal_text_001"
TASK_PROFILE = "healthcare_claim_appeal_review"
ORIGINAL_TEXT = """Synthetic claim appeal narrative: On April 18, 2026, Robert Smith, member MBR-7788-2211 under policy POL-ACME-4401, contacted the oncology claims unit by phone at +1-212-555-0171 and by email at robert.smith.synthetic@example.test regarding denied claim CLM-2026-4401 and medical record MR-US-2026-3001. The member reported that the service was ordered by Dr. Emily Carter at Northbridge Oncology Center after a follow-up visit for high-risk oncology monitoring. The original denial stated that prior authorization was incomplete, although the clinical note says the authorization packet was faxed on April 12, 2026.

The appeal letter lists the member's date of birth as 1979-11-24, home address as 742 Evergreen Terrace, Springfield, IL 62704, and includes an internal case note marked synthetic-sensitive-oncology-note. The disputed claim amount is bucketed at 38000 USD. The reviewer should preserve the business context: oncology, denied, appeal, high-risk monitoring, prior authorization, missing documentation, fax timeline, and recommended next action. The reviewer must not infer or restore member identity, contact information, policy identifiers, claim identifiers, medical record identifiers, physician identity, facility identity, street address, date of birth, or internal notes."""

EXPECTED_SENSITIVE_VALUES = [
    "Robert Smith",
    "MBR-7788-2211",
    "POL-ACME-4401",
    "+1-212-555-0171",
    "robert.smith.synthetic@example.test",
    "CLM-2026-4401",
    "MR-US-2026-3001",
    "Dr. Emily Carter",
    "Northbridge Oncology Center",
    "1979-11-24",
    "742 Evergreen Terrace",
    "Springfield, IL 62704",
    "synthetic-sensitive-oncology-note",
]

EXPECTED_UTILITY_TERMS = [
    "oncology",
    "denied",
    "appeal",
    "high-risk",
    "prior authorization",
    "incomplete",
    "faxed",
    "April 12, 2026",
    "38000 USD",
    "recommended next action",
]

SENSITIVE_TABLE = [
    ("member name", "Robert Smith", "hmac_token"),
    ("member id", "MBR-7788-2211", "hmac_token"),
    ("policy id", "POL-ACME-4401", "hmac_token"),
    ("claim id", "CLM-2026-4401", "hmac_token"),
    ("medical record id", "MR-US-2026-3001", "hmac_token"),
    ("phone", "+1-212-555-0171", "hmac_token"),
    ("email", "robert.smith.synthetic@example.test", "hmac_token"),
    ("physician", "Dr. Emily Carter", "hmac_token"),
    ("facility", "Northbridge Oncology Center", "hmac_token"),
    ("date of birth", "1979-11-24", "suppress"),
    ("street address", "742 Evergreen Terrace, Springfield, IL 62704", "address_generalize"),
    ("internal note", "synthetic-sensitive-oncology-note", "suppress"),
]


def contains_any(text, values):
    return [value for value in values if value and value in text]


def score_utility(output, terms):
    if not terms:
        return 1.0
    hits = sum(1 for term in terms if term.lower() in output.lower())
    return hits / len(terms)


def extract_tokens(value):
    text = json.dumps(value, ensure_ascii=False)
    return sorted(set(re.findall(r"<[A-Z_]+_[0-9a-f]+>", text)))


def fenced_json(value):
    return "```json\n" + json.dumps(value, ensure_ascii=False, indent=2) + "\n```"


def fenced_text(value):
    return "```text\n" + value + "\n```"


def write_markdown(path, result):
    lines = [
        f"# Complex Text Task Report: {CASE_ID}",
        "",
        "> Public test-data notice: this report was generated from synthetic and simulated test data only. Names, organizations, identifiers, accounts, addresses, medical facts, original inputs, projected views, model I/O, and restored fields in this file are artificial fixtures for evaluating PrivaGate. They must not be interpreted as real personal, customer, patient, operational, or business data. The external API was used only as an OpenAI-compatible test endpoint, and no API keys or Authorization headers are recorded here.",
        "",
        "## Test Task",
        "",
        f"- Case ID: `{CASE_ID}`",
        f"- Task profile: `{TASK_PROFILE}`",
        f"- Created at: `{result['created_at']}`",
        f"- Gateway: `{result['gateway_url']}`",
        f"- External model: `{result['external_model']['model']}`",
        "",
        "## Original Synthetic Input",
        "",
        fenced_text(ORIGINAL_TEXT),
        "",
        "## Sensitive Values Expected To Be Removed",
        "",
        "| Type | Synthetic value | Expected mechanism |",
        "|---|---|---|",
    ]
    for kind, value, mechanism in SENSITIVE_TABLE:
        lines.append(f"| {kind} | `{value}` | `{mechanism}` |")

    lines.extend(
        [
            "",
            "## Utility Facts Expected To Be Preserved",
            "",
            "| Utility fact | Purpose |",
            "|---|---|",
        ]
    )
    for term in EXPECTED_UTILITY_TERMS:
        lines.append(f"| `{term}` | Required for healthcare claim appeal review |")

    lines.extend(
        [
            "",
            "## PrivaGate Projection",
            "",
            f"- Audit ID: `{result['audit_id']}`",
            f"- Input digest: `{result['input_digest']}`",
            f"- External view digest: `{result['external_view_digest']}`",
            f"- External-view raw sensitive leaks: `{len(result['external_view_sensitive_leaks'])}`",
            "",
            "### External View",
            "",
            fenced_json(result["projection"]["external_view"]),
            "",
            "### Privacy Report",
            "",
            fenced_json(result["projection"]["privacy_report"]),
            "",
            "### Utility Report",
            "",
            fenced_json(result["projection"]["utility_report"]),
            "",
            "### Manual Review Gate",
            "",
            fenced_json(result["manual_review"]),
            "",
            "## External Model Input",
            "",
            fenced_json(result["external_model"]["request"]),
            "",
            "## External Model Output",
            "",
            fenced_text(result["external_model"]["output"]),
            "",
            "## Output Inspection",
            "",
            fenced_json(result["inspection"]),
            "",
            "## Local Restoration Test",
            "",
            "### Restoration On Actual External Output",
            "",
            fenced_json(result["restoration"]["actual_output"]),
            "",
            "### Restoration Probe With Tokens From External View",
            "",
            fenced_text(result["restoration"]["probe_input"]),
            "",
            fenced_json(result["restoration"]["probe_result"]),
            "",
            "## Conclusion",
            "",
            f"- Privacy report passed: `{str(result['privacy_passed']).lower()}`",
            f"- Output inspection passed: `{str(result['inspection']['passed']).lower()}`",
            f"- External-view sensitive leak count: `{len(result['external_view_sensitive_leaks'])}`",
            f"- External-output sensitive leak count: `{len(result['external_model']['sensitive_leaks'])}`",
            f"- Utility score: `{result['external_model']['utility_score']:.2f}`",
            f"- Overall passed: `{str(result['passed']).lower()}`",
        ]
    )
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("--gateway-url", default=os.environ.get("PRIVAGATE_URL", DEFAULT_GATEWAY))
    parser.add_argument("--external-base-url", default=os.environ.get("EXTERNAL_MODEL_BASE_URL", ""))
    parser.add_argument("--external-api-key", default=os.environ.get("EXTERNAL_MODEL_API_KEY", ""))
    parser.add_argument("--external-model", default=os.environ.get("EXTERNAL_MODEL_NAME", ""))
    parser.add_argument("--timeout", type=int, default=180)
    parser.add_argument("--model-retries", type=int, default=3)
    parser.add_argument("--utility-threshold", type=float, default=0.6)
    parser.add_argument("--out-dir", default=str(ROOT / "data" / "complex-text-task-results"))
    return parser.parse_args()


def main():
    args = parse_args()
    missing = [
        name
        for name, value in [
            ("EXTERNAL_MODEL_BASE_URL", args.external_base_url),
            ("EXTERNAL_MODEL_API_KEY", args.external_api_key),
            ("EXTERNAL_MODEL_NAME", args.external_model),
        ]
        if not value
    ]
    if missing:
        raise SystemExit("missing required external model config: " + ", ".join(missing))

    gateway_process = start_gateway_if_needed(args.gateway_url)
    try:
        case_input = {"content_type": "text", "payload": ORIGINAL_TEXT}
        projection = project(args.gateway_url, case_input)
        audit_id = projection["audit_summary"]["audit_id"]
        external_view = projection["external_view"]
        manual_review = approve_manual_review_if_needed(
            args.gateway_url,
            projection,
            reason="synthetic English complex text task reviewed before external dispatch",
        )

        external_view_text = json.dumps(external_view, ensure_ascii=False)
        external_view_sensitive_leaks = contains_any(external_view_text, EXPECTED_SENSITIVE_VALUES)

        system_prompt = (
            "You are a healthcare claim appeal reviewer. You will receive only a "
            "PrivaGate external_view. Produce a professional claim-review memo. "
            "Preserve clinical context, administrative context, denial rationale, "
            "evidence conflict, claim amount bucket, and recommended next actions. "
            "Do not infer, reconstruct, or restore tokenized, generalized, or suppressed "
            "sensitive values. Keep token markers unchanged when referring to tokenized entities."
        )
        user_prompt = json.dumps(
            {
                "task_profile": TASK_PROFILE,
                "task_instruction": (
                    "Return a structured English memo with: 1. case summary; "
                    "2. denial rationale; 3. evidence conflict; 4. risk assessment; "
                    "5. recommended next actions. Do not restore or guess any token, "
                    "generalized address, or suppressed field."
                ),
                "utility_terms_to_preserve": EXPECTED_UTILITY_TERMS,
                "external_view": external_view,
            },
            ensure_ascii=False,
        )
        external_request = chat_completion_request(
            args.external_model,
            [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt},
            ],
        )
        external_output, external_response = chat_completion(
            args.external_base_url,
            args.external_api_key,
            external_request,
            timeout=args.timeout,
            retries=args.model_retries,
            label=CASE_ID,
        )

        inspection = inspect_output(args.gateway_url, audit_id, external_output)
        actual_restoration = restore_output(args.gateway_url, audit_id, external_output)

        tokens = extract_tokens(external_view)
        probe_input = "Restoration probe: " + " ".join(tokens[:12])
        probe_restoration = restore_output(args.gateway_url, audit_id, probe_input)

        external_output_leaks = contains_any(external_output, EXPECTED_SENSITIVE_VALUES)
        utility_score = score_utility(external_output, EXPECTED_UTILITY_TERMS)
        privacy_passed = all(
            item["passed"] for item in projection["privacy_report"]["verification_results"]
        )
        passed = (
            privacy_passed
            and len(external_view_sensitive_leaks) == 0
            and inspection["passed"]
            and len(external_output_leaks) == 0
            and utility_score >= args.utility_threshold
        )

        result = {
            "created_at": dt.datetime.now(dt.timezone.utc).isoformat(),
            "gateway_url": args.gateway_url,
            "case_id": CASE_ID,
            "audit_id": audit_id,
            "input_digest": projection["audit_summary"]["input_digest"],
            "external_view_digest": projection["audit_summary"]["external_view_digest"],
            "projection": projection,
            "manual_review": manual_review,
            "privacy_passed": privacy_passed,
            "external_view_sensitive_leaks": external_view_sensitive_leaks,
            "external_model": {
                "model": args.external_model,
                "request": external_request,
                "response": external_response,
                "output": external_output,
                "sensitive_leaks": external_output_leaks,
                "utility_score": utility_score,
            },
            "inspection": inspection,
            "restoration": {
                "actual_output": actual_restoration,
                "probe_input": probe_input,
                "probe_result": probe_restoration,
            },
            "passed": passed,
        }

        out_dir = Path(args.out_dir)
        out_dir.mkdir(parents=True, exist_ok=True)
        json_path = out_dir / f"{CASE_ID}.json"
        md_path = out_dir / f"{CASE_ID}.md"
        json_path.write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
        write_markdown(md_path, result)
        print(
            json.dumps(
                {
                    "case_id": CASE_ID,
                    "passed": passed,
                    "privacy_passed": privacy_passed,
                    "inspection_passed": inspection["passed"],
                    "external_view_sensitive_leak_count": len(external_view_sensitive_leaks),
                    "external_output_sensitive_leak_count": len(external_output_leaks),
                    "utility_score": utility_score,
                    "markdown_report": str(md_path),
                    "json_report": str(json_path),
                },
                ensure_ascii=False,
                indent=2,
            )
        )
        if not passed:
            raise SystemExit(1)
    finally:
        if gateway_process is not None:
            gateway_process.terminate()
            try:
                gateway_process.wait(timeout=10)
            except Exception:
                gateway_process.kill()


if __name__ == "__main__":
    main()
