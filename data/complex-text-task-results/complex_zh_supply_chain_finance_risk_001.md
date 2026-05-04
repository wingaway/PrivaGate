# Complex Text Task Report: complex_zh_supply_chain_finance_risk_001

> Public test-data notice: this report was generated from synthetic and simulated test data only. Names, organizations, identifiers, accounts, addresses, business events, original inputs, projected views, model I/O, and restored fields in this file are artificial fixtures for evaluating ProofGate. They must not be interpreted as real personal, customer, patient, operational, or business data. The external API was used only as an OpenAI-compatible test endpoint, and no API keys or Authorization headers are recorded here.

## Test Task

- Case ID: `complex_zh_supply_chain_finance_risk_001`
- Task profile: `supply_chain_finance_risk_review`
- Created at: `2026-05-04T09:02:10.794608+00:00`
- Gateway: `http://127.0.0.1:8080`
- External model: `deepseek-v4-flash`

## Original Synthetic Input

```text
合成供应链金融复核材料：2026年4月28日，借款企业“苏州星瀚精密制造有限公司”通过联系人陈启明提交应收账款融资申请，联系人手机号为 139-2188-4501，邮箱为 chen.qiming.synthetic@example.test。该企业统一社会信用代码为 91320594MA1SYNTH88，开户行为“合成银行苏州工业园区支行”，银行账号为 6222 0202 8888 9911。申请融资金额归入 420万元 桶，基础交易对应采购订单 PO-2026-SZ-8841 和销售合同 SC-2026-ACME-7788，买方为“杭州云岚医疗设备集团有限公司”，买方联系人李若涵，电话 137-5566-0912，收货地址为 浙江省杭州市滨江区江南大道1888号。

材料显示，订单约定交付日期为2026年4月20日，物流单号为 LOG-7788-20260419，承运车辆车牌为 苏E7S921。发票号码 INV-2026-000918 的开票日期为2026年4月18日，发票金额归入 420万元 桶，但物流轨迹显示货物在2026年4月23日才完成签收，且签收人记录为王建舟。内部系统备注显示 synthetic-internal-risk-note: invoice-before-delivery; buyer-confirmation-pending。担保人为赵明远，身份证号 320581198609097731，手机号 136-7788-2091，与借款企业存在历史关联交易。历史事件中，2026年3月该借款企业曾出现一次发票回款延期，状态为 overdue_7d；2026年4月10日新增一笔供应商付款，收款方为“上海辰曜贸易有限公司”，银行账号为 6222 6666 0101 7788，付款用途备注为“设备预付款”，但该供应商未出现在本次订单的合同附件中。

风控复核要求：请基于脱敏后的材料输出专业风险复核意见，必须保留以下业务事实和关系：应收账款融资、订单交付、发票早于签收、买方确认待完成、担保人存在历史关联交易、曾发生 overdue_7d、供应商付款与合同附件不一致、建议补充买方确权函、物流签收证明、发票验真和资金用途说明。不得还原或猜测企业名称、联系人姓名、手机号、邮箱、统一社会信用代码、银行账号、采购订单号、销售合同号、物流单号、车牌号、发票号、身份证号、具体地址或内部风险备注原文。
```

## Sensitive Values Expected To Be Removed

| Type | Synthetic value | Expected mechanism |
|---|---|---|
| borrower organization | `苏州星瀚精密制造有限公司` | `hmac_token` |
| buyer organization | `杭州云岚医疗设备集团有限公司` | `hmac_token` |
| supplier organization | `上海辰曜贸易有限公司` | `hmac_token` |
| contact person | `陈启明 / 李若涵 / 赵明远 / 王建舟` | `hmac_token` |
| phone | `139-2188-4501 / 137-5566-0912 / 136-7788-2091` | `hmac_token` |
| email | `chen.qiming.synthetic@example.test` | `hmac_token` |
| registration id | `91320594MA1SYNTH88` | `hmac_token` |
| bank account | `6222 0202 8888 9911 / 6222 6666 0101 7788` | `hmac_token` |
| business ids | `PO / SC / LOG / INV identifiers` | `hmac_token` |
| license plate | `苏E7S921` | `hmac_token` |
| national id | `320581198609097731` | `hmac_token` |
| street address | `浙江省杭州市滨江区江南大道1888号` | `address_generalize` |
| internal note | `synthetic-internal-risk-note...` | `suppress` |

## Utility Facts Expected To Be Preserved

