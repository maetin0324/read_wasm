FROM rust:latest
SHELL ["/bin/bash", "-l", "-c"]
RUN apt-get update -y
RUN apt-get upgrade -y
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
   #  iproute2 \
   #  iputils-ping \
    clang \
 && apt-get -y clean \
 && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /home/vscode/Git/ && mkdir -p /home/vscode/ucx_ex/
WORKDIR /home/vscode/Git/
RUN apt-get update -y
# RUN apt install git vim -y
RUN git clone --recursive  https://github.com/openucx/ucx.git
RUN apt-get install autoconf automake libtool -y
RUN apt install build-essential -y
RUN apt install valgrind -y
RUN mkdir install-debug
WORKDIR  /home/vscode/Git/ucx/
RUN ./autogen.sh
RUN ./contrib/configure-devel --prefix=$PWD/install-debug
RUN make -j4
RUN make install
WORKDIR /home/vscode/ucx_ex
COPY . .
