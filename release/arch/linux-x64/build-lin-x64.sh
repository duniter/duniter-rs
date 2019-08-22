#!/bin/bash

# -------------
# Check params
# -------------

if [[ -z "${1}" ]]; then
	echo "Fatal: no version given to build script"
	exit 1
fi

# -----------
# Constants
# -----------

DURS_TAG="v${1}"
DURS_DEB_VER=" ${1}"

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
	  "arch": "x64"
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
	cargo deb --manifest-path "${ROOT}/bin/dunitrust-${1}/Cargo.toml" --output "${BIN}/dunitrust-server-${1}-${DURS_TAG}-linux-x64.deb"
	create_desc "${BIN}/dunitrust-${1}-${DURS_TAG}-linux-x64.deb" "${1}" "Linux (Ubuntu/Debian)"
}

# ------------
# BEGIn SCRIPT
# ------------

# Prepare
mkdir -p "${RELEASES}" "${BIN}" || exit 1
#rustup add target ${TARGET} || exit 1

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
cp "${ROOT}/target/release/dunitrust" "${RELEASES}/server_/" || exit 1
#cp "${ROOT}/target/release/dunitrust" "${RELEASES}/desktop_" || exit 1

# Copy logo
cp "${ROOT}/images/duniter-rs.png" "${RELEASES}/server_/" || exit 1
#cp "${ROOT}/images/duniter-rs.png" "${RELEASES}/desktop_" || exit 1

# package tar.gz for server variant
cd "${RELEASES}/server_"
tar czf "${BIN}/dunitrust-server-${DURS_TAG}-linux-x64.tar.gz" * || exit 1
create_desc "${BIN}/dunitrust-server-${DURS_TAG}-linux-x64.tar.gz" "Server" "Linux (generic)"
