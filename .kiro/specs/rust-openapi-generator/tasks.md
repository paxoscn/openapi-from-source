# 实现计划

- [x] 1. 初始化项目结构和核心依赖
  - 创建 Cargo 项目，配置 Cargo.toml 添加所有必需依赖（syn, serde, serde_json, serde_yaml, clap, walkdir, anyhow, log, env_logger）
  - 创建模块目录结构（cli, scanner, parser, detector, extractor, type_resolver, schema_generator, openapi_builder, serializer）
  - 在 main.rs 中设置基本的程序入口和日志初始化
  - _需求: 7.1, 8.4_

- [x] 2. 实现 CLI 模块和参数解析
  - 在 cli.rs 中定义 CliArgs 结构体和相关枚举（OutputFormat, Framework）
  - 使用 clap 实现命令行参数解析，支持 --format, --output, --framework, --verbose, --help, --version 参数
  - 实现参数验证逻辑，确保项目路径存在且有效
  - 实现 run 函数框架，协调整体流程（暂时返回占位符结果）
  - _需求: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7_

- [x] 3. 实现文件扫描器
  - 在 scanner.rs 中创建 FileScanner 结构体和 ScanResult 结构体
  - 实现 scan 方法，使用 walkdir 递归遍历目录
  - 过滤出所有 .rs 文件，跳过 target 目录和隐藏目录
  - 实现错误处理，记录无法访问的目录为警告
  - _需求: 1.1, 1.2, 1.3, 1.4_

- [x] 3.1 为文件扫描器编写单元测试
  - 创建临时测试目录结构
  - 测试正常扫描、空目录、嵌套目录等场景
  - _需求: 1.1, 1.2_

- [x] 4. 实现 AST 解析器
  - 在 parser.rs 中创建 AstParser 和 ParsedFile 结构体
  - 实现 parse_file 方法，使用 syn::parse_file 解析单个 Rust 文件
  - 实现 parse_files 方法，批量解析文件并收集错误
  - 对解析失败的文件记录警告并继续处理其他文件
  - _需求: 1.2, 8.3_

- [x] 4.1 为 AST 解析器编写单元测试
  - 测试有效和无效 Rust 代码的解析
  - 测试错误处理和警告记录
  - _需求: 1.2_

- [x] 5. 实现框架检测器
  - 在 detector.rs 中创建 FrameworkDetector 和 DetectionResult 结构体
  - 实现 detect 方法，通过分析 use 语句检测 axum 和 actix-web 的使用
  - 查找 "use axum::" 和 "use actix_web::" 模式
  - 返回检测到的所有框架列表
  - _需求: 7.4_

- [x] 5.1 为框架检测器编写单元测试
  - 测试 Axum 项目检测
  - 测试 Actix-Web 项目检测
  - 测试混合项目和无框架项目
  - _需求: 7.4_

- [x] 6. 定义路由提取器接口和数据结构
  - 在 extractor/mod.rs 中定义 RouteExtractor trait
  - 创建 RouteInfo, HttpMethod, Parameter, ParameterLocation, TypeInfo 等核心数据结构
  - 实现这些结构的基本方法（如 Debug, Clone 等派生宏）
  - _需求: 2.1, 3.1_

- [x] 7. 实现 Axum 路由提取器
  - 在 extractor/axum.rs 中创建 AxumExtractor 结构体
  - 实现 RouteExtractor trait 的 extract_routes 方法
  - 实现 find_router_definitions 方法，查找 Router::new() 和相关方法链
  - 实现 parse_route_method 方法，解析 .route(), .get(), .post() 等方法调用
  - 提取路由路径和 HTTP 方法
  - _需求: 2.1, 2.2_

- [x] 7.1 实现 Axum 路径参数和嵌套路由处理
  - 解析路径中的参数（如 "/:id"）
  - 处理 .nest() 方法，正确组合嵌套路径
  - _需求: 2.3, 2.4_

