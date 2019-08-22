#!/bin/bash

# ------------
# Check params
# ------------

if [[ -z "${1}" ]]; then
	echo "Fatal: no version given to build script"
	exit 1
fi

# ---------
# Constants
# ---------

DURS_TAG="v${1}"
DURS_DEB_VER=" ${1}"
TARGET="armv7-unknown-linux-gnueabihf"

ROOT="${PWD}"
WORK_NAME=work
WORK="${ROOT}/${WORK_NAME}"
RELEASES="${WORK}/releases"
BIN="${WORK}/bin"

# ---------
# Functions
# ---------

# Create description.
# -
# Parameters:
# 1. Initial file name.
# 2. Building type (either “desktop” or “server”).
# 3. Category (OS, distribution).
create_desc() {
	cat >"${1}".desc <<-EOF
	{
	  "version": "${DURS_TAG}",
	  "job": "${CI_JOB_ID}",
	  "type": "${2^}",
	  "category": "${3}",
	  "arch": "arm v7"
	}
	EOF
}

# Server specific building phase.
# -
# Parameters:
# 1. Building directory.
build_extra_server() {
	mkdir -p "${1}/lib/systemd/system" || exit 1
	cp "${ROOT}/release/extra/systemd/dunitrust.service" "${1}/lib/systemd/system" || exit 1
}

# Debian package building.
# -
# Parameters:
# 1. Building type (either “desktop” or “server”).
build_deb_pack() {
	#cd "bin/dunitrust-${1}"
	#cargo build --release --target=armv7-unknown-linux-gnueabihf --features=ssl
	cargo deb --manifest-path="bin/dunitrust-${1}/Cargo.toml" --target=${TARGET} --variant=arm --output "${BIN}/dunitrust-${1}-${DURS_TAG}-armv7.deb"
	create_desc "${BIN}/dunitrust-${1}-${DURS_TAG}-armv7.deb" "${1}" "Linux (Ubuntu/Debian/Raspbian)"
}

# ------------
# BEGIN SCRIPT
# ------------

# Prepare
mkdir -p "${RELEASES}" "${BIN}" || exit 1

# Clean up
rm -rf "${BIN}/"*.{deb,tar.gz}{,.desc}

# ---------------------
# Build Debian packages
# ---------------------

build_deb_pack server
#build_deb_pack desktop


# ---------------
# Build .tar.gz
# ---------------

# Create releases directory
mkdir -p "${RELEASES}/server_" || exit 1
#mkdir -p "${RELEASES}/desktop_" || exit 1

# Copy binary (build by cargo deb)
cp "${ROOT}/target/${TARGET}/release/dunitrust" "${RELEASES}/server_/" || exit 1
#cp "${ROOT}/target/release/dunitrust" "${RELEASES}/desktop_" || exit 1

# Copy logo
cp "${ROOT}/images/duniter-rs.png" "${RELEASES}/server_/" || exit 1
#cp "${ROOT}/images/duniter-rs.png" "${RELEASES}/desktop_" || exit 1


# package tar.gz for server variant
cd "${RELEASES}/server_"
tar czf "${BIN}/dunitrust-server-${DURS_TAG}-armv7.tar.gz" * || exit 1
create_desc "${BIN}/dunitrust-server-${DURS_TAG}-armv7.tar.gz" "Server" "Linux (generic)"
