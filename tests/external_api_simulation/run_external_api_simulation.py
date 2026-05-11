import argparse
import datetime as dt
import http.client
import json
import os
import shutil
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_GATEWAY = "http://127.0.0.1:8080"


def default_gateway_env():
    env = os.environ.copy()
    env.setdefault("CARGO_HOME", str(ROOT / ".cargo-home"))
    env.setdefault("CARGO_TARGET_DIR", str(ROOT / "target"))
    if (ROOT / ".rustup-home" / "toolchains").exists():
        env.setdefault("RUSTUP_HOME", str(ROOT / ".rustup-home"))
    env.setdefault("PRIVAGATE_HMAC_KEY", "external-api-simulation-secret")
    env.setdefault("PRIVAGATE_POLICY_PATH", str(ROOT / "config" / "policy.sample.json"))
    env.setdefault("PRIVAGATE_MAPPING_LOG", str(ROOT / "data" / "external-api-simulation-mappings.jsonl"))
    env.setdefault("PRIVAGATE_AUDIT_LOG", str(ROOT / "data" / "external-api-simulation-audit.jsonl"))
    for key in ["CARGO_HOME", "CARGO_TARGET_DIR", "PRIVAGATE_MAPPING_LOG", "PRIVAGATE_AUDIT_LOG"]:
        path = Path(env[key])
        directory = path if path.suffix == "" else path.parent
        directory.mkdir(parents=True, exist_ok=True)
    return env


def http_json(method, url, payload=None, headers=None, timeout=120):
    body = None if payload is None else json.dumps(payload, ensure_ascii=False).encode("utf-8")
    request = urllib.request.Request(
        url,
        data=body,
        method=method,
        headers={
            "Content-Type": "application/json; charset=utf-8",
            **(headers or {}),
        },
    )
    with urllib.request.urlopen(request, timeout=timeout) as response:
        return json.loads(response.read().decode("utf-8"))


def chat_completion_request(model, messages):
    return {
        "model": model,
        "messages": messages,
        "temperature": 0,
    }


def chat_completion(base_url, api_key, payload, timeout=120, retries=2, label="model"):
    url = base_url.rstrip("/") + "/chat/completions"
    headers = {"Authorization": f"Bearer {api_key}"}
    retryable_errors = (
        http.client.IncompleteRead,
        TimeoutError,
        ConnectionResetError,
        urllib.error.URLError,
    )
    for attempt in range(retries + 1):
        try:
            response = http_json("POST", url, payload, headers=headers, timeout=timeout)
            return response["choices"][0]["message"]["content"], response
        except retryable_errors as error:
            if attempt >= retries:
                raise
            delay_seconds = 2 * (attempt + 1)
            print(
                f"{label} request failed with {error.__class__.__name__}; "
                f"retrying in {delay_seconds}s ({attempt + 1}/{retries})",
                file=sys.stderr,
            )
            time.sleep(delay_seconds)


def wait_gateway(gateway_url, timeout_seconds=30):
    deadline = time.time() + timeout_seconds
    while time.time() < deadline:
        try:
            health = http_json("GET", gateway_url.rstrip("/") + "/healthz", timeout=5)
            if health.get("status") == "ok":
                return True
        except Exception:
            time.sleep(0.5)
    return False


