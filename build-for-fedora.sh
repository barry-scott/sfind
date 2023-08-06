#!/bin/bash
set -e
RPM_VERSION="$(grep '^version = ' Cargo.toml)"
RPM_VERSION="${RPM_VERSION#version = \"}"
RPM_VERSION="${RPM_VERSION%\"}"

function usage {
    colour-print "<>error Usage: <> $0 copr-release|copr-testing|mock [x86_64|aarch64]"
}

case "$1" in
copr-release)
    CMD=copr
    COPR_REPO=tools
    ;;

copr-testing)
    CMD=copr
    COPR_REPO=tools-testing
    ;;

mock)
    CMD=mock
    ;;
*)
    usage
    exit 1
    ;;
esac
case "$2" in
x86_64)
    ARCH=x86_64
    ;;
aarch64)
    ARCH=aarch64
    ;;
"")
    ARCH=$(arch)
    ;;
*)
    usage
    exit 1
    ;;
esac

rm -rf tmp
mkdir tmp

colour-print "<>info Info:<> make source"
git archive main --format=tar --prefix=sfind-${RPM_VERSION}/ --output=tmp/sfind-${RPM_VERSION}.crate
cd tmp
colour-print "<>info Info:<> make specfile"
rust2rpm ..
cd ..

. /etc/os-release

# use host's arch for srpm
MOCK_SRPM_ROOT=fedora-${VERSION_ID}-$(arch)
# use user's arch for rpm
MOCK_RPM_ROOT=fedora-${VERSION_ID}-${ARCH}

colour-print "<>info Info:<> build SRPM"
ls -l tmp
mock \
    --buildsrpm \
    --root ${MOCK_SRPM_ROOT} \
    --spec tmp/rust-sfind.spec \
    --sources tmp/sfind-${RPM_VERSION}.crate


colour-print "<>info Info:<> copy SRPM"
ls -l /var/lib/mock/${MOCK_SRPM_ROOT}/result
cp -v /var/lib/mock/${MOCK_SRPM_ROOT}/result/rust-sfind-${RPM_VERSION}-*.src.rpm tmp

SRPM=tmp/rust-sfind-${RPM_VERSION}-*.src.rpm

case "$CMD" in
copr)
    set -x
    colour-print "<>info Info:<> copr build <>em $COPR_REPO<> of <>em ${SRPM}<> for <>em $ARCH<>"
    copr-cli build -r ${MOCK_RPM_ROOT} ${COPR_REPO} ${SRPM}
    ;;

mock)
    colour-print "<>info Info:<> build RPM for <>em $ARCH<>"
    mock \
        --rebuild \
        --root ${MOCK_RPM_ROOT} \
            tmp/rust-sfind-${RPM_VERSION}-*.src.rpm
    ls -l /var/lib/mock/${MOCK_RPM_ROOT}/result
    cp -v /var/lib/mock/${MOCK_RPM_ROOT}/result/sfind-${RPM_VERSION}*${ARCH}.rpm tmp
    ;;

*)
    colour-print "<>error Error:<> bad CMD value of <>em $CMD<>"
    ;;
esac
