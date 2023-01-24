<p align="center"><img style="width: 150px;" src="https://cdn.justjs.dev/assets/svg/maid.svg"></p>
<h1 align="center">Maid</h1>

Maid is a task runner / build tool that aims to be simpler and easier to use than, for example, GNU Make.
Tasks are stored in a file called `maidfile` using the TOML syntax.

<img style="width: 1100px;" src="https://cdn.justjs.dev/assets/maid_screenshot.png">

### Quick Start
See the [installation](#installation) section for how to install just on your computer. Try running maid --version to make sure that it's installed correctly.

Once maid is installed and working, create a file named maidfile in the root of your project with the following contents:

```toml
[tasks.hello]
path = ""
info = "this is a comment"
script = "echo 'hello world'"
```

Running maid with no arguments shows a list of tasks in the maidfile:

```bash
$ maid
? Select a task to run:
> hello: this is a comment
[↑↓ to move, enter to select, type to filter]
```
### Installation
Pre-built binaries for Linux, MacOS, and Windows can be found on the [releases](https://github.com/exact-labs/maid/releases) page.

#### Building
- Clone the project `git clone https://github.com/exact-labs/maid.git`
- Open a terminal in the project folder
- Check if you have cargo (Rust's package manager) installed, just type in `cargo`
- If cargo is installed, run `cargo build --release`
- Put the executable into one of your PATH entries
  - Linux: usually /bin/ or /usr/bin/
  - Windows: C:\Windows\System32 is good for it but don't use windows
