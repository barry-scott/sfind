#!/bin/bash
set -e
RPM_VERSION="$(grep '^version = ' Cargo.toml)"
RPM_VERSION="${RPM_VERSION#version = \"}"
RPM_VERSION="${RPM_VERSION%\"}"

function usage {
    colour-print "<>error Usage: <> $0 copr-release|copr-testing|mock [x86_64|aarch64] [<fedora-release>]"
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

case "$3" in
rawhide)
    VERSION_ID=$3
    ;;

[0-9][0-9])
    VERSION_ID=$3
    ;;
"")
    . /etc/os-release
    ;;
*)
    usage
    exit 1
    ;;
esac

rm -rf tmp
mkdir -p tmp/{SPECS,SOURCES,tmpdir}

colour-print "<>info Info:<> make source"
git archive main --format=tar --prefix=sfind-${RPM_VERSION}/ --output=tmp/SOURCES/sfind-${RPM_VERSION}.crate
cd tmp/SOURCES
colour-print "<>info Info:<> make specfile"
# rust2rpm fails on one host becuase of some unknown issue with TMPDIR
# using an empty directory works around the failure
TMPDIR=$PWD/tmp/tmpdir rust2rpm --path ./sfind-${RPM_VERSION}.crate
mv *.spec ../SPECS
cd ../..

# use host's arch for srpm
MOCK_SRPM_ROOT=fedora-${VERSION_ID}-$(arch)
# use user's arch for rpm
MOCK_RPM_ROOT=fedora-${VERSION_ID}-${ARCH}

colour-print "<>info Info:<> build SRPM"
ls -l tmp
mock \
    --buildsrpm \
    --root ${MOCK_SRPM_ROOT} \
    --spec tmp/SPECS/rust-sfind.spec \
    --sources tmp/SOURCES

colour-print "<>info Info:<> copy SRPM"
ls -l /var/lib/mock/${MOCK_SRPM_ROOT}/result
cp -v /var/lib/mock/${MOCK_SRPM_ROOT}/result/rust-sfind-${RPM_VERSION}-*.src.rpm tmp

SRPM=tmp/rust-sfind-${RPM_VERSION}-*.src.rpm

case "$CMD" in
copr)
    colour-print "<>info Info:<> copr build <>em $COPR_REPO<> of <>em ${SRPM}<> for <>em $ARCH<>"
    set -x
    copr-cli build -r ${MOCK_RPM_ROOT} ${COPR_REPO} ${SRPM}
    ;;

mock)
    colour-print "<>info Info:<> build RPM for <>em $ARCH<>"
    set -x
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
