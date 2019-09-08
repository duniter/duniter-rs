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

# ArchLinux package building.
# -
# Parameters:
# 1. Building type (either “desktop” or “server”).
build_arch_pack() {
	cd "${ROOT}/bin/dunitrust-${1}"
	cargo-arch arch --manifest-path "${ROOT}/bin/dunitrust-${1}"
	mv dunitrust-*.pkg.tar.xz "${BIN}/dunitrust-${1}-${DURS_TAG}-archlinux-x64.pkg.tar.xz"
	cd "${ROOT}"
	create_desc "${BIN}/dunitrust-${1}-${DURS_TAG}-archlinux-x64.pkg.tar.xz" "${1}" "Linux (ArchLinux)"
}

# ------------
# BEGIN SCRIPT
# ------------

# Prepare
mkdir -p "${RELEASES}" "${BIN}" || exit 1
#rustup add target ${TARGET} || exit 1

# Clean up
rm -rf "${BIN}/"*.pkg.tar.xz{,.desc}

# ---------------------
# Build ArchLinux packages
# ---------------------

build_arch_pack server
