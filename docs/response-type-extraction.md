# Axum 响应类型提取

## 功能概述

Axum 提取器现在可以从 handler 函数的返回值中自动解析 JSON 响应的数据结构。这使得生成的 OpenAPI 文档能够包含完整的响应模式信息。

## 支持的响应类型

### 1. 直接 Json 响应

```rust
async fn get_user() -> Json<User> {
    // ...
}
```

提取结果：`response_type = User`

### 2. Vec 响应

```rust
async fn list_users() -> Json<Vec<User>> {
    // ...
}
```

提取结果：`response_type = Vec<User>` (is_vec = true)

### 3. Result 包装的响应

```rust
async fn create_user() -> Result<Json<User>, String> {
    // ...
}
```

提取结果：从 `Ok` 类型中提取 `User`

### 4. 元组响应（带状态码）

```rust
async fn create_user() -> (StatusCode, Json<User>) {
    // ...
}
```

提取结果：从元组中查找并提取 `Json<User>` 中的 `User`

### 5. 简单类型响应

```rust
async fn health_check() -> &'static str {
    "OK"
}
```

提取结果：`response_type = str`

## 实现细节

响应类型解析在 `AxumVisitor::analyze_handlers()` 方法中进行，该方法会：

1. 遍历所有提取的路由
2. 查找对应的 handler 函数签名
3. 解析函数的返回类型
4. 处理各种包装类型（Json, Result, 元组等）
5. 提取最终的数据类型

## 解析流程

```
函数签名 -> 返回类型
    |
    v
parse_response_type()
    |
    v
parse_return_type()
    |
    +-- Reference (&T) -> 提取 T
    +-- Path (Json<T>) -> 提取 T
    +-- Result<T, E> -> 递归解析 T
    +-- Tuple -> 查找 Json<T> 并提取 T
    |
    v
extract_type_info() -> TypeInfo
```

## 测试覆盖

新功能包含以下测试：

- `test_json_response_type` - 测试 `Json<T>` 响应
- `test_result_json_response_type` - 测试 `Result<Json<T>, E>` 响应
- `test_tuple_response_with_json` - 测试元组响应
- `test_vec_response_type` - 测试 `Vec<T>` 响应
- `test_string_response_type` - 测试简单类型响应
- `test_response_type_extraction` - 集成测试

## 使用示例

参见 `examples/response_type_demo.rs` 获取完整的使用示例。

## 限制

1. 不支持 `impl IntoResponse` 类型（无法从 trait 推断具体类型）
2. 复杂的泛型嵌套可能需要进一步增强
3. 自定义响应类型需要实现相应的解析逻辑

## 未来改进

- 支持更多 Axum 响应类型（Html, Bytes 等）
- 改进元组响应的处理
- 支持自定义响应包装器
- 添加响应状态码的提取
