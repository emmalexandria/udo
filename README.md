<h3 align="center">udo</h3>
<p align="center"><b>sudo without the surplus</b></p>

<img src="./media/demo.gif?raw=true">

udo (*/juː duː/*) is a cross-platform CLI for running commands as another user. In other words, it's an alternative to [sudo](https://github.com/sudo-project/sudo) and [OpenDoas](https://github.com/Duncaen/OpenDoas). It's written in Rust, but unlike most projects this is not entirely for memory safety reasons. Rather, it is to take advantage of the Rust ecosystem's extensive CLI libs.

*If you take interest in udo, please read the [Word of Warning](#a-word-of-warning) section first*

### Features
- Beautiful output with support for icons.
- Human-readable `.toml` config file.
- Support for running as any user.
- Setting and management of environment variables, unsetting unsafe variables.

### Motivation
The primary motivation for udo was for my own usage. I wanted a suid tool which had nice output, easy to understand configuration, and which provided a modern CLI. I decided to release it because I figured I couldn't be the *only* one. doas is nice in that it's far more simple than sudo, but it's still *boring*. I doubt this will be a common sentiment — most people want their suid tool to be boring.

#### Goals
udo does not aim for feature parity with sudo, nor does it aim to be usable in complex or security-critical environments (e.g. servers). 
- Beautiful, configurable output.
- Easily readable and editable formats.
- Secure *enough* for home use.
- Take full advantage of modern Rust crates (e.g. crossterm, serde, clap).
- Sit between `doas` and `sudo` in terms of complexity.

### Installation
udo currently is not packaged for any distro. This creates some difficulty in installation, as programs like sudo or udo need to be owned by root and have the suid permission bit set. Get the binary from the prebuilt releases or using `cargo install udo`, and then run the following as root: 

```
chown root <path/to/udo>
chmod u+s <path/to/udo>
```

You will also need to create a PAM configuration file, usually at `/etc/pam.d/udo`. This file should be owned by root and have mode `644`. This file will differ based on your system, but a valid file for MacOS looks like this:

```
auth       required       pam_opendirectory.so
account    required       pam_opendirectory.so 
```

### Usage
```
A modern replacement for sudo/doas

Usage: udo [OPTIONS] [command]...
       udo <COMMAND>

Commands:
  --install  Creates config, PAM service, and sets correct permissions binary [aliases: -I]
  --shell    Runs a shell as the given user, optionally imitating a login [aliases: -s]
  --config   Manage your udo config [aliases: -C]

Arguments:
  [command]...  The command to run

Options:
  -n, --nocheck      Skips validating the permissions and owner of udo
  -u, --user <user>  [default: root]
  -c, --clear        Clear the login cache
  -h, --help         Print help
  -V, --version      Print version
```

### A Word of Warning
I am not really a Unix developer, nor am I used to writing secure tools. This tool was created for my own use and as a learning exercise. I do have an interest in making it better for general usage, but I don't know where to start. If you use udo, please be aware that it probably ***will*** break, have vulnerabilities, 
fail to execute commands correctly, and more. This will be the case until it reaches `v1.0.0`

I would like udo to be robust enough that it can be the sole `suid` tool installed on some systems *some day* but it is nowhere near that point. In addition, by installing it you are probably increasing your attack surface dramatically. I'm proud of this piece of software, but it is still very much a work in progress. 

