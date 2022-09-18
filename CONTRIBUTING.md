<!--
SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
SPDX-License-Identifier: Apache-2.0
-->

# Community and Contribution Guidelines

First and foremost, thank you for your interest in the Fern project,
we are so happy to have you!

Being an ambitious project, full potential can't be reached without a
supporting community. **No contribution is too small, and all
contributions are valued.**

We are at an early stage of the Fern project, and our _Community and
Contribution Guidelines_ are still work in progress, especially when it comes
to code contributions for bugs, new features, architecture changes, etc.

Nonetheless, we have a [Discord Community](https://discord.gg/A9QSke2Vum).
You are welcome to drop in and ask questions, discuss bugs, feature requests, or
really any concern not covered in these guidelines. Please join us, and say hi!


## Code of Conduct

The Fern project is committed to provide a friendly, safe, and welcoming
environment for all contributors and participants. For _minimum_ expected
behavior please refer to our [Code of Conduct](CODE_OF_CONDUCT.md).


## Licensing

The Fern project requires all source code contributions to be made under the
[Apache License, Version 2.0](LICENSE).


## Coding Style

To ensure code styling consistency across the Fern project:

* Rust code should match the output of `rustfmt` and pass `make style`.


## Developer Certificate of Origin (DCO)

### Signed-Off and Signed Commits

The Fern project requires all source code contributions to be accompanied by a
[Developer Certificate of Origin](https://developercertificate.org) sign-off,
and strongly encourages [signing commits](https://git-scm.com/book/en/Git-Tools-Signing-Your-Work).

The sign-off is a simple line at the end of a commit message, an affirmation
that the source code being submitted was wrote by the contributor, or otherwise
that the contributor has permission to submit the source code as open source.

Using Git, add `-s` for DCO sign-off, and add `-S` for commit signature:
```shell
git commit -s -S -m 'Signed and signed-off commit'
```

Adding `-s` for DCO sign-off will add a _Signed-off-by_ line to the commit
message, for example:

    Signed-off-by: Piotr PAWLICKI <piotrek@seovya.net>

Adding `-S` for commit signature will not add any line to the commit message,
because the commit signature is handled as a commit metadata directly.

Please note that for both sign-off and commit signature using your legal name
is mandatory. DCO is an almost friction-less often used alternative to a
[Contributor License Agreement](https://en.wikipedia.org/wiki/Contributor_License_Agreement).
Having proper Intellectual Property protection is an essential element for
an open source project like Fern, and thus unfortunately handles, pseudonyms,
or anonymous contributions cannot be accepted.


### Fixing DCO Sign-Off

If your contribution fails the DCO check, chances are that it is the entire commit
history of the Pull Request which needs a fix. In such case, best practice is to
[squash](https://gitready.com/advanced/2009/02/10/squashing-commits-with-rebase.html)
the commit history into a single commit, then append the DCO sign-off as described above,
and finally [force push](https://git-scm.com/docs/git-push#Documentation/git-push.txt---no-force-with-lease).

For example, with 2 commits to fix in the PR history:
```shell
git rebase -i HEAD^^
(proceed with interactive squash, amend with DCO sign-off)
git push origin --force-with-lease
```

Please note that generally speaking rewriting a commit history like this is a
hindrance to the review process, and should only be done to correct DCO issues.


## Conventional Commits

A good commit message should describe what changed, where, and why. By adopting
[Conventional Commits, Version 1.0.0](https://www.conventionalcommits.org/),
we believe those expectations can be easily met.

A commit message should therefore be structured as follows:

    <type>(scope): <description>
    [optional <BLANK LINE> if body]
    [optional body]
    <BLANK LINE>
    [optional and required footer(s)]


### Commit Subject Line

For the Fern proxy repository, the following _types_ are defined:

* `fix`: patch a bug in the codebase. This correlates with `PATCH` in
  [Semantic Versioning](#semantic-versioning).
* `feat`: introduce a new feature to the codebase. This correlates with `MINOR`
  in [Semantic Versioning](#semantic-versioning).
* `refactor`: change code in a way that neither fixes a bug nor introduces a
  feature. If refactoring does introduce a **BREAKING CHANGE**, this correlates
  with `MAJOR` in [Semantic Versioning](#semantic-versioning).
* `docs`: change existing documentation (ex: source code description, examples).
* `perf`: performance improvement, therefore ideally backed with a benchmark.
* `chore`: change resulting from a change by a 3rd party (ex: dependency update).
* `style`: change not affecting operations performed by the code (ex: variable
  naming, formatting, reordering code blocks).
* `test`: add a missing test, improve or deduplicate existing ones.
* `ci`: change Continuous Integration configuration files and scripts.

From an end-user perspective, `docs`, `perf`, `chore`, `style`, `test`, and `ci`
types of changes should _never_ introduce a BREAKING CHANGE, and thus _at best_
only lead to a `PATCH` release in [Semantic Versioning](#semantic-versioning).

A commit subject line will be preferably 50 characters or less, and in any case
no more than 72 characters. Having such limit prevents commit subject lines to
be truncated or overflow when displayed in various tools.

Last but not least, the _description_ will start with a verb in lowercase,
present-tense imperative-mood. This style allows to read your _description_ as:

    If applied, this commit will <description>.

To illustrate this convention, here are some examples of commit subject lines:

    docs(proxy): add module-level description for `Pipe`
    feat(masking): add ability to enable transformation per `DataRow`
    feat(proxy): add ability to configure data masking per `DataRow`
    chore(proxy): bump `Config` dependency from `0.13` to `1`
    refactor(interfaces)!: make masking agnostic to PostgreSQL structs
    perf(proxy): avoid memory allocation for `srv_addr` in `Listener`

Example scopes, depending on type of change: `proxy`, `interfaces`, `masking`,
`encryption`, `tokenization`, `postgresql`, `s3`, `circleci`, `github`, ...

**Note:** if you are stuggling to write a concise yet meaningful commit
subject line, you might be committing too many changes at once. Strive for
[atomic commits](https://www.aleksandrhovhannisyan.com/blog/atomic-git-commits/).


### Commit Message Body

With length limitations and other style constraints, a commit subject line
might not be enough to express in details what your commit is about.
No worries, that's exactly what commit message bodies are for!

Similarly to commit subject lines, commit messages bodies use present-tense
imperative-mood and wrap lines at 72 columns (exception for long URLs). They
can have several paragraphs to express all required details in depth.

If your patch fixes an open issue, you can add a reference to it **after**
the message body, i.e. as a _footer_. Use the `Fixes: #` prefix and the issue
number. For other references use the `Refs: #` prefix.

To illustrate this convention, here is a sample complete commit message:

    docs(contributing): illustrate commit message expectations

    Body of commit message is a few lines of text, explaining things
    in more detail, possibly giving some background about the issue
    being fixed, context for a new feature, etc.

    The body of the commit message can be several paragraphs, and
    please do proper word-wrap and keep columns shorter than about
    72 characters or so. That way, `git log` will show things
    nicely even when it is indented.

    Fixes: #1337
    Refs: #421, #51
    Signed-off-by: Piotr PAWLICKI <piotrek@seovya.net>

Well formatted commit messages make the review process so much more enjoyable.
Fellow Fern project contributors will be grateful, so will be your future-self!


## Semantic Versioning

The Fern project wants to have a meaningful and easy to understand versioning
policy. As such, it abides to [Semantic Versioning, Version 2.0.0](https://semver.org/):

    MAJOR.MINOR.PATCH

Given a version number `MAJOR.MINOR.PATCH`:

* A `PATCH` release should only contain bug fixes and/or documentation changes.
  In all cases changes in a `PATCH` release are backward compatible for end-users.
* A `MINOR` release may contain new features, minor dependency updates,
  depreciation warnings, internal implementations changes that are NON-BREAKING.
  In all cases changes in a `MINOR` release are backward compatible for end-users.
* A `MAJOR` release differs from the previous two as only a `MAJOR` release may
  introduce a BREAKING CHANGE, _without_ backward compatibility for end-users.

This policy might however be a bit relaxed until we reach 1.0.0, especially in
the very first iterations where adding core features and associated abilities
to configure them require some experimentation. This is fine and aligned with
Semantic Versioning where MAJOR version zero is all about rapid development!

In all cases, breaking changes will always be indicated properly, as defined
in [Conventional Commits](#conventional-commits) specification.
