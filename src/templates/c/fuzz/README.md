# Fuzzing Setup for {{project_name}}

## Targets

- `{{target_name}}`: Simple fuzz target

## Building

```bash
cd fuzz
./build.sh
```

## Running Locally

```bash
./bin/{{target_name}} ./testsuite/{{target_name}}/
```

## Using Mayhem

```bash
mayhem run
```
