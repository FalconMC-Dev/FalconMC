# FalconMC Contribution Guide

Thank you for your interest in FalconMC. We look forward to collaborating with you.

This guide will provide an overview of our development guidelines and contribution workflow.

### New Contributors

This guide assumes an understanding of Git and common development tools. Some resources if you need to brush up:

- [Finding ways to contribute to open source on GitHub](https://docs.github.com/en/get-started/exploring-projects-on-github/finding-ways-to-contribute-to-open-source-on-github)
- [Set up Git](https://docs.github.com/en/get-started/quickstart/set-up-git)
- [GitHub flow](https://docs.github.com/en/get-started/quickstart/github-flow)
  - We require all branches to follow the format described in the [Branch format](#branch-format) section.
- [Collaborating with pull requests](https://docs.github.com/en/github/collaborating-with-pull-requests)

We recommend using [Visual Studio Code](https://code.visualstudio.com/), but any editor supporting rust-analyzer do just fine. If you do use VSCode, be sure to install the appropriate extensions:

- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
  - We recommend using the pre-release version, unless you run in to problems. (Procedural macro tooling is still in active development and frequently has [ABI breakages](https://github.com/rust-lang/rust-analyzer/issues/12525))
- **(RECOMMENDED)** [Even Better TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml)
- **(OPTIONAL)** [Rust Macro Expand](https://marketplace.visualstudio.com/items?itemName=Odiriuss.rust-macro-expand)
  - Requires [cargo-expand](https://github.com/dtolnay/cargo-expand) (`cargo install cargo-expand`)
  - Requires Rust nightly (`rustup toolchain install nightly`)
  
## Getting Started

An overview of project architecture can be found in the [wiki](https://wiki.falconmc.org/), on the [architecture wiki page](https://wiki.falconmc.org/falconmc/developer/architecture.html).

### Issues

#### Create a new issue

If you find a problem with FalconMC, [search if an issue already exists](https://docs.github.com/en/github/searching-for-information-on-github/searching-on-github/searching-issues-and-pull-requests#search-by-the-title-body-or-comments).
If a related issue exists, please comment any information about your experience which you believe may be useful for solving the issue.
If a related issue does not exist, we welcome you to open a new issue using the relevant [issue format](https://github.com/FalconMC-Dev/FalconMC/issues/new/choose).

#### Solve an existing issue

Find an issue that interests you by searching our [existing issues](https://github.com/FalconMC-Dev/FalconMC/issues). Don't be discouraged by an issue if it is assigned to someone; you are always welcome to contribute. If you have any questions about the status of an issue, add a comment or ask on our [Discord](https://discord.com/invite/HC82fwYXW5).

### Making changes

#### Forking the repository

- Using GitHub Desktop:
  - [Getting started with GitHub Desktop](https://docs.github.com/en/desktop/installing-and-configuring-github-desktop/getting-started-with-github-desktop) will guide you through setting up Desktop.
  - Once Desktop is set up, you can use it to [fork the repo](https://docs.github.com/en/desktop/contributing-and-collaborating-using-github-desktop/cloning-and-forking-repositories-from-github-desktop)!

- Using the command line:
  - [Fork the repo](https://docs.github.com/en/github/getting-started-with-github/fork-a-repo#fork-an-example-repository) so that you can make your changes without affecting the original project until you're ready to merge them.

#### Using branches

##### Branch format

We require all branches to follow a semantic naming scheme. Your branch can have any name while you are developing on your fork, but to submit a pull request, we require it abide by the following format:

- **Features/Enhancements**: `feature/<short feature description>`
- **Bug Fixes**: `bugfix/<short bug description>`

For most changes, one of these two categories will make the most sense. If you find that your changes don't quite fall in line with either, feel free to prefix your branch how you see fit. 

##### Creating a branch

- Using GitHub Desktop:
  - The [Managing branches](https://docs.github.com/en/desktop/contributing-and-collaborating-using-github-desktop/making-changes-in-a-branch/managing-branches) page is a good resource for learning branches on GitHub Desktop.
  
- Using the command line:
  - `git branch <new-branch> [base-branch]`

#### Commiting your changes

In order to fully utilize Git, we recommend frequently committing your changes. Commit messages are an art we don't expect you to have mastered, but we do encourage you to follow [Atom's contribution guide](https://github.com/atom/atom/blob/master/CONTRIBUTING.md#git-commit-messages) when writing them.

#### Creating a pull request

**TODO: CREATE A PULL REQUEST TEMPLATE**

Once you have finished your changes, it is time to create a pull request (aka. PR).

1. Using Github Desktop or the Github website, create a pull request from your forked branch into FalconMC's `develop` branch.
2. If your PR resolves any issues, be sure to [link them to the PR](https://docs.github.com/en/issues/tracking-your-work-with-issues/linking-a-pull-request-to-an-issue).
3. Enable the checkbox to [allow maintainer edits](https://docs.github.com/en/github/collaborating-with-issues-and-pull-requests/allowing-changes-to-a-pull-request-branch-created-from-a-fork) so we can make any necessary changes to your branch.

#### Review Process
Once you have submitted your PR, we will review your changes. We may have questions or request for additional information.
- We may ask for changes to be made before a PR can be merged, either using [suggested changes](https://docs.github.com/en/github/collaborating-with-issues-and-pull-requests/incorporating-feedback-in-your-pull-request) or pull request comments. You can apply suggested changes directly through the UI. You can make any other changes in your fork, then commit them to your branch.
- As you update your PR and apply changes, mark each conversation as [resolved](https://docs.github.com/en/github/collaborating-with-issues-and-pull-requests/commenting-on-a-pull-request#resolving-conversations).
- If you run into any merge issues, checkout this [git tutorial](https://github.com/skills/resolve-merge-conflicts) to help you resolve merge conflicts and other issues.

#### Profit???

We sincerely thank you for your contribution to FalconMC. If you have any questions or concerns about our development cycle or how to contribute, please ask us on our [Discord server](https://discord.com/invite/HC82fwYXW5).

### Strategies for Handling Deprecated or Modified Features Across Software Versions:

* Discontinued - The feature is no longer available after a specific version.
    * Is it possible to emulate?
      Yes - (anwser)
      No - (anwser)
  * Introduced in Version X - The feature is only accessible in versions starting from X.
    * Is it possible to emulate?
* Limited Availability (Versions X-Y) - The feature is only usable in a specific range of versions, excluding the latest and earliest supported versions.
* Functionality/Data Changes: The feature has undergone modifications, but some shared functionality or data may still be present.
  * Does changes affect the represented data (eg. is it naming or serialization change)
    * Yes
      * Can functionality be emulated?
        * Yes - (anwser)
        * No - (anwser)
    * No  - (anwser)

#### Indicating not supported features

If users encounter an unsupported feature it shall be handled in such a way to not introduce client desynchronization, indicate to the user that it's not possible to do that, and be as not disturbing as possible (for example, kicking a player if he uses block from newer version shall not be the case)

#### Emulation Possible?

While the feature itself is absent, alternative methods or workarounds may achieve similar results in newer versions.

For features requiring emulation, prioritize implementation in the standard API first. Subsequently, consider offering an optional version-specific API (if applicable) and, if necessary, build the emulated feature on top of or in accordance with the existing API supporting implementation for the native feature.

#### Unified data structure

This multi-versioning approach leverages a flexible data structure capable of representing data from various versions. It acts as an abstracted universal data-type, converting into and out of version-specific formats. If an unsupported version is encountered, it shall use `Result::Err` or `Option::None`. This ensures seamless access to historical data, eliminating the need for managing separate structures for each version. This approach could also abstraction layer for other types. On top of it there might be implemented api that returns values (indicating it's not supported feature but not breaking anything).
