# apsis-book

[English](./README.md) | 简体中文

ApsisRS 的 Web 服务层。它在浏览器中呈现面向商业航天与卫星互联网的
ApsisRS 计算内核：概念文档、知识图谱、从库源码中提取的接口文档，以及一组
在线工具——用户给定参数，工具在这些内核上实际计算。

服务构建为单一二进制，其中嵌入了它所服务的全部资源。运行时不读取内容目录，
也不读取前端构建产物；二进制旁边不需要存在这两者。

## 线上服务

服务部署在 <https://apsis.focus1024.com/>，提供的就是本项目的内容：同样的文档、
图谱、接口目录与工具，出自同一份构建。它没有任何与特定部署相关之处——它所服务的
一切都内嵌在二进制里，因此从源码构建出的副本就是同一个服务。

线上实例默认以中文打开，可在顶栏切换到英文。

## 服务内容

| 板块 | 内容 | 路由前缀 |
|------|------|----------|
| 概念文档 | 关于时间尺度、参考系、元素格式、传播模型的文档，附各自的单位与适用范围 | `/api/docs` |
| 知识图谱 | 概念及其关系，以及库的 crate 结构与算法流程图 | `/api/graph` |
| 接口文档 | 服务所构建的 ApsisRS 各层的公共接口，从源码文档注释中提取 | `/api/api` |
| 在线工具 | TLE 解析、SGP4 传播、轨迹采样、时间尺度换算、坐标换算、参考系旋转、开普勒换算与霍曼转移规划 | `/api/tools` |

## 设计约束

服务遵循它所属工作区的约束。

- **一个运行时依赖。** `[dependencies]` 中只有 `apsis-rs` 一项，且是路径依赖。
  HTTP 服务器、JSON 编解码器、路由器、Markdown 渲染器与图模型，全部基于标准库
  编写。
- **不使用 `unsafe`。** 本 crate 不含任何 unsafe 代码块。
- **请求路径中不 panic。** 畸形请求、非法参数或计算失败都产生一个响应，而不是
  panic。处理请求的工作线程会把处理函数中的 panic 捕获下来作为最后一道防线，
  并返回服务器错误。
- **注释自足。** 每条注释都解释代码、算法、接口或项目本身；没有注释指向私有
  文档、指令或计划。

## 构建

必须先构建前端，发布二进制才能嵌入其产物；发布构建会自动完成这一步。

每个构建出的资源，其文件名都带有内容哈希，因此替换二进制就替换了这些文件名。
浏览器因而不可能握着旧包：新包是在它从未见过的 URL 下请求的。入口页面以
`no-cache, must-revalidate` 提供，带哈希的资源则以
`public, max-age=31536000, immutable` 提供。服务不注册 service worker。

```sh
# 开发：调试构建嵌入 frontend/dist 中已有的内容，不运行前端构建。
cargo build

# 发布：build.rs 构建前端并嵌入结果。
cargo build --release
```

二进制位于 `target/release/apsis-book`。

环境变量：

| 变量 | 作用 |
|------|------|
| `APSIS_BOOK_HOST` | 绑定的网络接口，默认 `127.0.0.1` |
| `APSIS_BOOK_PORT` | 绑定的端口，默认 `8080` |
| `APSIS_BOOK_ADDRESS` | 以 `HOST:PORT` 表示的绑定地址，覆盖上面两项 |
| `APSIS_BOOK_WORKERS` | 工作线程数，默认 4 |
| `APSIS_BOOK_SKIP_FRONTEND` | 设为 `1` 时在发布构建中跳过前端构建 |
| `APSIS_BOOK_FRONTEND_DIR` | 前端目录，默认 `frontend` |

命令行选项优先于环境变量：

```sh
# 在 8080 端口绑定所有网络接口。
apsis-book --host 0.0.0.0 --port 8080

# 合并写法等价。
apsis-book --address 0.0.0.0:8080
```

`--host` 与 `--port` 可以同时给出。`--address` 不能与它们中的任何一个合用，
因为二者生效的先后顺序没有定义。主机可以是字面地址、方括号中的 IPv6 字面量，
或由操作系统解析的名字。

### 语言

界面为双语，默认以中文打开。读者用顶栏中的控件切换到英文；该选择保存在浏览器
中，对所有页面生效。

