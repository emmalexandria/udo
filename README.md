<h3 align="center">udo</h3>
<p align="center"><b>sudo without the surplus</b></p>

udo (*/juː duː/*) is a cross-platform cli for running commands as another user. In other words, it's an alternative to [sudo](https://github.com/sudo-project/sudo) and 
[OpenDoas](https://github.com/Duncaen/OpenDoas). It's written in Rust, but unlike most projects this is not entirely for memory safety reasons. Rather, it is to take advantage 
of the Rust ecosystems extensive CLI libs.

### Motivation
The primary motivation for udo was for my own usage. I wanted a suid tool which had nice output, easy to understand configuration, and which provided a modern CLI.
I decided to release it because I figured I couldn't be the *only* one. doas is nice in that it's far more simple than sudo, but it's still *boring*. I doubt this 
will be a common sentiment — most people want their suid tool to be boring.

#### Goals
udo does not aim for feature parity with sudo, nor does it aim to be usable in complex or security-critical environments (e.g. servers). 
- Beautiful, configurable output.
- Easily readable and editable formats.
- Secure *enough* for home use.
- Take full advantage of modern Rust crates (e.g. crossterm, serde, clap)

### Installation
udo currently is not packaged for any distro. This creates some difficulty in installation, as programs like sudo or udo need to be owned by root and have the suid 
permission bit set. Get the binary from the prebuilt releases or using `cargo install udo`, and then run the following as root: 

```
chown root <path/to/udo>
chmod u+s <path/to/udo>
```

You will also need to create a PAM configuration file, usually at `/etc/pam.d/udo`. This file should be owned by root and have mode `644`. This file will differ 
based on your system, but a valid file for MacOS looks like this:

```
auth       required       pam_opendirectory.so
account    required       pam_opendirectory.so 
```