| Utility fact | Purpose |
|---|---|
| `应收账款融资` | Required for supply-chain finance risk review |
| `发票早于签收` | Required for supply-chain finance risk review |
| `买方确认` | Required for supply-chain finance risk review |
| `overdue_7d` | Required for supply-chain finance risk review |
| `供应商付款` | Required for supply-chain finance risk review |
| `合同附件` | Required for supply-chain finance risk review |
| `买方确权函` | Required for supply-chain finance risk review |
| `物流签收证明` | Required for supply-chain finance risk review |
| `发票验真` | Required for supply-chain finance risk review |
| `资金用途说明` | Required for supply-chain finance risk review |

## ProofGate Projection

- Audit ID: `14b7944b-2213-4024-9774-1e3a3d7c3e7d`
- Input digest: `sha256:1ff12ef693138b2af2af5ffc630e81fd9d92b98e14747c8d5a0bd29cb488ac1b`
- External view digest: `sha256:c0f4c5600b15101da19cd1ccd11fa29f07d6df6a67a91c8fbba2ac55fe2c79f2`
- External-view raw sensitive leaks: `0`

### External View

```json
{
  "content_type": "text",
  "payload": "合成供应链金融复核材料：2026年4月28日，借款企业“<ORGANIZATION_821eb69849cdf27f7a2fdb02>”通过联系人<PERSON_e62f00d451cc04abe63b24e0>交应收账款融资申请，联系人<PERSON_9a32e824b37108b6c16aefc5> <PHONE_79a319019d967e1f246c0f74>，邮箱为 <EMAIL_5d5ff114c719fd7b0ba96c2f>。该企业统一社会信用代码为 <REGISTRATION_ID_415be918a7f34b1f8bb77c23>，开户行为“<ORGANIZATION_668851359621297828fb3e54>”，银行账号为 <BANK_CARD_338a7aac100f3eaf4daa242e>。申请融资金额归入 420万元 桶，基础交易对应采购订单 <ORDER_23d340fedf2cee082b7c4bfb> 和销售合同 <CONTRACT_cbd23b2d33d3e446cd245209>，买方为“<ORGANIZATION_81f5cccbb263a3227a2a81d6>”，买方联系人<PERSON_458abd69eeca29a56cf18269>，电话 <PHONE_ef7f5afce53c3ae1c9d86f19>，收货地址为 浙江省杭州市。\n\n材料显示，订单约定交付日期为2026年4月20日，物流单号为 <LOGISTICS_0b31eb4fc51553bccfbe438b>，承运车辆车牌为 <LICENSE_PLATE_8e23e2def8d89f26a43e250b>。发票号码 <INVOICE_c37ccabf8ebe80ae6c2c5dab> 的开票日期为2026年4月18日，发票金额归入 420万元 桶，但物流轨迹显示货物在2026年4月23日才完成签收，且签收人记录为<PERSON_d69094fa38ee1e28d80ed76a>。内部系统备注显示 [SUPPRESSED]。担保人为<PERSON_1945dcc580993f6d03f449a9>，身份证号 <NATIONAL_ID_0d4952296864882ffc15c57d>，手机号 <PHONE_2cdff5b143546a1a58e3682b>，与借款企业存在历史关联交易。历史事件中，2026年3月该借款企业曾出现一次发票回款延期，状态为 overdue_7d；2026年4月10日新增一笔供应商付款，收款方为“<ORGANIZATION_8ef2ef1dd0785118b705d554>”，银行账号为 <BANK_CARD_726cd361989c9116415701a7>，付款用途备注为“设备预付款”，但该供应商未出现在本次订单的合同附件中。\n\n风控复核要求：请基于脱敏后的材料输出专业风险复核意见，必须保留以下业务事实和关系：应收账款融资、订单交付、发票早于签收、买方确认待完成、担保人存在历史关联交易、曾发生 overdue_7d、供应商付款与合同附件不一致、建议补充买方确权函、物流签收证明、发票验真和资金用途说明。不得还原或猜测企业名称、联系人<PERSON_d576b968e92<PHONE_d25086eac6b161b67723f53d>>、手机号、邮箱、统一社会信用代码、银行账号、采购订单号、销售合同号、物流单号、车牌号、发票号、身份证号、具体地址或内部风险备注原文。"
}
```

### Privacy Report

