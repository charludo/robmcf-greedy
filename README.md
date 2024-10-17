<p align="center">
  <h2 align="center">🤖 robmcf-greedy</h2>
</p>

<p align="center">
	a greedy solver for finding robust minimum cost flows in (rail) networks
</p>

&nbsp;

### 📦 Installation

Two installation methods exist.
First, the project includes a `flake.nix` file which can be used to create the exact environment in which this project was developed.
Using this method is recommended, and guarantees you will be able to run and compile all parts of the project.
Alternatively, you can manually set up the dependencies.

&nbsp;
#### Nix Flake

In case you are not familiar with [nix](https://nixos.org/guides/how-nix-works/) or [nix flakes](https://nixos.wiki/wiki/Flakes), the easiest way to get started is to use the [Determinate Systems Nix installer](https://github.com/DeterminateSystems/nix-installer), e.g. by executing
```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```
in a terminal inside any Linux system (note: macOS and the Windows subsystem for Linux are also supported by the installer, but this project has not been tested on these systems and no support can be provided).
This will not only install Nix, but also configure it for use with flakes out-of-the-box.

Alternatively, your distribution's package manager likely has the `nix` package available, though you will need to activate flake support manually.

Once you have installed Nix, simply enter this project's directory inside your terminal and execute
```bash
nix develop
```

If you want to uninstall the project and its dependencies, simply delete this project's folder, then run
```bash
/nix/nix-installer uninstall
```

After a short while (only required during the first use), the development environment used for this project will be active in your terminal.
You can continue to the "Configuration" section.

&nbsp;

#### Manual Setup

While nothing is preventing you from manually setting up all dependencies, this has not been tested and no support can be provided.
At a minimum, you will need to install [rust and cargo](https://www.rust-lang.org/tools/install) and Gurobi.

&nbsp;

### ⚙️ Configuration

Gurobi requires a license key.
If you are using the nix flake, this file is expected to live at `$HOME/.config/gurobi/gurobi.lic`.

In case you want to change this, or have used the manual setup, you need to change the value of the `GRB_LICENSE_FILE` environment variable to point to your license file.

No further configuration is required.

&nbsp;

### 🚀 Usage (CLI)

The project comes with two interfaces, a `cli` package and a `gui` package.
The `cli` is the main way to use this project, and the only way to interact with the solver library itself (apart from writing your own interface - the solver is a normal `rust` library, after all).

To use the `cli`, execute
```bash
cargo run -p cli
```
from within the project's folder.
You will be greeted by the following help message:
```help
CLI for the Greedy RobMCF solver library

Usage: cli [OPTIONS] <COMMAND>

Commands:
  solve      Solve RobMCF greedily for the given network
  ilp        Attempt to solve the entire network via an ILP. No
             greedy involvement
  benchmark  Benchmark the solution process. Should use "None" or
             "Greedy" for the remainder function
  random     Create a completely random network instead of using an
             input file
  latex      Export the network vertices and arcs as a latex figure
  help       Print this message or the help of the given
             subcommand(s)

Options:
  -v, --debug            Enable [v]erbose debug logging
  -t, --trace            Enable [t]race logging
  -q, --quiet            Disable logging, [q]uieting output. Takes
                         precedence over debug
  -e, --export <EXPORT>  [E]xport all resulting measures, appending
                         them to the given file
  -h, --help             Print help

Solver Parameters:
  -c, --costs <COSTS>
          Function used to calculate the [c]ost of the overall
          solution [default: max] [possible values: max, mean,
          median]
  -d, --delta <DELTA>
          [D]istance function used in determining intermediate arc
          sets [default: logarithmic-medium] [possible values:
          linear-mini, linear-low, linear-medium, linear-high,
          logarithmic-mini, logarithmic-low, logarithmic-medium,
          logarithmic-high, unlimited]
  -r, --draw <DRAW>
          Function used to calculate the [r]elative draw of supply
          towards fixerd arcs [default: peer-pressure] [possible
          values: none, linear, linear-non-neg, quadratic,
          quadratic-non-neg, cubic, cubic-non-neg, exponential,
          exponential-non-neg, peer-pressure]
  -s, --slack <SLACK>
          Function used in determining the total [s]lack available
          to scenarios [default: none] [possible values: none,
          const-10, const-100, const-1000, unlimited, balance-min,
          difference-to-max, difference-to-max-plus-min,
          difference-to-max-plus-10, difference-to-max-plus-100,
          difference-to-max-plus-1000]
  -m, --remainder <REMAINDER>
          [M]ethod by which a solution for routing supply which
          cannot use fixed arcs is found [default: none] [possible
          values: none, greedy, gurobi]
```
Discussing all options available in the `cli` is out of scope for this README, please instead use the provided help messages.
Note that further help is available for the subcommands, for example try executing
```bash
cargo run -p cli solve --help
```
to see all available options for solving networks which are already saved to disk as JSON.

Using `cargo run` executes the project in debug mode.
To build a (faster) release version, execute
```bash
cargo build -p cli --release
```
and find the result at `target/release/cli`.

Example networks can be found in the `examples/` folder, or generated yourself using the `random` subcommand.

&nbsp;

### 🚀 Usage (CLI)

The `gui` is used primarily to visually debug networks.
It shows vertices and arcs, and indicates fixed arcs through color, capacity through arc width, and the arc load through opacity.

To access it, run
```bash
cargo run -p gui
```
from within the project's folder.
Note that you will have to adjust the network file to open by editing `gui/src/main.rs`; as I said, it's a debugging tool 🙂

&nbsp;

### 📚 Documentation

Comments have been added wherever they are useful in helping understand *why* an action was taken.
In accordance with [best practices](https://mitcommlab.mit.edu/broad/commkit/coding-and-comment-style/) however, comments detailing *what* is being done have not been added, apart from a select few instances where this was not obvious from the context.

It can be helpful to build the documentation for the library:
```bash
cargo doc --document-private-items
```
or even for the `cli`:
```bash
cargo doc -p cli
```
The resulting HTML file can be opened in your browser.

&nbsp;
