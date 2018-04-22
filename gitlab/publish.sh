OIFS=$IFS
IFS='/' read -r first a <<< "$CI_COMMIT_TAG"
cd $first
IFS=$OIFS
cargo login $DUNITER_CRATES_TOKEN
cargo publish