def start_gateway_if_needed(gateway_url):
    if wait_gateway(gateway_url, timeout_seconds=2):
        return None

    env = default_gateway_env()
    exe_name = "privagate-gateway.exe" if os.name == "nt" else "privagate-gateway"
    exe = Path(env["CARGO_TARGET_DIR"]) / "debug" / exe_name
    if not exe.exists():
        if os.name == "nt" and (ROOT / "scripts" / "cargo.ps1").exists():
            run(
                [
                    "powershell",
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    str(ROOT / "scripts" / "cargo.ps1"),
                    "build",
                    "-p",
                    "privagate-gateway",
                ],
                env=env,
            )
        elif shutil.which("cargo"):
            run(["cargo", "build", "-p", "privagate-gateway"], env=env)
        else:
            raise RuntimeError("cargo is not installed or not on PATH")

    process = subprocess.Popen(
        [str(exe)],
        cwd=str(ROOT),
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    if not wait_gateway(gateway_url, timeout_seconds=30):
        process.terminate()
        raise RuntimeError("privagate-gateway did not become healthy")
    return process


def run(command, env=None):
    completed = subprocess.run(command, cwd=str(ROOT), text=True, env=env)
    if completed.returncode != 0:
        raise RuntimeError(f"command failed: {' '.join(command)}")


def project(gateway_url, case_input, task_profile=None):
    payload = dict(case_input)
    if task_profile:
        payload["task_profile"] = task_profile
    return http_json("POST", gateway_url.rstrip() + "/v1/project", payload)


def approve_manual_review_if_needed(
    gateway_url,
    projection,
    reviewer="synthetic-test-reviewer",
    reason="synthetic external API simulation reviewed before external dispatch",
):
    review = projection.get("manual_review")
    if not review or not review.get("required"):
        return review
    if review.get("status") == "approved":
        return review
    return http_json(
        "POST",
        gateway_url.rstrip() + "/v1/review/approve",
        {
            "audit_id": projection["audit_summary"]["audit_id"],
            "reviewer": reviewer,
            "reason": reason,
        },
    )


def inspect_output(gateway_url, audit_id, output):
    return http_json(
        "POST",
        gateway_url.rstrip() + "/v1/inspect-output",
        {"audit_id": audit_id, "output": output},
    )


def restore_output(gateway_url, audit_id, output):
    return http_json(
        "POST",
        gateway_url.rstrip() + "/v1/restore-output",
        {"audit_id": audit_id, "output": output},
    )


def score_utility(output, terms):
    if not terms:
        return 1.0
    lower = output.lower()
    hits = sum(1 for term in terms if term.lower() in lower)
    return hits / len(terms)


def contains_any(text, values):
    return [value for value in values if value and value in text]


def run_case(args, case):
    task_instruction = case.get(
        "task_instruction",
        "基于 external_view 完成风险摘要，保留角色、状态、时间顺序和主要业务关系。",
    )
    local_prompt = [
        {
            "role": "system",
            "content": (
                "你在本次测试中模拟本地辅助模型。测试数据是合成数据，允许发送给你。"
                "请只输出 JSON，字段包括 risk_tags、sensitive_candidates、utility_notes、task_profile。"
            ),
        },
        {
            "role": "user",
            "content": json.dumps(
                {
                    "task_profile": case.get("task_profile"),
                    "task_instruction": task_instruction,
                    "payload": case["input"]["payload"],
                },
                ensure_ascii=False,
            ),
        },
    ]
    local_request = chat_completion_request(args.local_model, local_prompt)
    local_output, local_response = chat_completion(
        args.local_base_url,
        args.local_api_key,
        local_request,
        timeout=args.timeout,
        retries=args.model_retries,
        label=f"{case['case_id']} local_model",
    )

    projection = project(args.gateway_url, case["input"], case.get("task_profile"))
    audit_id = projection["audit_summary"]["audit_id"]
    external_view = projection["external_view"]
    manual_review = approve_manual_review_if_needed(args.gateway_url, projection)
    task_contract_assessment = projection.get("task_contract_assessment", {})

    external_prompt = [
        {
            "role": "system",
            "content": (
                "你在本次测试中模拟外部业务模型。你只能使用 external_view 完成指定任务。"
                "不要猜测或还原真实姓名、手机号、身份证号、客户号、合同号、账号、病历号、设备号或任何 token 的原值。"
                "输出应保留可见的角色、状态、事件顺序、风险原因和可行动建议。"
            ),
        },
        {
            "role": "user",
            "content": json.dumps(
                {
                    "task_profile": case.get("task_profile"),
                    "task_instruction": task_instruction,
                    "utility_terms_to_preserve": case.get("expected_utility_terms", []),
                    "external_view": external_view,
                },
                ensure_ascii=False,
            ),
        },
    ]
    external_request = chat_completion_request(args.external_model, external_prompt)
    external_output, external_response = chat_completion(
        args.external_base_url,
        args.external_api_key,
        external_request,
        timeout=args.timeout,
        retries=args.model_retries,
        label=f"{case['case_id']} external_model",
    )

    inspection = inspect_output(args.gateway_url, audit_id, external_output)
    restoration = restore_output(args.gateway_url, audit_id, external_output)

    local_leaks = contains_any(local_output, case["expected_sensitive_values"])
    external_leaks = contains_any(external_output, case["expected_sensitive_values"])
    utility_score = score_utility(external_output, case["expected_utility_terms"])
    privacy_passed = all(item["passed"] for item in projection["privacy_report"]["verification_results"])
    utility_constraints_passed = all(
        item["passed"] for item in projection["utility_report"]["constraint_results"]
    )

    result = {
        "case_id": case["case_id"],
        "audit_id": audit_id,
        "input_digest": projection["audit_summary"]["input_digest"],
        "external_view_digest": projection["audit_summary"]["external_view_digest"],
        "manual_review": manual_review,
        "local_model": {
            "model": args.local_model,
            "output": local_output,
            "sensitive_echo_count": len(local_leaks),
            "sensitive_echo_values": local_leaks,
        },
        "external_model": {
            "model": args.external_model,
            "output": external_output,
            "sensitive_echo_count": len(external_leaks),
            "sensitive_echo_values": external_leaks,
            "utility_score": utility_score,
        },
        "gateway": {
            "privacy_passed": privacy_passed,
            "utility_constraints_passed": utility_constraints_passed,
            "task_contract_dispatch_allowed": task_contract_assessment.get(
                "dispatch_allowed", True
            ),
            "task_contract_issues": task_contract_assessment.get("issues", []),
            "output_inspection_passed": inspection["passed"],
            "unauthorized_sensitive_output_count": inspection[
                "unauthorized_sensitive_output_count"
            ],
            "restore_replacements": restoration["replacements"],
        },
        "passed": (
            privacy_passed
            and utility_constraints_passed
            and inspection["passed"]
            and len(external_leaks) == 0
            and utility_score >= args.utility_threshold
        ),
    }
    if args.record_model_io:
        result["model_io"] = {
            "local_model": {
                "request": local_request,
                "response": local_response,
                "content": local_output,
            },
            "desensitization_gateway": {
                "request": case["input"],
                "response": projection,
                "manual_review": manual_review,
                "external_view": external_view,
            },
            "external_model": {
                "request": external_request,
                "response": external_response,
                "content": external_output,
            },
        }
    return result


def write_model_io_trace(path, report):
    with path.open("w", encoding="utf-8") as file:
        for case in report["cases"]:
            model_io = case.get("model_io")
            if not model_io:
                continue
            for role, io_record in model_io.items():
                file.write(
                    json.dumps(
                        {
                            "created_at": report["created_at"],
                            "case_id": case["case_id"],
                            "audit_id": case["audit_id"],
                            "role": role,
                            **io_record,
                        },
                        ensure_ascii=False,
                    )
                    + "\n"
                )


def write_markdown(path, report):
    lines = [
        "# 外部 API 模拟实测结果",
        "",
        f"- 时间：{report['created_at']}",
        f"- 网关：`{report['gateway_url']}`",
        f"- 本地模型模拟：`{report['local_model']}`",
        f"- 外部模型模拟：`{report['external_model']}`",
        f"- 记录模型输入输出：{str(report['record_model_io']).lower()}",
        f"- 用例数：{report['summary']['case_count']}",
        f"- 通过数：{report['summary']['passed_count']}",
        f"- 失败数：{report['summary']['failed_count']}",
        "",
        "## 用例结果",
        "",
        "| case_id | passed | privacy | utility_constraints | manual_review | output_inspection | utility_score | external_leaks |",
        "|---|---:|---:|---:|---|---:|---:|---:|",
    ]
    for case in report["cases"]:
        review = case.get("manual_review")
        review_status = "not_required" if not review else review.get("status", "unknown")
        lines.append(
            "| {case_id} | {passed} | {privacy} | {constraints} | {review} | {inspection} | {score:.2f} | {leaks} |".format(
                case_id=case["case_id"],
                passed=str(case["passed"]).lower(),
                privacy=str(case["gateway"]["privacy_passed"]).lower(),
                constraints=str(case["gateway"]["utility_constraints_passed"]).lower(),
                review=review_status,
                inspection=str(case["gateway"]["output_inspection_passed"]).lower(),
                score=case["external_model"]["utility_score"],
                leaks=case["external_model"]["sensitive_echo_count"],
            )
        )
    if report["record_model_io"]:
        lines.extend(
            [
                "",
                "## 输入输出记录",
                "",
                "- 完整模型请求和响应写入 `model_io_trace.jsonl`。",
                "- 汇总 JSON 的每个 case 也包含 `model_io` 字段。",
                "- 记录不包含 API Key 或 Authorization 请求头。",
            ]
        )
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("--dataset", default=str(Path(__file__).with_name("dataset.json")))
    parser.add_argument("--gateway-url", default=os.environ.get("PRIVAGATE_URL", DEFAULT_GATEWAY))
    parser.add_argument("--local-base-url", default=os.environ.get("LOCAL_MODEL_BASE_URL", ""))
    parser.add_argument("--local-api-key", default=os.environ.get("LOCAL_MODEL_API_KEY", ""))
    parser.add_argument("--local-model", default=os.environ.get("LOCAL_MODEL_NAME", ""))
    parser.add_argument("--external-base-url", default=os.environ.get("EXTERNAL_MODEL_BASE_URL", ""))
    parser.add_argument("--external-api-key", default=os.environ.get("EXTERNAL_MODEL_API_KEY", ""))
    parser.add_argument("--external-model", default=os.environ.get("EXTERNAL_MODEL_NAME", ""))
    parser.add_argument("--utility-threshold", type=float, default=0.34)
    parser.add_argument("--timeout", type=int, default=120)
    parser.add_argument("--model-retries", type=int, default=2)
    parser.add_argument("--out-dir", default=str(ROOT / "data" / "external-api-simulation-results"))
    parser.add_argument(
        "--record-model-io",
        action="store_true",
        help="Record full model request and response bodies. API keys and headers are never written.",
    )
    return parser.parse_args()


def main():
    args = parse_args()
    missing = [
        name
        for name, value in [
            ("LOCAL_MODEL_BASE_URL", args.local_base_url),
            ("LOCAL_MODEL_API_KEY", args.local_api_key),
            ("LOCAL_MODEL_NAME", args.local_model),
            ("EXTERNAL_MODEL_BASE_URL", args.external_base_url),
            ("EXTERNAL_MODEL_API_KEY", args.external_api_key),
            ("EXTERNAL_MODEL_NAME", args.external_model),
        ]
        if not value
    ]
    if missing:
        raise SystemExit("missing required model config: " + ", ".join(missing))

    gateway_process = start_gateway_if_needed(args.gateway_url)
    try:
        dataset = json.loads(Path(args.dataset).read_text(encoding="utf-8"))
        cases = []
        for index, case in enumerate(dataset["cases"], start=1):
            print(
                f"running case {index}/{len(dataset['cases'])}: {case['case_id']}",
                file=sys.stderr,
            )
            cases.append(run_case(args, case))
        report = {
            "created_at": dt.datetime.now(dt.timezone.utc).isoformat(),
            "gateway_url": args.gateway_url,
            "local_model": args.local_model,
            "external_model": args.external_model,
            "record_model_io": args.record_model_io,
            "summary": {
                "case_count": len(cases),
                "passed_count": sum(1 for case in cases if case["passed"]),
                "failed_count": sum(1 for case in cases if not case["passed"]),
            },
            "cases": cases,
        }
        out_dir = Path(args.out_dir)
        out_dir.mkdir(parents=True, exist_ok=True)
        json_path = out_dir / "external_api_simulation_report.json"
        md_path = out_dir / "external_api_simulation_report.md"
        trace_path = out_dir / "model_io_trace.jsonl"
        json_path.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
        write_markdown(md_path, report)
        if args.record_model_io:
            write_model_io_trace(trace_path, report)
        print(json.dumps(report["summary"], ensure_ascii=False, indent=2))
        if report["summary"]["failed_count"] > 0:
            raise SystemExit(1)
    finally:
        if gateway_process is not None:
            gateway_process.terminate()
            try:
                gateway_process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                gateway_process.kill()


if __name__ == "__main__":
    main()
