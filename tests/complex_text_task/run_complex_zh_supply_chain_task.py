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


CASE_ID = "complex_zh_supply_chain_finance_risk_001"
TASK_PROFILE = "supply_chain_finance_risk_review"
ORIGINAL_TEXT = """合成供应链金融复核材料：2026年4月28日，借款企业“苏州星瀚精密制造有限公司”通过联系人陈启明提交应收账款融资申请，联系人手机号为 139-2188-4501，邮箱为 chen.qiming.synthetic@example.test。该企业统一社会信用代码为 91320594MA1SYNTH88，开户行为“合成银行苏州工业园区支行”，银行账号为 6222 0202 8888 9911。申请融资金额归入 420万元 桶，基础交易对应采购订单 PO-2026-SZ-8841 和销售合同 SC-2026-ACME-7788，买方为“杭州云岚医疗设备集团有限公司”，买方联系人李若涵，电话 137-5566-0912，收货地址为 浙江省杭州市滨江区江南大道1888号。

材料显示，订单约定交付日期为2026年4月20日，物流单号为 LOG-7788-20260419，承运车辆车牌为 苏E7S921。发票号码 INV-2026-000918 的开票日期为2026年4月18日，发票金额归入 420万元 桶，但物流轨迹显示货物在2026年4月23日才完成签收，且签收人记录为王建舟。内部系统备注显示 synthetic-internal-risk-note: invoice-before-delivery; buyer-confirmation-pending。担保人为赵明远，身份证号 320581198609097731，手机号 136-7788-2091，与借款企业存在历史关联交易。历史事件中，2026年3月该借款企业曾出现一次发票回款延期，状态为 overdue_7d；2026年4月10日新增一笔供应商付款，收款方为“上海辰曜贸易有限公司”，银行账号为 6222 6666 0101 7788，付款用途备注为“设备预付款”，但该供应商未出现在本次订单的合同附件中。

风控复核要求：请基于脱敏后的材料输出专业风险复核意见，必须保留以下业务事实和关系：应收账款融资、订单交付、发票早于签收、买方确认待完成、担保人存在历史关联交易、曾发生 overdue_7d、供应商付款与合同附件不一致、建议补充买方确权函、物流签收证明、发票验真和资金用途说明。不得还原或猜测企业名称、联系人姓名、手机号、邮箱、统一社会信用代码、银行账号、采购订单号、销售合同号、物流单号、车牌号、发票号、身份证号、具体地址或内部风险备注原文。"""

EXPECTED_SENSITIVE_VALUES = [
    "苏州星瀚精密制造有限公司",
    "杭州云岚医疗设备集团有限公司",
    "上海辰曜贸易有限公司",
    "合成银行苏州工业园区支行",
    "陈启明",
    "李若涵",
    "赵明远",
    "王建舟",
    "139-2188-4501",
    "137-5566-0912",
    "136-7788-2091",
    "chen.qiming.synthetic@example.test",
    "91320594MA1SYNTH88",
    "6222 0202 8888 9911",
    "6222 6666 0101 7788",
    "PO-2026-SZ-8841",
    "SC-2026-ACME-7788",
    "LOG-7788-20260419",
    "苏E7S921",
    "INV-2026-000918",
    "320581198609097731",
    "浙江省杭州市滨江区江南大道1888号",
    "synthetic-internal-risk-note: invoice-before-delivery; buyer-confirmation-pending",
]

EXPECTED_UTILITY_TERMS = [
    "应收账款融资",
    "发票早于签收",
    "买方确认",
    "overdue_7d",
    "供应商付款",
    "合同附件",
    "买方确权函",
    "物流签收证明",
    "发票验真",
    "资金用途说明",
]

SENSITIVE_TABLE = [
    ("borrower organization", "苏州星瀚精密制造有限公司", "hmac_token"),
    ("buyer organization", "杭州云岚医疗设备集团有限公司", "hmac_token"),
    ("supplier organization", "上海辰曜贸易有限公司", "hmac_token"),
    ("contact person", "陈启明 / 李若涵 / 赵明远 / 王建舟", "hmac_token"),
    ("phone", "139-2188-4501 / 137-5566-0912 / 136-7788-2091", "hmac_token"),
    ("email", "chen.qiming.synthetic@example.test", "hmac_token"),
    ("registration id", "91320594MA1SYNTH88", "hmac_token"),
    ("bank account", "6222 0202 8888 9911 / 6222 6666 0101 7788", "hmac_token"),
    ("business ids", "PO / SC / LOG / INV identifiers", "hmac_token"),
    ("license plate", "苏E7S921", "hmac_token"),
    ("national id", "320581198609097731", "hmac_token"),
    ("street address", "浙江省杭州市滨江区江南大道1888号", "address_generalize"),
    ("internal note", "synthetic-internal-risk-note...", "suppress"),
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
        "> Public test-data notice: this report was generated from synthetic and simulated test data only. Names, organizations, identifiers, accounts, addresses, business events, original inputs, projected views, model I/O, and restored fields in this file are artificial fixtures for evaluating PrivaGate. They must not be interpreted as real personal, customer, patient, operational, or business data. The external API was used only as an OpenAI-compatible test endpoint, and no API keys or Authorization headers are recorded here.",
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
        lines.append(f"| `{term}` | Required for supply-chain finance risk review |")

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
            reason="synthetic Chinese complex text task reviewed before external dispatch",
        )

        external_view_text = json.dumps(external_view, ensure_ascii=False)
        external_view_sensitive_leaks = contains_any(external_view_text, EXPECTED_SENSITIVE_VALUES)

        system_prompt = (
            "You are a supply-chain finance risk review expert. You will receive only a "
            "PrivaGate external_view. Produce a professional Chinese risk review memo. "
            "Preserve business type, subject roles, event timeline, risk facts, evidence gaps, "
            "recommended due-diligence materials, and conclusion. Do not infer, reconstruct, "
            "or restore tokenized, generalized, or suppressed sensitive values. Keep token "
            "markers unchanged when referring to tokenized entities."
        )
        user_prompt = json.dumps(
            {
                "task_profile": TASK_PROFILE,
                "task_instruction": (
                    "请输出结构化中文复核意见，包含：一、业务摘要；二、主体与关系；三、关键时间线；"
                    "四、主要风险点；五、证据缺口；六、建议补充材料；七、初步结论。"
                    "不得还原或猜测任何 token、泛化地址或被抑制字段。"
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