```json
{
  "report_type": "privacy_proof",
  "report_id": "9c8dd1c2-a9f5-4d62-a4fa-3c3d4e9921a1",
  "created_at": "2026-05-04T09:01:30.088835900Z",
  "policy_version": "2026.05.03-production-sample",
  "input_digest": "sha256:1ff12ef693138b2af2af5ffc630e81fd9d92b98e14747c8d5a0bd29cb488ac1b",
  "external_view_digest": "sha256:c0f4c5600b15101da19cd1ccd11fa29f07d6df6a67a91c8fbba2ac55fe2c79f2",
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
      "field_name": "bank_card",
      "field_type": "bank_card",
      "mechanism": "hmac_token",
      "key_domain": "local-kms/proofgate-gateway/hmac/v1",
      "token_count": 1
    },
    {
      "field_name": "bank_card",
      "field_type": "bank_card",
      "mechanism": "hmac_token",
      "key_domain": "local-kms/proofgate-gateway/hmac/v1",
      "token_count": 1
    },
    {
      "field_name": "business_registration_id",
      "field_type": "registration_id",
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
      "field_name": "company_name",
      "field_type": "organization",
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
      "field_name": "contract_id",
      "field_type": "contract",
      "mechanism": "hmac_token",
      "key_domain": "local-kms/proofgate-gateway/hmac/v1",
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
      "field_name": "id_card",
      "field_type": "national_id",
      "mechanism": "hmac_token",
      "key_domain": "local-kms/proofgate-gateway/hmac/v1",
      "token_count": 1
    },
    {
      "field_name": "invoice_id",
      "field_type": "invoice",
      "mechanism": "hmac_token",
      "key_domain": "local-kms/proofgate-gateway/hmac/v1",
      "token_count": 1
    },
    {
      "field_name": "license_plate",
      "field_type": "license_plate",
      "mechanism": "hmac_token",
      "key_domain": "local-kms/proofgate-gateway/hmac/v1",
      "token_count": 1
    },
    {
      "field_name": "logistics_id",
      "field_type": "logistics",
      "mechanism": "hmac_token",
      "key_domain": "local-kms/proofgate-gateway/hmac/v1",
      "token_count": 1
    },
    {
      "field_name": "order_id",
      "field_type": "order",
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
      "field_name": "phone",
      "field_type": "phone",
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
      "field_name": "phone",
      "field_type": "phone",
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
  "report_id": "f65e911b-07d4-4aa1-8051-a80c297fde63",
  "task_profile": "contract_risk_review",
  "external_view_digest": "sha256:c0f4c5600b15101da19cd1ccd11fa29f07d6df6a67a91c8fbba2ac55fe2c79f2",
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
      "content": "You are a supply-chain finance risk review expert. You will receive only a ProofGate external_view. Produce a professional Chinese risk review memo. Preserve business type, subject roles, event timeline, risk facts, evidence gaps, recommended due-diligence materials, and conclusion. Do not infer, reconstruct, or restore tokenized, generalized, or suppressed sensitive values. Keep token markers unchanged when referring to tokenized entities."
    },
    {
      "role": "user",
      "content": "{\"task_profile\": \"supply_chain_finance_risk_review\", \"task_instruction\": \"请输出结构化中文复核意见，包含：一、业务摘要；二、主体与关系；三、关键时间线；四、主要风险点；五、证据缺口；六、建议补充材料；七、初步结论。不得还原或猜测任何 token、泛化地址或被抑制字段。\", \"utility_terms_to_preserve\": [\"应收账款融资\", \"发票早于签收\", \"买方确认\", \"overdue_7d\", \"供应商付款\", \"合同附件\", \"买方确权函\", \"物流签收证明\", \"发票验真\", \"资金用途说明\"], \"external_view\": {\"content_type\": \"text\", \"payload\": \"合成供应链金融复核材料：2026年4月28日，借款企业“<ORGANIZATION_821eb69849cdf27f7a2fdb02>”通过联系人<PERSON_e62f00d451cc04abe63b24e0>交应收账款融资申请，联系人<PERSON_9a32e824b37108b6c16aefc5> <PHONE_79a319019d967e1f246c0f74>，邮箱为 <EMAIL_5d5ff114c719fd7b0ba96c2f>。该企业统一社会信用代码为 <REGISTRATION_ID_415be918a7f34b1f8bb77c23>，开户行为“<ORGANIZATION_668851359621297828fb3e54>”，银行账号为 <BANK_CARD_338a7aac100f3eaf4daa242e>。申请融资金额归入 420万元 桶，基础交易对应采购订单 <ORDER_23d340fedf2cee082b7c4bfb> 和销售合同 <CONTRACT_cbd23b2d33d3e446cd245209>，买方为“<ORGANIZATION_81f5cccbb263a3227a2a81d6>”，买方联系人<PERSON_458abd69eeca29a56cf18269>，电话 <PHONE_ef7f5afce53c3ae1c9d86f19>，收货地址为 浙江省杭州市。\\n\\n材料显示，订单约定交付日期为2026年4月20日，物流单号为 <LOGISTICS_0b31eb4fc51553bccfbe438b>，承运车辆车牌为 <LICENSE_PLATE_8e23e2def8d89f26a43e250b>。发票号码 <INVOICE_c37ccabf8ebe80ae6c2c5dab> 的开票日期为2026年4月18日，发票金额归入 420万元 桶，但物流轨迹显示货物在2026年4月23日才完成签收，且签收人记录为<PERSON_d69094fa38ee1e28d80ed76a>。内部系统备注显示 [SUPPRESSED]。担保人为<PERSON_1945dcc580993f6d03f449a9>，身份证号 <NATIONAL_ID_0d4952296864882ffc15c57d>，手机号 <PHONE_2cdff5b143546a1a58e3682b>，与借款企业存在历史关联交易。历史事件中，2026年3月该借款企业曾出现一次发票回款延期，状态为 overdue_7d；2026年4月10日新增一笔供应商付款，收款方为“<ORGANIZATION_8ef2ef1dd0785118b705d554>”，银行账号为 <BANK_CARD_726cd361989c9116415701a7>，付款用途备注为“设备预付款”，但该供应商未出现在本次订单的合同附件中。\\n\\n风控复核要求：请基于脱敏后的材料输出专业风险复核意见，必须保留以下业务事实和关系：应收账款融资、订单交付、发票早于签收、买方确认待完成、担保人存在历史关联交易、曾发生 overdue_7d、供应商付款与合同附件不一致、建议补充买方确权函、物流签收证明、发票验真和资金用途说明。不得还原或猜测企业名称、联系人<PERSON_d576b968e92<PHONE_d25086eac6b161b67723f53d>>、手机号、邮箱、统一社会信用代码、银行账号、采购订单号、销售合同号、物流单号、车牌号、发票号、身份证号、具体地址或内部风险备注原文。\"}}"
    }
  ],
  "temperature": 0
}
```

