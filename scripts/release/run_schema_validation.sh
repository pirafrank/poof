#!/usr/bin/env bash
set -e

cd "$(dirname "${BASH_SOURCE[0]}")"

echo "Running JSON Schema validation..."

python -c '
import json
import jsonschema

with open("./build-environment.schema.json") as schema_file:
    schema = json.load(schema_file)

jsonschema.Draft202012Validator.check_schema(schema)
print("Schema is valid Draft 2020-12")
'
