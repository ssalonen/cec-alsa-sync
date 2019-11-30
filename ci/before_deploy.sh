# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    cross rustc --bin cec-alsa-sync --target $TARGET --release -- -C lto

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz $src/target/$TARGET/release
    sha256sum $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz > $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz.sha256
    cd $src

    rm -rf $stage
}

main
