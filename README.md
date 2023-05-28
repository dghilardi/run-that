# Run That

Utility software to define scripts repositories and run contained scripts.

## Configuration

Files named `.run-that` will be searched and parsed from the root to the current working directory, they need to be *TOML* files with the following structure.

If multiple files are found and the same key are defined in different files, the files nearest to the current working directory overrides the others.

```toml
[buckets.bash-utils]
priority = 100 # Optional
source.type = "Git"
source.url = "git@gitlab.com:bash/bash-utils.git"
source.reference = "1234567"

[buckets.bash-override]
priority = 200 # Optional
source.type = "Git"
source.url = "https://gitlab.com/bash/bash-override.git"
source.reference = "7654321"
```

## Execution

You can run a script by specifying its name inside the repo, and passing the required args.

```sh
run-that run -s simple_script.sh -- some args
```

Run-that will checkout the repository revision in the cache directory (if not already done) and call the specified script.

If multiple scripts with the same name are found in different repositories the one with highest priority will be executed -- if multiple repos have the highest priority the execution will fail.