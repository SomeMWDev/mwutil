# mwutil

A CLI tool which helps managing MediaWiki development environments.

## Installation

Requirements:
* bash
* [uv](https://docs.astral.sh/uv/)

```sh
cd /path/to/mwutil
uv tool install . -e
```

### Enabling mwutil in a dev environment

```sh
echo "{}" > /path/to/basedir/.mwutil.json
```

### Autocompletion

```sh
uv tool install argcomplete
activate-global-python-argcomplete
echo 'eval "$(register-python-argcomplete mwutil)"' >> ~/.bashrc
source ~/.bashrc
```

## Features

### Database dumps

Dumps will be stored in a subdirectory of the basedir, which can be configured via the `dumpdir` option in the JSON config.
The default is `dumps`.

Create a dump:
```sh
mwutil dump create test
```

Delete a dump:
```sh
mwutil dump delete test
```

Import a dump:
```sh
mwutil dump import test
```
