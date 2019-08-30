# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM registry.duniter.org/docker/dunitrust/dunitrust-ci-lin64:latest as build

LABEL maintainer="elois <elois@dunitrust.org>"
LABEL version="0.1.3"
LABEL description="Dunitrust server (Divende Universel Rust)"

# copy source tree
COPY ./ ./

# build dunitrust-server in release with features
RUN cargo build --release --manifest-path bin/dunitrust-server/Cargo.toml --features ssl

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM debian:jessie-slim

# install needed shared librairies 
RUN apt-get update && \
   apt-get install -y ca-certificates libssl-dev && \
   apt-get clean && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

RUN useradd -s /bin/sh -u 1000 -U user

RUN mkdir -p /home/user/.config /var/lib/dunitrust && chown -R user:user /home/user /var/lib/dunitrust

# copy the build artifact from the build stage
COPY --from=build --chown=user:user /target/release/dunitrust /usr/bin/

VOLUME /var/lib/dunitrust

USER user
WORKDIR /home/user

CMD ["dunitrust", "start"]

#run whith `docker run -it IMAGE`
