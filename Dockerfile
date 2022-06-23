FROM ubuntu:20.04
RUN sed -i s/archive.ubuntu.com/mirrors.ustc.edu.cn/g /etc/apt/sources.list \
    && sed -i s/security.ubuntu.com/mirrors.ustc.edu.cn/g /etc/apt/sources.list \
    && apt update && apt install build-essential libssl-dev ca-certificates -y

WORKDIR /svc
COPY target/release/rockserver /svc/rockserver
COPY config.yaml /svc/config.yaml
ENTRYPOINT ["sh", "-c", "LOG_LEVEL=info ./rockserver"]
