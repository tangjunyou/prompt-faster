# Prompt Faster 后端 Dockerfile
FROM rust:1.85-slim-bookworm

WORKDIR /app

# 安装系统依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 复制项目文件
COPY . .

# 构建项目（开发模式）
RUN cargo build

# 创建数据目录
RUN mkdir -p data

# 暴露端口
EXPOSE 3000

# 启动命令
CMD ["cargo", "run"]
