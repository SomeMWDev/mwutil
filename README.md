# mwutil

A CLI tool which helps managing MediaWiki development environments.

> [!NOTE]
> This tool is specifically built to work with [mw-dev-kit](https://github.com/SomeMWDev/mw-dev-kit) and will not work
> in other development environments.

## Versions

There are currently two versions of the tool: The old python version in [/py](/py), and the rust rewrite in [/rs](/rs).

While the rust rewrite is currently still missing some modules, it is already stable and offers the following advantages:
* Improved execution performance (some commands are up to 100x faster)
* Improved autocompletion (e.g. for maintenance scripts) that is way more responsive due to the performance improvements
* Improved error handling
* Bugfixes
* New/improved features
