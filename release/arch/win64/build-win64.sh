#!/bin/bash

if [[ -z "${1}" ]]; then
	echo "Fatal: no version given to build script"
	exit 1
fi

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

# -----------
# Prepare
# -----------

DURS_TAG="v${1}"
DURS_VER=" ${1}"
TARGET="x86_64-pc-windows-gnu"

#rustup add target ${TARGET} || exit 1

# -----------
# Folders
# -----------

ROOT="${PWD}"
WORK_NAME=work
WORK="${ROOT}/${WORK_NAME}"
DOWNLOADS="${WORK}/downloads"
RELEASES="${WORK}/releases"
BIN="${WORK}/bin"

mkdir -p "${DOWNLOADS}" "${RELEASES}" "${BIN}" || exit 1
rm -rf "${BIN}/"*.{deb,tar.gz}{,.desc} # Clean up

# -----------
# Downloads
# -----------

cd "${DOWNLOADS}"

# -----------
# Releases
# -----------

# Prepare sources
mkdir -p "${RELEASES}/dunitrust" || exit 1
cp -r $(find "${ROOT}" -mindepth 1 -maxdepth 1 ! -name "${WORK_NAME}") "${RELEASES}/dunitrust" || exit 1
cd "${RELEASES}/dunitrust"
rm -Rf .gitignore .git || exit 1 # Remove git files

# Build binary
echo ">> Building binary..."
cd "${ROOT}/bin/dunitrust-server"
cargo build --release --target=${TARGET} || exit 1

mkdir -p "${RELEASES}/server_" || exit 1
cp "${ROOT}/target/${TARGET}/release/dunitrust.exe" "${RELEASES}/server_/" || exit 1
#cp "${ROOT}/target/${TARGET}/release/dunitrust" "${RELEASES}/desktop_" || exit 1

# Copy logo
#cp "${ROOT}/images/dunitrust.png" "${RELEASES}/server_/" || exit 1
#cp "${ROOT}/images/dunitrust.png" "${RELEASES}/desktop_" || exit 1


# ---------------
# Build .zip
# ---------------

cd "${RELEASES}/server_"
zip "${BIN}/dunitrust-server-${DURS_TAG}-windows-x64.zip" * || exit 1
create_desc "${BIN}/dunitrust-server-${DURS_TAG}-windows-x64.zip" "Server" "Windows"