## External Model Output

```text
**供应链金融风险复核意见**

**一、业务摘要**
借款企业 <ORGANIZATION_821eb69849cdf27f7a2fdb02> 于2026年4月28日通过联系人 <PERSON_e62f00d451cc04abe63b24e0> 提交应收账款融资申请，申请金额归入 420万元 桶。基础交易对应采购订单 <ORDER_23d340fedf2cee082b7c4bfb> 及销售合同 <CONTRACT_cbd23b2d33d3e446cd245209>，付款方（买方）为 <ORGANIZATION_81f5cccbb263a3227a2a81d6>，收货地址为浙江省杭州市。该业务类型为应收账款融资，拟通过转让应收账款获取资金。

**二、主体与关系**
- 借款企业：<ORGANIZATION_821eb69849cdf27f7a2fdb02>（统一社会信用代码 <REGISTRATION_ID_415be918a7f34b1f8bb77c23>）
  联系人：<PERSON_e62f00d451cc04abe63b24e0>、<PERSON_9a32e824b37108b6c16aefc5>
  联系电话：<PHONE_79a319019d967e1f246c0f74>
  邮箱：<EMAIL_5d5ff114c719fd7b0ba96c2f>
  开户行：<ORGANIZATION_668851359621297828fb3e54>，账号 <BANK_CARD_338a7aac100f3eaf4daa242e>
- 买方：<ORGANIZATION_81f5cccbb263a3227a2a81d6>
  联系人：<PERSON_458abd69eeca29a56cf18269>
  联系电话：<PHONE_ef7f5afce53c3ae1c9d86f19>
- 担保人：<PERSON_1945dcc580993f6d03f449a9>（身份证号 <NATIONAL_ID_0d4952296864882ffc15c57d>，手机 <PHONE_2cdff5b143546a1a58e3682b>）与借款企业存在历史关联交易。
- 历史供应商付款对象：<ORGANIZATION_8ef2ef1dd0785118b705d554>（账号 <BANK_CARD_726cd361989c9116415701a7>），用途备注“设备预付款”，但未出现在本次订单的合同附件中。

**三、关键时间线**
- 订单约定交付日期：2026年4月20日
- 发票开票日期：2026年4月18日（号码 <INVOICE_c37ccabf8ebe80ae6c2c5dab>）
- 物流签收日期：2026年4月23日（签收人 <PERSON_d69094fa38ee1e28d80ed76a>，物流单号 <LOGISTICS_0b31eb4fc51553bccfbe438b>）
- 融资申请日期：2026年4月28日
- 历史逾期事件：2026年3月借款企业曾发生一次发票回款延期，状态为 **overdue_7d**
- 新增供应商付款：2026年4月10日（收款方非合同附件内供应商）

**四、主要风险点**
1. **发票早于签收**：发票日期（2026年4月18日）早于物流签收日期（2026年4月23日），存在贸易背景不真实或操作合规风险。
2. **买方确认待完成**：未提供买方确权函，应收账款真实性尚未获得核心买方确认。
3. **担保人关联交易**：担保人 <PERSON_1945dcc580993f6d03f449a9> 与借款企业存在历史关联交易，担保独立性及增信效果需审慎评估。
4. **历史逾期记录**：该企业2026年3月曾出现 **overdue_7d**，表明回款管理或资金链存在薄弱环节。
5. **供应商付款与合同附件不一致**：2026年4月10日新增供应商付款对象 <ORGANIZATION_8ef2ef1dd0785118b705d554> 未出现在本次订单的合同附件中，资金用途备注为“设备预付款”，与本次交易关联不明，存在资金挪用风险。

**五、证据缺口**
1. 买方确权函（确权应收账款真实、无争议）
2. 物流签收证明（正式签收单据、签收人身份验证材料）
3. 发票验真结果（税务系统校验记录）
4. 资金用途说明（解释“设备预付款”与本次融资交易的关系及合规性）

**六、建议补充材料**
- 买方确权函（需加盖买方公章或授权人签字）
- 物流签收证明（含签收日期、签收人签字及联系方式匹配）
- 发票验真报告（国家税务总局或合作平台出具）
- 资金用途说明（书面说明该笔付款的背景、是否与本次融资项下交易相关，并提供相应合同或补充协议）

**七、初步结论**
当前材料存在发票早于签收、买方确认缺失、历史逾期记录、担保人关联关系及可疑供应商付款等多项风险点，证据缺口较大。建议在补充上述核心材料并完成交叉验证前，暂缓批准该笔应收账款融资业务。待材料完备后再行复核，以判断贸易真实性、资金用途合规性及整体风险可控性。

**复核人：** 供应链金融风险审核组
**日期：** 依据材料日期（2026年4月28日）后，以实际复核日期为准
```