后端为每一条面向读者的字符串同时保存两种语言，因此切换无需请求即可生效。
一篇概念文档的英文写为 `content/concepts/<id>.md`，中文译为
`content/concepts/<id>.zh.md`。没有译文的文档对所有读者显示英文，并附一条说明。

## 运行

```sh
./target/release/apsis-book --address 127.0.0.1:8080
```

### Docker

镜像只携带二进制，别无其他，因为二进制携带了它所提供的一切。构建上下文是本目录
的上一级，因为该 crate 以路径依赖引用 `apsis-rs`：

```sh
# 在工作区根目录执行。
docker build -f apsis-book/Dockerfile -t apsis-book:0.1.0 .
docker run --rm -p 8080:8080 apsis-book:0.1.0
```

命令中显式给出主机。程序的默认值是回环地址，那里它只回应自己，因此不带这一项启动
的容器不会向任何人提供服务。

`docker-compose.yml` 做同样的事，并带上长时间运行的服务所需要的设置：除非被停止
否则自动重启，因文件系统不被写入而以只读方式挂载，去掉取得更多权限的能力，并且
每三十秒探测一次服务。

```sh
cd apsis-book
docker compose up --build
```

该探针用 shell 写成而不是用某个工具，因为镜像不携带 HTTP 客户端：它只携带一个
二进制，为探针添加客户端会破坏这一点。`bash` 打开连接、写入请求并读回状态行。

镜像本身没有 `HEALTHCHECK`。探测 `GET /health` 是编排系统应当做的事，而在容器内部
探测意味着给一个不需要其他工具的体积里加上 HTTP 客户端。Kubernetes 的探针无须
它即可表达同一件事：

```yaml
livenessProbe:
  httpGet: { path: /health, port: 8080 }
```

镜像为其构建所在的架构而构建。多架构镜像需要每个架构各一个构建器；Dockerfile
顶部的 `ARG` 就是各版本号，用 `--build-arg RUST_VERSION=` 可更改其中之一。

打开 `http://127.0.0.1:8080`。JSON 接口不依赖前端即可访问：

- `GET /health`
- `GET /api/docs`、`GET /api/docs/<id>`、`GET /api/docs/search?q=`
- `GET /api/graph`、`GET /api/graph/crates`、`GET /api/graph/flows`、`GET /api/graph/<flow>`
- `GET /api/api`、`GET /api/api/crates/<name>`、`GET /api/api/items/<name>`
- `GET /api/tools`、`GET /api/tools/<id>`、`POST /api/tools/<id>/run`

一次工具调用接受一个参数 JSON 对象，返回一个已整形的结果：

```sh
curl -s http://127.0.0.1:8080/api/tools/hohmann-transfer/run \
  -H 'content-type: application/json' \
  -d '{"from_radius": 6778, "to_radius": 42164}'
```

## 内容布局

内容是任何单一库所实现之物的一个超集，分为两组。

**概念 / Concepts** 描述这个领域。各层沿用库的分层，因为那是读者接触这个学科的
顺序：

| 层 | 主题 |
|----|------|
| 导论 | 范围、组织方式，以及如何阅读一页 |
| 基础层 | 单位、量纲、数值方法、误差 |
| 物理层 | 时间与坐标系是两个彼此无关的主题，故并列于此 |
| 轨道层 | 根数、元素格式、摄动、传播器、轨道机动 |
| 射频与传播 | 多普勒、链路预算、介质、干扰 |
| 星座与网络 | 星座、可见性、拓扑、切换、路由 |
| 运行层 | 遥测、跟踪、指令、规划 |
| 仿真层 | 场景、事件，以及结果的含义 |

**实现 / Implementation** 描述某一个特定的库对上述内容的实现情况，包括它如何
组织、以及它的算法从何而来。把它们分开，正是概念集保持独立的原因：一篇关于
链路预算的页面不会因为某个库变了而改变。

文档归属于它所在目录所命名的层，因此新增一篇文档就是在它所属的层下创建文件，
并在 `chapters.json` 中列出。

## 重新生成接口目录

接口文档由 ApsisRS 源码生成，而非手工撰写。那里的文档注释先写英文、再写中文，
提取器把两半拆开，因此两种语言都来自代码旁的同一段注释，谁也不会落后于它所描述的
代码。源码变化时在本目录运行提取脚本，
然后重新构建：

