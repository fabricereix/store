#!/bin/bash
set -e

export PATH=target/debug:$PATH
rm -rf target/{packages,installer}

find integration -name "*.ini" | sort | while read -r db_file; do
  options_file=${db_file%.ini}.options
  options="$(xargs <"$options_file")"
  cmd="store --db-file $db_file --packages-dir target/packages --tmp-dir target/installer $options"
  echo "$cmd"
  set +e
  output=$(echo "$cmd"  | sh 2>&1 )
  exit_code=$?
  set -e

  # test ok
  if grep -q "test_ok" <<< "$db_file"; then
    if [[ "$exit_code" != "0" ]]
    then
       echo "Expected exit code 0"
    fi

  # test errors
  else
    expected_exit_code_file="${db_file%.*}.exit"
    expected=$(cat "$expected_exit_code_file")
    if [[ "$exit_code" != "$expected" ]]
    then
       echo "Exit code"
       echo "  actual: $exit_code"
       echo "  expected: $expected"
       exit 1
    fi
  fi

  expected_log_file="${db_file%.*}.log"
  expected=$(envsubst <"$expected_log_file")
  if [[ "$expected" != "$output" ]]
  then
    echo "Error differs"
    diff <( echo "$expected" ) <( echo "$output" )
    exit 1
  fi

done

cmd="store --db-file integration/test_ok/mypackage_build.ini --tmp-dir target/installer --packages-dir target/packages reinstall package-build"
echo "$cmd" | tee | sh

cmd="store --db-file integration/test_ok/mypackage_build.ini --tmp-dir target/installer --packages-dir target/packages uninstall package-build"
echo "$cmd" | tee | sh

cmd="store --db-file integration/test_ok/mypackage_build.ini --tmp-dir target/installer  --packages-dir target/packages info"
echo "$cmd" | tee | sh