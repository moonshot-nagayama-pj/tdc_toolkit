# Contributing

Thank you for your interest in contributing to our project. To make it as easy as possible to engage with our work, please read this short guide.

## Getting started with development

Please see [`development-environment.md`](https://github.com/moonshot-nagayama-pj/public-documents/blob/main/engineering/development-environment.md) for more information.

The easiest way to build all code for development, including both Rust and Python, is to run the check script:

```sh
bin/check.bash
```

This will build the code and then run static analysis and unit tests. The same script runs on all pull requests, and must pass before a pull request is accepted.

If you are not using `direnv` as recommended in [our engineering documentation](https://github.com/moonshot-nagayama-pj/public-documents), be sure to activate the virtual environment before using Python commands such as `uv sync` or `maturin develop`:

```
source .venv/bin/activate
```

## Discussing and proposing changes

To make a trivial change, or a change that has already been agreed to in discussions outside of GitHub, please create a pull request.

If a change might need more discussion, please create a GitHub issue in this project before working on a pull request.

## Making pull requests

Before making a pull request, please be sure to run the script `bin/check.bash`. This will run our static analysis checks and unit tests, all of which must pass before a pull request will be accepted.

Ensure that pull requests have meaningful titles. The title may be used in the changelog.

## Licensing and attribution

By contributing your work to this repository, you agree to license it in perpetuity under the terms described in [`LICENSE.md`](LICENSE.md). You are also asserting that the work is your own, and that you have the right to license it to us.

If you wish to integrate our work into your own projects, please follow the attribution guidelines in `LICENSE.md`.

## Code of conduct

In order to provide a safe and welcoming environment for all people to contribute to our project, we have adopted a code of conduct, which you can read in [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).

## Making a release

Releasing this project requires permission to run workflows in the GitHub Actions environment `release`.

1. The release script requires [the cargo-edit extension](https://github.com/killercup/cargo-edit); be sure to install this in your development environment first.
1. From an up-to-date `main` branch, run the `./bin/release.bash` script.
    - If you wish to release using the version number currently in `Cargo.toml`, provide no arguments.
    - If you wish to modify the version number, specify what kind of version bump to make -- `major`, `minor`, or `patch` -- as an argument to this script.
1. Wait for the release check script in GitHub Actions to finish.
1. Edit and publish the draft in [Releases](https://github.com/moonshot-nagayama-pj/tdc_toolkit/releases) (the generate release note function is sufficient for most cases). This will also register a new version at [Zenodo](https://zenodo.org/).
1. Approve the publish action.
1. Create a PR to merge this branch (the script should open a browser tab to create a new PR automatically).
1. Wait for PR approval, merge the PR, and finish!

The release script is loosely based on steps from [the Pallets release procedure](https://palletsprojects.com/contributing/release) and [PnPQ](https://github.com/moonshot-nagayama-pj/PnPQ).