- [x] 7.2 实现 Axum 处理函数分析
  - 实现 extract_handler_info 方法，获取处理函数的签名
  - 实现 parse_extractors 方法，识别 Json<T>, Path<T>, Query<T> 等提取器
  - 从提取器中提取类型信息
  - _需求: 2.5_

- [x] 7.3 为 Axum 提取器编写单元测试
  - 测试简单路由提取
  - 测试嵌套路由和路径参数
  - 测试各种提取器识别
  - _需求: 2.1, 2.2, 2.3, 2.4, 2.5_

- [x] 8. 实现 Actix-Web 路由提取器
  - 在 extractor/actix.rs 中创建 ActixExtractor 结构体
  - 实现 RouteExtractor trait 的 extract_routes 方法
  - 实现 find_route_macros 方法，查找 #[get], #[post] 等路由宏
  - 实现 parse_route_macro 方法，从宏属性中提取路径和 HTTP 方法
  - _需求: 3.1, 3.2_

- [x] 8.1 实现 Actix-Web 路径参数和作用域处理
  - 解析路径中的参数（如 "/{id}"）
  - 查找 .scope() 配置，正确组合作用域路径
  - _需求: 3.3, 3.4_

- [x] 8.2 实现 Actix-Web 处理函数分析
  - 实现 parse_extractors 方法，识别 web::Json<T>, web::Path<T>, web::Query<T> 等提取器
  - 从提取器中提取类型信息
  - _需求: 3.5_

- [x] 8.3 为 Actix-Web 提取器编写单元测试
  - 测试路由宏解析
  - 测试作用域和路径参数
  - 测试各种提取器识别
  - _需求: 3.1, 3.2, 3.3, 3.4, 3.5_

- [x] 9. 实现类型解析器基础功能
  - 在 type_resolver.rs 中创建 TypeResolver, ResolvedType, TypeKind 等结构体
  - 实现 new 方法，初始化类型解析器并建立文件索引
  - 实现 find_struct_definition 方法，在所有解析的文件中查找结构体定义
  - 实现基本的 resolve_type 方法框架
  - _需求: 4.1, 4.2_

- [x] 9.1 实现结构体字段解析
  - 在 resolve_type 中实现结构体字段的提取
  - 创建 StructDef 和 FieldDef 结构体
  - 解析字段名称、类型和可选性（Option<T>）
  - _需求: 4.2_

- [x] 9.2 实现 Serde 属性解析
  - 实现 parse_serde_attributes 方法
  - 识别 #[serde(rename = "...")], #[serde(skip)], #[serde(flatten)] 等属性
  - 将 Serde 配置应用到字段定义中
  - _需求: 4.3, 4.4_

- [x] 9.3 实现递归类型解析
  - 处理嵌套结构体，递归解析引用的类型
  - 实现类型缓存，避免重复解析
  - 处理循环引用（记录警告并使用占位符）
  - _需求: 4.5_

- [x] 9.4 为类型解析器编写单元测试
  - 测试基本类型识别
  - 测试结构体解析
  - 测试 Serde 属性处理
  - 测试递归和嵌套类型
  - _需求: 4.1, 4.2, 4.3, 4.4, 4.5_

- [x] 10. 实现 Schema 生成器
  - 在 schema_generator.rs 中创建 SchemaGenerator 和 Schema 结构体
  - 实现 new 方法，接收 TypeResolver
  - 实现基本类型到 OpenAPI schema 的映射（String, i32, bool 等）
  - _需求: 5.5_

- [x] 10.1 实现复杂类型的 Schema 生成
  - 实现 Vec<T> 到 array schema 的转换
  - 实现 Option<T> 的处理（标记为非必需）
  - 实现自定义结构体到 schema 引用的转换
  - _需求: 5.5_

- [x] 10.2 实现参数 Schema 生成
  - 实现 generate_parameter_schema 方法
  - 为路径参数、查询参数生成对应的 schema
  - _需求: 5.4_

- [x] 10.3 为 Schema 生成器编写单元测试
  - 测试基本类型映射
  - 测试复杂类型转换
  - 测试参数 schema 生成
  - _需求: 5.4, 5.5_

