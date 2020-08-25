# Note: We don't use Alpine and its packaged Rust/Cargo because they're too often out of date,
# preventing them from being used to build Substrate/Polkadot/Plug.

FROM phusion/baseimage:0.11 as builder
LABEL description="Pl^g build image: The Pl^g binary is built here."

ENV DEBIAN_FRONTEND=noninteractive

ARG PROFILE=release
WORKDIR /plug

COPY . /plug

RUN apt-get update && \
	apt-get dist-upgrade -y -o Dpkg::Options::="--force-confold" && \
	apt-get install -y \
	clang \
	cmake \
	git \
	libssl-dev \
	pkg-config

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
	export PATH="$PATH:$HOME/.cargo/bin" && \
	.plug/bootstrap && \
	cargo build "--$PROFILE"

# ===== SECOND STAGE ======

FROM phusion/baseimage:0.11
LABEL description="Pl^g runner image: A minimal image for running Pl^g."
ARG PROFILE=release

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	useradd -m -u 1000 -U -s /bin/sh -d /plug plug && \
	mkdir -p /plug/.local/share/plug && \
	chown -R plug:plug /plug/.local && \
	ln -s /plug/.local/share/plug /data

COPY --from=builder /plug/target/$PROFILE/plug /usr/local/bin
COPY --from=builder /plug/target/$PROFILE/subkey /usr/local/bin
COPY --from=builder /plug/target/$PROFILE/node-rpc-client /usr/local/bin
COPY --from=builder /plug/target/$PROFILE/chain-spec-builder /usr/local/bin

# checks
RUN ldd /usr/local/bin/plug && \
	/usr/local/bin/plug --version

# Shrinking
RUN rm -rf /usr/lib/python* && \
	rm -rf /usr/bin /usr/sbin /usr/share/man

USER plug
EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/plug"]