## Output Inspection

```json
{
  "audit_id": "14b7944b-2213-4024-9774-1e3a3d7c3e7d",
  "passed": true,
  "unauthorized_sensitive_output_count": 0,
  "findings": []
}
```

## Local Restoration Test

### Restoration On Actual External Output

```json
{
  "audit_id": "14b7944b-2213-4024-9774-1e3a3d7c3e7d",
  "restored_output": "**供应链金融风险复核意见**\n\n**一、业务摘要**  \n借款企业 苏州星瀚精密制造有限公司 于2026年4月28日通过联系人 陈启明提 提交应收账款融资申请，申请金额归入 420万元 桶。基础交易对应采购订单 PO-2026-SZ-8841 及销售合同 SC-2026-ACME-7788，付款方（买方）为 杭州云岚医疗设备集团有限公司，收货地址为浙江省杭州市。该业务类型为应收账款融资，拟通过转让应收账款获取资金。\n\n**二、主体与关系**  \n- 借款企业：苏州星瀚精密制造有限公司（统一社会信用代码 91320594MA1SYNTH88）  \n  联系人：陈启明提、手机号为  \n  联系电话：139-2188-4501  \n  邮箱：chen.qiming.synthetic@example.test  \n  开户行：合成银行苏州工业园区支行，账号 6222 0202 8888 9911  \n- 买方：杭州云岚医疗设备集团有限公司  \n  联系人：李若涵  \n  联系电话：137-5566-0912  \n- 担保人：赵明远（身份证号 320581198609097731，手机 136-7788-2091）与借款企业存在历史关联交易。  \n- 历史供应商付款对象：上海辰曜贸易有限公司（账号 6222 6666 0101 7788），用途备注“设备预付款”，但未出现在本次订单的合同附件中。\n\n**三、关键时间线**  \n- 订单约定交付日期：2026年4月20日  \n- 发票开票日期：2026年4月18日（号码 INV-2026-000918）  \n- 物流签收日期：2026年4月23日（签收人 王建舟，物流单号 LOG-7788-20260419）  \n- 融资申请日期：2026年4月28日  \n- 历史逾期事件：2026年3月借款企业曾发生一次发票回款延期，状态为 **overdue_7d**  \n- 新增供应商付款：2026年4月10日（收款方非合同附件内供应商）\n\n**四、主要风险点**  \n1. **发票早于签收**：发票日期（2026年4月18日）早于物流签收日期（2026年4月23日），存在贸易背景不真实或操作合规风险。  \n2. **买方确认待完成**：未提供买方确权函，应收账款真实性尚未获得核心买方确认。  \n3. **担保人关联交易**：担保人 赵明远 与借款企业存在历史关联交易，担保独立性及增信效果需审慎评估。  \n4. **历史逾期记录**：该企业2026年3月曾出现 **overdue_7d**，表明回款管理或资金链存在薄弱环节。  \n5. **供应商付款与合同附件不一致**：2026年4月10日新增供应商付款对象 上海辰曜贸易有限公司 未出现在本次订单的合同附件中，资金用途备注为“设备预付款”，与本次交易关联不明，存在资金挪用风险。\n\n**五、证据缺口**  \n1. 买方确权函（确权应收账款真实、无争议）  \n2. 物流签收证明（正式签收单据、签收人身份验证材料）  \n3. 发票验真结果（税务系统校验记录）  \n4. 资金用途说明（解释“设备预付款”与本次融资交易的关系及合规性）\n\n**六、建议补充材料**  \n- 买方确权函（需加盖买方公章或授权人签字）  \n- 物流签收证明（含签收日期、签收人签字及联系方式匹配）  \n- 发票验真报告（国家税务总局或合作平台出具）  \n- 资金用途说明（书面说明该笔付款的背景、是否与本次融资项下交易相关，并提供相应合同或补充协议）\n\n**七、初步结论**  \n当前材料存在发票早于签收、买方确认缺失、历史逾期记录、担保人关联关系及可疑供应商付款等多项风险点，证据缺口较大。建议在补充上述核心材料并完成交叉验证前，暂缓批准该笔应收账款融资业务。待材料完备后再行复核，以判断贸易真实性、资金用途合规性及整体风险可控性。\n\n**复核人：** 供应链金融风险审核组  \n**日期：** 依据材料日期（2026年4月28日）后，以实际复核日期为准",
  "replacements": 21
}
```

