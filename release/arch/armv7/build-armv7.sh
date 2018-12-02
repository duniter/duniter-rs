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
	cp "${ROOT}/release/extra/systemd/durs.service" "${1}/lib/systemd/system" || exit 1
}

# Debian package building.
# -
# Parameters:
# 1. Building type (either “desktop” or “server”).
# 2. Debian package name.
build_deb_pack() {
	rm -rf "${RELEASES}/durs-armv7"
	mkdir "${RELEASES}/durs-armv7" || exit 1
	cp -r "${ROOT}/release/extra/debian/package/"* "${RELEASES}/durs-armv7" || exit 1
	build_extra_${1} "${RELEASES}/durs-armv7"
	mkdir -p "${RELEASES}/durs-armv7/opt/durs/" || exit 1
	chmod 755 "${RELEASES}/durs-armv7/DEBIAN/"post* || exit 1
	chmod 755 "${RELEASES}/durs-armv7/DEBIAN/"pre* || exit 1
	sed -i "s/Version:.*/Version:${DURS_DEB_VER}/g" "${RELEASES}/durs-armv7/DEBIAN/control" || exit 1

	cd "${RELEASES}/${1}_/"
	zip -qr "${RELEASES}/durs-armv7/opt/durs/durs.zip" * || exit 1

	sed -i "s/Package: .*/Package: ${2}/g" "${RELEASES}/durs-armv7/DEBIAN/control" || exit 1

	cd "${RELEASES}"
	dpkg-deb --build durs-armv7 || exit 1
	mv durs-armv7.deb "${BIN}/duniter-rust-${1}-${DURS_TAG}-armv7.deb" || exit 1
	create_desc "${BIN}/duniter-rust-${1}-${DURS_TAG}-armv7.deb" "${1}" "Linux (Ubuntu/Debian/Raspbian)"
}

# -----------
# Prepare
# -----------

DURS_TAG="v${1}"
DURS_DEB_VER=" ${1}"
TARGET="armv7-unknown-linux-gnueabihf"

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
mkdir -p "${RELEASES}/durs" || exit 1
cp -r $(find "${ROOT}" -mindepth 1 -maxdepth 1 ! -name "${WORK_NAME}") "${RELEASES}/durs" || exit 1
cd "${RELEASES}/durs"
rm -Rf .gitignore .git || exit 1 # Remove git files

# Build binary
echo ">> Building binary..."
cd "${ROOT}/bin/durs-server"
cargo build --release --features=ssl --target=${TARGET} || exit 1

mkdir -p "${RELEASES}/server_" || exit 1
cp "${ROOT}/target/${TARGET}/release/durs" "${RELEASES}/server_/" || exit 1
#cp "${ROOT}/target/release/durs" "${RELEASES}/desktop_" || exit 1

# Copy logo
cp "${ROOT}/images/duniter-rs.png" "${RELEASES}/server_/" || exit 1
#cp "${ROOT}/images/duniter-rs.png" "${RELEASES}/desktop_" || exit 1


# ---------------
# Build .tar.gz
# ---------------

cd "${RELEASES}/server_"
tar czf "${BIN}/duniter-rust-server-${DURS_TAG}-armv7.tar.gz" * || exit 1
create_desc "${BIN}/duniter-rust-server-${DURS_TAG}-armv7.tar.gz" "Server" "Linux (generic)"

# -----------------------
# Build Debian packages
# -----------------------

build_deb_pack server durs
#build_deb_pack desktop durs
