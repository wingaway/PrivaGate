# 外部模型 Adapter 边界

项目保留 `/v1/model-dispatch` 作为外部模型调用边界。当前默认实现不接入具体模型供应商。

## 原则

- adapter 只能接收 `external_view`。
- adapter 不能接收原始输入。
- adapter 不能接收本地 HMAC key。
- adapter 不能接收 token 映射表。
- adapter 输出必须经过 `/v1/inspect-output` 或同等本地输出检查。

## 当前状态

默认实现为 `DisabledModelAdapter`：

- 不发起任何网络请求。
- 不读取任何模型 API key。
- 返回 `dispatched=false`。

该设计确保隐私投影、证明报告、审计和输出检查可以独立运行。具体模型供应商接入只应发生在 adapter 实现层。
