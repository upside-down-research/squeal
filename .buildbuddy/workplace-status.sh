#!/bin/sh

# This script will be run bazel when building process starts to
# generate key-value information that represents the status of the
# workspace. The output should be like
#
# KEY1 VALUE1
# KEY2 VALUE2
#
# If the script exits with non-zero code, it's considered as a failure
# and the output will be discarded.

set -e

### output for image builds.

echo "CONTAINER_REPO ${CONTAINER_REPO}"
echo "CONTAINER_TAG ${CONTAINER_TAG}"

### -- buildbuddy status info below

git config --global --add safe.directory /workspace

remove_url_credentials() {
  if command -v perl >/dev/null 2>&1; then
    perl -pe 's#//.*?:.*?@#//#'
  else
    cat
  fi
}

repo_url=$(git config --get remote.origin.url | remove_url_credentials)
echo "REPO_URL $repo_url"

commit_sha=$(git rev-parse HEAD)
echo "COMMIT_SHA $commit_sha"

git_branch=$(git rev-parse --abbrev-ref HEAD)
echo "GIT_BRANCH $git_branch"

git_tree_status=$(git diff-index --quiet HEAD -- && echo 'Clean' || echo 'Modified')
echo "GIT_TREE_STATUS $git_tree_status"

# Note: the "STABLE_" suffix causes these to be part of the "stable" workspace
# status, which may trigger rebuilds of certain targets if these values change
# and you're building with the "--stamp" flag.
latest_version_tag=$(./.buildbuddy/latest_version_tag.sh)
echo "STABLE_VERSION_TAG $latest_version_tag"
echo "STABLE_COMMIT_SHA $commit_sha"
