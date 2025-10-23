# 需求文档

## 简介

这是一个命令行工具，用于自动从 Rust Web 项目中生成 OpenAPI 文档。该工具通过遍历项目目录，分析路由配置和处理函数，自动提取 API 端点信息并生成标准的 OpenAPI 规范文档（支持 YAML 和 JSON 格式）。目前支持两个主流的 Rust Web 框架：Axum 和 Actix-Web。

## 需求

### 需求 1：项目目录扫描

**用户故事：** 作为开发者，我希望工具能够扫描指定的 Rust 项目目录，以便自动发现所有的路由定义文件。

#### 验收标准

1. WHEN 用户提供一个有效的项目目录路径 THEN 系统 SHALL 递归遍历该目录及其所有子目录
2. WHEN 系统遍历目录时 THEN 系统 SHALL 识别所有 .rs 文件
3. WHEN 遇到无法访问的目录或文件 THEN 系统 SHALL 记录警告信息并继续处理其他文件
4. WHEN 用户提供的路径不存在 THEN 系统 SHALL 返回清晰的错误信息

### 需求 2：Axum 框架路由解析

**用户故事：** 作为使用 Axum 框架的开发者，我希望工具能够识别和解析 Axum 的路由配置，以便生成准确的 API 文档。

#### 验收标准

1. WHEN 系统检测到 Axum 路由定义（如 `Router::new()`, `.route()`, `.nest()`）THEN 系统 SHALL 提取路由路径和 HTTP 方法
2. WHEN 路由使用 `.get()`, `.post()`, `.put()`, `.delete()`, `.patch()` 等方法链 THEN 系统 SHALL 正确识别对应的 HTTP 方法
3. WHEN 路由包含路径参数（如 `/users/:id`）THEN 系统 SHALL 提取参数名称和位置
4. WHEN 路由使用 `.nest()` 进行路径嵌套 THEN 系统 SHALL 正确组合完整的路由路径
5. WHEN 处理函数使用提取器（如 `Json<T>`, `Path<T>`, `Query<T>`）THEN 系统 SHALL 识别请求体和参数的数据结构

### 需求 3：Actix-Web 框架路由解析

**用户故事：** 作为使用 Actix-Web 框架的开发者，我希望工具能够识别和解析 Actix-Web 的路由配置，以便生成准确的 API 文档。

#### 验收标准

1. WHEN 系统检测到 Actix-Web 路由宏（如 `#[get]`, `#[post]`, `#[put]`, `#[delete]`, `#[patch]`）THEN 系统 SHALL 提取路由路径和 HTTP 方法
2. WHEN 路由使用 `.service()` 或 `.route()` 配置 THEN 系统 SHALL 正确解析路由定义
3. WHEN 路由包含路径参数（如 `/users/{id}`）THEN 系统 SHALL 提取参数名称和位置
4. WHEN 使用 `.scope()` 进行路径分组 THEN 系统 SHALL 正确组合完整的路由路径
5. WHEN 处理函数使用提取器（如 `web::Json<T>`, `web::Path<T>`, `web::Query<T>`）THEN 系统 SHALL 识别请求体和参数的数据结构

### 需求 4：数据结构分析

**用户故事：** 作为开发者，我希望工具能够分析路由处理函数中使用的数据结构，以便在 OpenAPI 文档中生成完整的 schema 定义。

#### 验收标准

1. WHEN 处理函数使用自定义结构体作为请求或响应类型 THEN 系统 SHALL 查找该结构体的定义
2. WHEN 结构体包含字段 THEN 系统 SHALL 提取字段名称、类型和可选性
3. WHEN 结构体使用 Serde 派生宏（如 `#[derive(Serialize, Deserialize)]`）THEN 系统 SHALL 识别序列化配置
4. WHEN 结构体字段使用 Serde 属性（如 `#[serde(rename)]`, `#[serde(skip)]`）THEN 系统 SHALL 应用相应的序列化规则
5. WHEN 遇到嵌套结构体 THEN 系统 SHALL 递归解析所有相关的类型定义

### 需求 5：OpenAPI 文档生成

**用户故事：** 作为开发者，我希望工具能够生成符合 OpenAPI 3.0 规范的文档，以便与其他工具和服务集成。

#### 验收标准

1. WHEN 系统完成路由解析 THEN 系统 SHALL 生成符合 OpenAPI 3.0 规范的文档结构
2. WHEN 生成文档时 THEN 系统 SHALL 包含所有发现的 API 端点及其路径、方法和参数
3. WHEN 端点包含请求体 THEN 系统 SHALL 在文档中生成对应的 requestBody schema
4. WHEN 端点包含路径或查询参数 THEN 系统 SHALL 在文档中生成对应的 parameters 定义
5. WHEN 发现数据结构 THEN 系统 SHALL 在 components/schemas 部分生成 schema 定义
6. WHEN 无法确定某些信息（如响应类型）THEN 系统 SHALL 使用合理的默认值或占位符

### 需求 6：输出格式支持

**用户故事：** 作为开发者，我希望能够选择输出 YAML 或 JSON 格式的 OpenAPI 文档，以便适应不同的使用场景。

#### 验收标准

1. WHEN 用户指定输出格式为 YAML THEN 系统 SHALL 生成格式正确的 YAML 文件
2. WHEN 用户指定输出格式为 JSON THEN 系统 SHALL 生成格式正确且缩进美观的 JSON 文件
3. WHEN 用户未指定输出格式 THEN 系统 SHALL 默认生成 YAML 格式
4. WHEN 用户指定输出文件路径 THEN 系统 SHALL 将文档写入指定位置
5. WHEN 未指定输出文件路径 THEN 系统 SHALL 将文档输出到标准输出

### 需求 7：命令行界面

**用户故事：** 作为开发者，我希望通过简单直观的命令行参数使用该工具，以便快速生成 API 文档。

#### 验收标准

1. WHEN 用户运行工具时 THEN 系统 SHALL 接受项目目录路径作为必需参数
2. WHEN 用户提供 `--format` 或 `-f` 参数 THEN 系统 SHALL 使用指定的输出格式（yaml 或 json）
3. WHEN 用户提供 `--output` 或 `-o` 参数 THEN 系统 SHALL 将结果写入指定文件
4. WHEN 用户提供 `--framework` 或 `-w` 参数 THEN 系统 SHALL 仅解析指定框架的路由（axum 或 actix-web）
5. WHEN 用户提供 `--help` 或 `-h` 参数 THEN 系统 SHALL 显示使用说明和所有可用选项
6. WHEN 用户提供 `--version` 或 `-v` 参数 THEN 系统 SHALL 显示工具版本信息
7. WHEN 命令行参数无效 THEN 系统 SHALL 显示清晰的错误信息和使用提示

### 需求 8：错误处理和日志

**用户故事：** 作为开发者，我希望工具能够提供清晰的错误信息和处理进度反馈，以便了解工具的运行状态和排查问题。

#### 验收标准

1. WHEN 发生错误时 THEN 系统 SHALL 显示描述性的错误信息，包含错误类型和位置
2. WHEN 处理大型项目时 THEN 系统 SHALL 显示进度信息（如已处理的文件数量）
3. WHEN 遇到无法解析的代码结构 THEN 系统 SHALL 记录警告信息并继续处理
4. WHEN 用户启用详细模式（`--verbose`）THEN 系统 SHALL 输出详细的调试信息
5. WHEN 解析完成时 THEN 系统 SHALL 显示摘要信息（如发现的端点数量、生成的 schema 数量）
