# FalconMC Contribution Guide

Thank you for your interest in FalconMC. We look forward to collaborating with you.

This guide will provide an overview of our development guidelines and contribution workflow.

### New Contributors

This document assumes an understanding of Git and common development tools. Some resources if you need to brush up:

- [Finding ways to contribute to open source on GitHub](https://docs.github.com/en/get-started/exploring-projects-on-github/finding-ways-to-contribute-to-open-source-on-github)
- [Set up Git](https://docs.github.com/en/get-started/quickstart/set-up-git)
- [GitHub flow](https://docs.github.com/en/get-started/quickstart/github-flow)
  - We require all branches to follow the format described in the [Branch format](#branch-format) section.
- [Collaborating with pull requests](https://docs.github.com/en/github/collaborating-with-pull-requests)

Use an editor that supports [`rust-analyzer`](https://rust-analyzer.github.io). If you want to debug the procedural macros
it might be useful to also install [`cargo-expand`](https://github.com/dtolnay/cargo-expand)

## Getting Started

An overview of project architecture can be found in the [wiki](https://wiki.falconmc.org/) on the [architecture page](https://wiki.falconmc.org/falconmc/developer/architecture.html).
There is currently a lot of stuff in work in progress so going through the rustdocs in the code
could help a lot. If there is something that is poorly explained, do not hesitate to ask for clarification!

### Issues

#### Creating a new issue

When you encounter a problem with FalconMC, [search if an issue already exists](https://docs.github.com/en/github/searching-for-information-on-github/searching-on-github/searching-issues-and-pull-requests#search-by-the-title-body-or-comments).
If a related issue exists, please comment any information about your experience which you believe may be useful to solve the issue.
If a related issue does not exist, please open a new issue using the relevant [issue format](https://github.com/FalconMC-Dev/FalconMC/issues/new/choose).

#### Solving an existing issue

You can find an issues that interest you by searching our [existing issues](https://github.com/FalconMC-Dev/FalconMC/issues).
Don't be discouraged by an issue if it is assigned to someone; you are always welcome to contribute.
If you have any questions about the status of an issue, add a comment or ask on our [Discord](https://discord.com/invite/HC82fwYXW5).

### Making changes

#### Forking the repository

- Using GitHub Desktop:
  - [Getting started with GitHub Desktop](https://docs.github.com/en/desktop/installing-and-configuring-github-desktop/getting-started-with-github-desktop)
     will guide you through setting up Desktop.
  - Once Desktop is set up, you can use it to
    [fork the repo](https://docs.github.com/en/desktop/contributing-and-collaborating-using-github-desktop/cloning-and-forking-repositories-from-github-desktop)!

- Using the command line:
  - [Fork the repo](https://docs.github.com/en/github/getting-started-with-github/fork-a-repo#fork-an-example-repository)
    so that you can make your changes without affecting the original project until you're ready to merge them.

#### Branches

##### Branch format

We require all branches to follow a semantic naming scheme. Your branch can have any name
while you are developing on your fork, but to submit a pull request, we require it abide by the following format:

- **Features/Enhancements**: `feature/<short feature description>`
- **Bug Fixes**: `bugfix/<short bug description>`

For most changes, one of these two categories will make the most sense.
If you find that your changes don't quite fall in line with either,
feel free to choose a suitable prefix or ask on the discord about it.

#### Commiting your changes

In order to fully utilize Git, we recommend frequently committing your changes.
Commit messages are an art we don't expect you to have mastered, but we encourage
developers to use the *present tense* and the *imperative mood* in commit messages.

#### Creating a pull request

**TODO : CREATE A PULL REQUEST TEMPLATE**

Once you have finished your changes, it is time to create a pull request (aka. PR).

1. Using Github Desktop or the Github website, create a pull request from your forked branch into FalconMC's `develop` branch.
2. If your PR resolves any issues, be sure to [link them to the PR](https://docs.github.com/en/issues/tracking-your-work-with-issues/linking-a-pull-request-to-an-issue).
3. Enable the checkbox to [allow maintainer edits](https://docs.github.com/en/github/collaborating-with-issues-and-pull-requests/allowing-changes-to-a-pull-request-branch-created-from-a-fork)
    so we can make any necessary changes to your branch.

#### Review Process
Once you have submitted your PR, we will review your changes. We may have questions or request for additional information.
- We may ask for changes to be made before a PR can be merged, either using [suggested changes](https://docs.github.com/en/github/collaborating-with-issues-and-pull-requests/incorporating-feedback-in-your-pull-request) or pull request comments. You can apply suggested changes directly through the UI. You can make any other changes in your fork, then commit them to your branch.
- As you update your PR and apply changes, mark each conversation as [resolved](https://docs.github.com/en/github/collaborating-with-issues-and-pull-requests/commenting-on-a-pull-request#resolving-conversations).
- If you run into any merge issues, checkout this [git tutorial](https://github.com/skills/resolve-merge-conflicts) to help you resolve merge conflicts and other issues.

We sincerely thank you for your contribution to FalconMC.
If you have any questions or concerns about our development cycle or how to contribute,
please ask us on our [Discord server](https://discord.com/invite/HC82fwYXW5).
