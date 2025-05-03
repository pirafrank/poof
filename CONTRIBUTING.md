# Contributing to poof

First off, thank you for considering contributing to poof! It's people like you that make poof such a great tool.

Following these guidelines helps to communicate that you respect the time of the developers managing and developing this open source project. In return, they should reciprocate that respect in addressing your issue, assessing changes, and helping you finalize your pull requests.

## Code of Conduct

This project and everyone participating in it is governed by the [poof Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## How Can I Contribute?

### Reporting Bugs

This section guides you through submitting a bug report for poof. Following these guidelines helps maintainers and the community understand your report, reproduce the behavior, and find related reports.

Before creating bug reports, please check existing issues as you might find out that you don't need to create one. When you are creating a bug report, please include as many details as possible. Fill out the required template if one is provided, the information it asks for helps us resolve issues faster.

> **Note:** If you find a **Closed** issue that seems like it is the same thing that you're experiencing, open a new issue and include a link to the original issue in the body of your new one.

#### Before Submitting A Bug Report

* **Check the documentation** for a list of common questions and problems.
* **Check the issue tracker** for any related issues that might be similar to the one you're experiencing.
* **Check the changelog** to see if the issue has been addressed in a recent release.
* **Update poof** to the latest version to ensure the issue hasn't already been fixed.

#### How Do I Submit A (Good) Bug Report?

Bugs are tracked as GitHub issues. Create an issue on the repository and provide the following information.

Mandatory information:

* **Use a clear and descriptive title** for the issue to identify the problem.
* **Describe the exact steps which reproduce the problem** in as many details as possible.
* **Describe the behavior you observed after following the steps** and point out what exactly is the problem with that behavior.
* **Explain which behavior you expected to see instead and why.**
* **Include debug information**, which gets printed via `poof debug`
* **Include a stack trace (if available)**, possibly with `RUST_BACKTRACE=full` set.
* **If the problem wasn't triggered by a specific action**, describe what you were doing before the problem happened.

Optional, but **highly recommended**:

* **Include screenshots and animated GIFs**, if possible, which show you following the described steps and clearly demonstrate the problem.

### Suggesting Enhancements

This section guides you through submitting an enhancement suggestion for poof, including completely new features and minor improvements to existing functionality.

Before creating enhancement suggestions, please check the issue tracker as you might find out that you don't need to create one. When you are creating an enhancement suggestion, please include as many details as possible.

#### Before Submitting An Enhancement Suggestion

* **Check the documentation** for information on existing features.
* **Perform a search** on the issue tracker to see if the enhancement has already been suggested. If it has, add a comment to the existing issue instead of opening a new one.

#### How Do I Submit A (Good) Enhancement Suggestion?

Enhancement suggestions are tracked as GitHub issues. Create an issue on the repository and provide the following information:

* **Use a clear and descriptive title** for the issue to identify the suggestion.
* **Provide a step-by-step description of the suggested enhancement** in as many details as possible.
* **Provide specific examples to demonstrate the steps**.
* **Describe the current behavior** and **explain which behavior you expected to see instead** and why.
* **Include screenshots and animated GIFs** which help demonstrate the suggestion.
* **Explain why this enhancement would be useful** to most poof users.
* **Specify which version of poof you're using.**
* **Specify debug info, and the name and version of your OS.**. Debug info can be obtained by running `poof debug`.

### Your First Code Contribution

Unsure where to begin contributing to *poof*? You can start by looking through issues tagged `good first issue` or `help wanted`.

### Pull Requests

Please follow these steps to have your contribution considered by the maintainers:

1. Follow the [style guides](#styleguides).
2. Ensure your code is tested.
3. Update documentation if you are changing functionality.
4. After you submit your pull request, verify that all status checks (like CI builds and tests) are passing.

While the prerequisites above must be satisfied prior to having your pull request reviewed, the reviewer(s) may ask you to complete additional design work, tests, or other changes before your pull request can be ultimately accepted.

## Styleguides

### Git Commit Messages

* Follow [git conventinal commits](https://www.conventionalcommits.org/en/v1.0.0/) message style.
* Use the present tense ("Add feature" not "Added feature").
* Use the imperative mood ("Move cursor to..." not "Moves cursor to...").
* Limit the first line to 72 characters or less.
* Reference issues and pull requests liberally after the first line (e.g., "Closes #123").

### Code Styleguide

* Follow the established coding style in the project.
* Use meaningful variable and function names.
* Write clear and concise comments where necessary.
* Use `just better` to format and lint code before submitting a PR. In an effort to improve the development workflow, git hooks are provided in the `hooks` dir. You may want to configure them to automatically performs some additional checks. To do so, run `just install-hooks` in the root of the repository.
* Use `just test` to run tests and ensure your code is working as expected.
* Avoid committing lines that are commented out or unused code.
* Avoid committing edits that are not related to the issue being worked on and you want to submit a PR to.
* Edits that cause a change in the code style or formatting should be avoided unless they are part of the issue being worked on and you want to submit a PR to.
* Edits causing breaking changes should be discussed with the maintainers before being submitted as a PR to minimize disruption to the users.
* No edits should be made to the `README.md` or other documentation files unless you are submitting a PR to update the documentation.
* PRs for documentation updates should reference the corresponding PR for the code changes, if applicable, yet they should be separate PRs to ensure that the documentation is always up-to-date with stable code changes.
* If you are submitting a PR to update the documentation, please follow the guidelines below.

### Documentation Styleguide

* Keep documentation clear, concise, and up-to-date.
* Use Markdown for documentation files.
* Use headings, lists, and code blocks to organize content.
* Use proper grammar and spelling.
* Use consistent formatting and style throughout the documentation.
* Emoji are welcome in documentation files, but use them sparingly and only when they add value to the content.
* Use code blocks for code snippets and examples.
* Use links to reference related issues, pull requests, and external resources.
* You may use images to better illustrate concepts or examples when necessary. The `.assets` folder is the right place to store them.

## Issue and Pull Request Labels

We use labels to help manage issues and pull requests. Below is a list of labels we use and their meanings split per category.

| Label              | Category     | Description                                                       |
| :----------------- | :----------- | :---------------------------------------------------------------- |
| `bug`              | triage       | A confirmed bug report.                                           |
| `security`.        | triage       | Potential vulnerability or security-related issue. |
| `enhancement`      | triage       | Suggestion of an improvement of an existing feature or functionality. |
| `feature request`  | triage       | Suggestion of a new feature or functionality.                     |
| `documentation`    | triage       | Improvements or additions to documentation.                       |
| `help wanted`      | triage       | Community contribution is welcome.                                |
| `good first issue` | triage       | Suitable for new contributors.                                    |
| `how to`           | triage       | Question about how to do something more than reporting an issue.  |
| `wontfix`          | triage       | The issue will not be addressed.                                  |
| `invalid`          | triage       | The issue is not valid or relevant.                               |
| `duplicate`        | triage       | The issue is a duplicate of another issue.                        |
| `in progress`      | wip          | The issue is being worked on.                                     |
| `ready for merge`  | wip          | The PR is ready to be merged.                                     |
| `needs info`       | post-process | More information is needed to proceed.                            |
| `needs review`     | post-process | The PR is ready for review.                                       |
| `needs testing`    | post-process | The PR needs to be tested.                                        |
