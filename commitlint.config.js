module.exports = {
    extends: ['gitmoji'],
    rules: {
        'type-enum': [
            2,
            'always',
            // declare emoji type names by copying them from commitizen-emoji
            [
                "codestyle", "perf", "prune", "bugfix", "hotfix", "feature", "docs", "deploy", "ui", "init", "tests", "security", "tags", "lint", "wip", "fixci", "downgrade", "upgrade", "depver", "ci", "analytics", "refactor", "depadd", "deprm", "config", "devscripts", "i18n", "typo", "flaky", "revert", "merge", "binary", "contract", "relocate", "license", "breaking", "assets", "a11y", "comment", "gibberish", "text", "db", "addlogs", "rmlogs", "contrib", "ux", "arch", "responsive", "mock", "joke", "gitignore", "snapshots", "poc", "seo", "types", "seed", "flags", "detect", "animation", "deprecate", "auth", "fix", "explore", "clean", "fall"
            ]
        ],
    },
};