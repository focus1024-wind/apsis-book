#
# The release image of apsis-book.
#
# The service embeds every asset it serves, so the image carries one binary
# and nothing else: no content directory, no frontend build, no runtime
# dependency beyond the C library. The build is in three stages, each of
# which needs a toolchain the next one does not.
#
# apsis-book 的发布镜像。
#
# 服务把它所提供的一切资源都内嵌其中，因此镜像只携带一个二进制，别无其他：没有
# 内容目录，没有前端构建产物，除 C 库外没有运行期依赖。构建分三个阶段，每个阶段
# 所需的工具链下一个阶段并不需要。
#
# The build context is the directory above this one, because the crate depends
# on `apsis-rs` by path. From the workspace root:
#
#     docker build -f apsis-book/Dockerfile -t apsis-book:0.1.0 .
#
# 构建上下文是本目录的上一级，因为该 crate 以路径依赖引用 `apsis-rs`。在工作区
# 根目录执行上面那条命令。
#
# Run it with the listener bound to every interface, because the default is
# the loopback address and a container that listens only there cannot be
# reached from outside itself:
#
#     docker run --rm -p 8080:8080 apsis-book:0.1.0
#
# 运行时把监听绑定到所有网络接口，因为默认是回环地址，而只监听回环的容器无法从
# 自身之外被访问。
#
#   docker run --rm -p 8080:8080 apsis-book:0.1.0

ARG NODE_VERSION=22
ARG RUST_VERSION=1.85
ARG DEBIAN_RELEASE=bookworm

# The frontend, built once and handed to the binary.
#
# 前端，构建一次后交给二进制。

FROM node:${NODE_VERSION}-${DEBIAN_RELEASE}-slim AS frontend

WORKDIR /build/frontend

# The manifests are copied on their own so that a change to a source file
# does not reinstall the dependencies. `npm ci` installs from the lock file,
# so the image gets the versions that were tested.
#
# 单独复制清单文件，使源码改动不会触发重新安装依赖。`npm ci` 依据 lock 文件安装，
# 因此镜像得到的是经过测试的那些版本。
COPY apsis-book/frontend/package.json apsis-book/frontend/package-lock.json ./
RUN npm ci

COPY apsis-book/frontend/ ./
RUN npm run build

# The binary.
#
# 二进制。

FROM rust:${RUST_VERSION}-${DEBIAN_RELEASE} AS builder

# The frontend was built in its own stage, so the build script is told not to
# build it again; that would need Node in this stage. It embeds what it finds
# in `frontend/dist`, which the next line places.
#
# 前端已在其自身的阶段构建完毕，因此告知构建脚本不要再次构建它——那会需要本阶段
# 安装 Node。它嵌入在 `frontend/dist` 中找到的内容，下一行正是把它放到那里。
ENV APSIS_BOOK_SKIP_FRONTEND=1

WORKDIR /build

# The crate depends on `apsis-rs` by the relative path `../apsis-rs`, so the
# workspace keeps its shape inside the image.
#
# 该 crate 以相对路径 `../apsis-rs` 依赖 `apsis-rs`，因此工作区在镜像内保持原样。
COPY apsis-rs/ ./apsis-rs/
COPY apsis-book/ ./apsis-book/
COPY --from=frontend /build/frontend/dist ./apsis-book/frontend/dist

# The image provides the toolchain this project builds with, and the
# toolchain file asks for the `stable` channel by name, which rustup would
# then fetch over the network. The file is removed here so that the image's
# own toolchain is the one used, and the build needs no network beyond the
# frontend's dependencies.
#
# 镜像提供了项目构建所用的工具链，而工具链文件按名字要求 `stable` 通道，rustup
# 会因此去网络上拉取。这里移除该文件，使镜像自带的工具链成为所用者，构建除前端的
# 依赖之外不需要网络。
RUN rm -f apsis-book/rust-toolchain.toml

# `--locked` fails rather than quietly updating a dependency, so the image is
# built from the versions the lock file records.
#
# `--locked` 会直接失败而不是悄悄更新依赖，因此镜像由 lock 文件所记录的版本构建。
RUN cargo build --release --locked --manifest-path apsis-book/Cargo.toml

# The runtime. It holds the binary and the C library the binary links
# against, and nothing that a build needed.
#
# 运行阶段。它持有二进制以及该二进制链接的 C 库，不含任何构建所需之物。

FROM debian:${DEBIAN_RELEASE}-slim AS runtime

LABEL org.opencontainers.image.title="apsis-book" \
      org.opencontainers.image.description="The documentation and tool service of ApsisRS" \
      org.opencontainers.image.licenses="GPL-3.0-or-later"

COPY --from=builder /build/apsis-book/target/release/apsis-book /usr/local/bin/apsis-book

# The process runs as an unprivileged user. The numeric identifier of `nobody`
# is used rather than a new account, because creating one would mean carrying
# `useradd` and its package in an image that needs no other tool.
#
# 进程以非特权用户运行。这里使用 `nobody` 的数字标识而不是新建账户，因为新建账户
# 意味着在一个不需要其他工具的体积里带上 `useradd` 及其软件包。
USER 65534:65534

# Documentation only; the port is set by the command below.
#
# 仅作说明；端口由下面的命令设定。
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/apsis-book"]

# The host is given explicitly. The program defaults to the loopback address,
# where it answers only itself, so a container started without this would
# serve nothing to anyone.
#
# 显式给出主机。程序的默认值是回环地址，那里它只回应自己，因此不带这一项启动的
# 容器不会向任何人提供服务。
CMD ["--host", "0.0.0.0", "--port", "8080"]
