## Overview

- We use handelbars as our templating language.

## MacOS Setup

MacOS clang does not come with libfuzzer support, so you will need to install.
For example,

```bash
brew install llvm
echo 'export PATH="$(brew --prefix llvm)/bin:$PATH"' >> ~/.zshrc
```

Make sure `which clang++` does **not** show macos clang (`/usr/bin/clang++`)

## Style guide

- Use hard line wraps configured at 78 characters. We suggest VSCode extension
  Rewrap.
-