```sh
scripts/extract_api.sh
cargo build --release
```

脚本默认读取 `../apsis-rs` 的工作区；设置 `APSIS_RS_WORKSPACE` 可指向别处的
检出。它需要 Python 3.8 或更高版本。Rust 构建不运行它，也不依赖 Python。

## 编辑内容

| 路径 | 内容 |
|------|------|
| `content/concepts/chapters.json` | 分组、各层与阅读顺序，含双语标题 |
| `content/concepts/<layer>/*.md` | 文档，每层一个目录；目录给出层，文件名给出标识 |
| `content/concepts/<layer>/*.zh.md` | 中文译文，文件名对应其所译的文件 |
| `content/graph/concepts.json` | 概念图谱 |
| `content/flows/*.json` | 算法流程图 |
| `content/api.json` | 生成的接口目录，含两种语言；不要手工编辑 |

公式写在美元符号之间：独占一行用 `$$...$$`，行内用 `$...$`。这是各类
Markdown 数学渲染器都识别的写法，也是本文档所采用的写法：公式是数学，
代码块不是放它的地方。服务把该记法翻译为 MathML，由浏览器自行排版。

文档头部声明 `title`、`summary`、`tags` 与 `prerequisites`。标识与层不由头部
声明：标识即文件名，层即目录，二者无法与目录结构相抵触。交叉引用写作
`[[id]]` 或 `[[id|label]]`，渲染为链接，前端无须整页加载即可路由。

## 目录结构

```text
apsis-book/
├── build.rs                  嵌入前端构建产物与文本内容
├── Cargo.toml                一个运行时依赖：apsis-rs
├── content/                  撰写的文档、图谱与生成的目录
│   └── concepts/<layer>/     文档，每层一个目录
├── Dockerfile                发布镜像，分三个阶段
├── Dockerfile.dockerignore   镜像构建不接收的内容
├── docker-compose.yml        以长期运行所需的设置来运行镜像
├── docs/                     设计说明
│   ├── design.md             服务为何如此构建
│   └── design.zh.md          同上，中文
├── frontend/                 Svelte 与 TypeScript，由 Vite 构建
│   ├── src/lib/              API 客户端、路由器、组件、格式化
│   ├── src/pages/            每个路由一个组件
│   └── dist/                 构建产物，由 build.rs 嵌入
├── scripts/extract_api.sh    接口目录生成器
├── scripts/render_audit.mjs  渲染每条路由并报告问题
├── src/
│   ├── api/                  路由器与 JSON 载荷
│   ├── docs/                 内容、Markdown、文档、接口目录
│   ├── embedded/             对嵌入式前端资源的访问
│   ├── graph/                知识图谱与流程图
│   ├── http/                 请求、响应与服务器
│   ├── tools/                工具注册表、处理函数、结果形状
│   ├── json.rs               JSON 编解码器
│   ├── lib.rs                各模块，以库的形式
│   ├── testkit.rs            测试所需的少数内部项
│   ├── main.rs               进程入口
│   └── state.rs              解析后的内容，供每个工作线程共享
└── tests/                   全部测试；`src/` 中没有
    ├── service.rs            针对构建出的二进制的端到端测试
    └── <module>.rs           各模块的测试
```

## 测试

```sh
cargo test          # 单元测试与端到端测试
cargo clippy --all-targets
cargo fmt --check
cd frontend && npm run check
```

发版还会渲染界面来检查：`node scripts/render_audit.mjs` 会在两种语言、两种主题与
两种宽度下打开每一条路由，并报告页面未加载、正文近乎空白、横向溢出，或公式丢字符
这四类问题。它需要 Chrome 与一份发布构建，把发现写入 `/tmp/matrix.jsonl`，且被
中断后会续跑而不是从头再来。

端到端测试在一个临时端口上启动构建出的二进制，并以普通 TCP 请求与之通信。

## 许可协议

GNU 通用公共许可证，第 3 版或更高版本，且仅此一种。见 `LICENSE`。

本服务依赖 `apsis-rs`，后者以 MIT 或 Apache-2.0 提供。这两种协议都允许在 GPL
作品内再分发，因此二者可以按本协议一并分发。本服务不提供任何其他许可。