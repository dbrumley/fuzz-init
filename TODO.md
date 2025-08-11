# Todo list

- [ ] Rewrite CLAUDE.md
- [ ] Analyze if we should add a feature to rename a template path depending on
  a flag.
- [x] Add in dev-mode validation to the template itself rather than hardcoding.
- [ ] Rewrite makefile to work similar to cmake with same file convention as
  'fuzz.sh'
- [ ] Rewrite TUTORIAL.md
- [x] Think through 'fuzz-init .' (.) as project name when using --minimal and
  what templating should do.  Resolution: just use names like "fuzz_harness_1"
  instead of {{project_name}}