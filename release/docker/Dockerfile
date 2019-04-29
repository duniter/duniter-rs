# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM registry.duniter.org/docker/duniter-rs-ci:v1.34.0 as build

LABEL maintainer="elois <elois@ifee.fr>"
LABEL version="0.1.1"
LABEL description="Durs server (Divende Universel RuSt)"

# copy source tree
COPY ./ ./

# build durs-server in release with features
RUN cargo build --release --manifest-path bin/durs-server/Cargo.toml --features ssl

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM debian:jessie-slim

# install needed shared librairies 
RUN apt-get update && \
   apt-get install -y ca-certificates libssl-dev && \
   apt-get clean && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

RUN useradd -s /bin/sh -u 1000 -U user

RUN mkdir -p /home/user/.config /var/lib/durs && chown -R user:user /home/user /var/lib/durs

# copy the build artifact from the build stage
COPY --from=build --chown=user:user /target/release/durs /usr/bin/

VOLUME /var/lib/durs

USER user
WORKDIR /home/user

CMD ["durs", "start"]

#run whith `docker run -it IMAGE`
