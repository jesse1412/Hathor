# Contributing
Hathor is written in [Rust](https://www.rust-lang.org/tools/install) and provides various [pre-commit](https://pre-commit.com/) rules which can make the pull request process smoother.

All pull requests must adhere to the currently enabled project linters to be accepted.

## Pre-requisites
It is recommended that before contributing, you install the provided pre-commit hooks. To install pre-commit, you'll need a [Python](https://www.python.org/downloads/) installation. It is recommended that you install pre-commit via [pipx](https://pypa.github.io/pipx/). pipx provides a sanitized Python installation for any Python packages installed using it. Note, you may need to add the pipx binaries folder (`~/. local/bin`) to your [Path environment variable](https://en.wikipedia.org/wiki/PATH_(variable)).

To install pipx

```shell
pip install pipx
```

To install pre-commit

```shell
pipx install pre_commit
```

To use the Hathor pre-commit hooks

```shell
pre-commit install
```

Once installed, the hooks will apply whenever you make a commit locally.