- [x] 11. 实现 OpenAPI 构建器
  - 在 openapi_builder.rs 中创建 OpenApiBuilder, PathItem, Operation 等结构体
  - 实现 new 方法，初始化默认的 Info 信息
  - 实现 add_route 方法，将 RouteInfo 转换为 OpenAPI Operation
  - 为每个路由生成 parameters 和 requestBody 定义
  - _需求: 5.1, 5.2, 5.3, 5.4_

- [x] 11.1 实现 Components 和 Schema 集成
  - 在 OpenApiBuilder 中维护 Components 结构
  - 集成 SchemaGenerator，收集所有引用的 schema 定义
  - 确保 schema 引用的一致性
  - _需求: 5.5_

- [x] 11.2 实现响应定义生成
  - 为每个操作生成默认的响应定义
  - 如果能推断响应类型，生成对应的 schema
  - 否则使用通用的成功响应占位符
  - _需求: 5.6_

- [x] 11.3 实现 build 方法
  - 实现 build 方法，构建最终的 OpenApiDocument
  - 确保文档结构符合 OpenAPI 3.0 规范
  - _需求: 5.1_

- [x] 11.4 为 OpenAPI 构建器编写单元测试
  - 测试路由添加
  - 测试 schema 收集
  - 测试文档结构的正确性
  - _需求: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 12. 实现序列化器
  - 在 serializer.rs 中实现 serialize_yaml 函数，使用 serde_yaml
  - 实现 serialize_json 函数，使用 serde_json 并配置美化输出
  - 实现 write_to_file 函数，将内容写入指定文件
  - 处理序列化和 IO 错误
  - _需求: 6.1, 6.2, 6.4, 6.5_

- [x] 12.1 为序列化器编写单元测试
  - 测试 YAML 序列化
  - 测试 JSON 序列化
  - 测试文件写入
  - _需求: 6.1, 6.2, 6.4_

- [x] 13. 实现错误类型和处理
  - 创建 error.rs 模块，定义 Error 枚举
  - 实现各种错误类型（IoError, ParseError, InvalidArgument 等）
  - 实现 Display trait，提供清晰的错误信息
  - 实现 From trait，方便错误转换
  - _需求: 8.1_

- [x] 14. 集成所有模块到主流程
  - 在 cli.rs 的 run 函数中集成所有模块
  - 实现完整的处理流程：扫描 → 解析 → 检测框架 → 提取路由 → 解析类型 → 生成 schema → 构建文档 → 序列化输出
  - 添加进度日志输出
  - _需求: 8.2_

- [x] 14.1 实现输出格式选择逻辑
  - 根据用户指定的格式选择序列化方法
  - 实现默认格式（YAML）的逻辑
  - 处理输出到文件或标准输出
  - _需求: 6.3, 6.5_

- [x] 14.2 实现框架过滤逻辑
  - 如果用户指定了框架，只使用对应的提取器
  - 如果未指定，使用检测到的所有框架
  - 如果未检测到框架且用户未指定，返回错误
  - _需求: 7.4_

- [x] 14.3 添加详细的日志和进度信息
  - 在关键步骤添加 INFO 级别日志
  - 显示处理进度（已处理文件数、发现的路由数等）
  - 在 verbose 模式下输出 DEBUG 信息
  - 在处理完成后显示摘要信息
  - _需求: 8.2, 8.4, 8.5_

- [x] 15. 创建集成测试
  - 在 tests/ 目录创建 integration_test.rs
  - 创建 tests/fixtures/ 目录，包含示例 Axum 和 Actix-Web 项目代码
  - 编写端到端测试，验证完整的文档生成流程
  - 验证生成的 OpenAPI 文档的结构和内容
  - _需求: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 16. 完善文档和使用示例
  - 创建 README.md，包含项目介绍、安装说明和使用示例
  - 添加命令行使用示例
  - 创建 CHANGELOG.md 记录版本历史
  - 在代码中添加必要的文档注释
  - _需求: 7.5_
