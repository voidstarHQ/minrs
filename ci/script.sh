# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cross build --target $TARGET
    cross build --target $TARGET --release

    if [ -n $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET
    cross test --target $TARGET --release

    cross run --target $TARGET --bin bw_2d
    cross run --target $TARGET --bin bw_2d --release
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ] || [ -z $APPVEYOR_REPO_TAG_NAME ]; then
    main
fi