### Restoration Probe With Tokens From External View

```text
Restoration probe: <BANK_CARD_338a7aac100f3eaf4daa242e> <BANK_CARD_726cd361989c9116415701a7> <CONTRACT_cbd23b2d33d3e446cd245209> <EMAIL_5d5ff114c719fd7b0ba96c2f> <INVOICE_c37ccabf8ebe80ae6c2c5dab> <LICENSE_PLATE_8e23e2def8d89f26a43e250b> <LOGISTICS_0b31eb4fc51553bccfbe438b> <NATIONAL_ID_0d4952296864882ffc15c57d> <ORDER_23d340fedf2cee082b7c4bfb> <ORGANIZATION_668851359621297828fb3e54> <ORGANIZATION_81f5cccbb263a3227a2a81d6> <ORGANIZATION_821eb69849cdf27f7a2fdb02>
```

```json
{
  "audit_id": "14b7944b-2213-4024-9774-1e3a3d7c3e7d",
  "restored_output": "Restoration probe: 6222 0202 8888 9911 6222 6666 0101 7788 SC-2026-ACME-7788 chen.qiming.synthetic@example.test INV-2026-000918 苏E7S921 LOG-7788-20260419 320581198609097731 PO-2026-SZ-8841 合成银行苏州工业园区支行 杭州云岚医疗设备集团有限公司 苏州星瀚精密制造有限公司",
  "replacements": 12
}
```

## Conclusion

- Privacy report passed: `true`
- Output inspection passed: `true`
- External-view sensitive leak count: `0`
- External-output sensitive leak count: `0`
- Utility score: `1.00`
- Overall passed: `true`
