# Remote

Remote is a powerful tool to mess up with other people's computers at informatics stages in Volterra.

# Installation

To install `remote` just run the following script!

```
#!/bin/bash
git clone https://github.com/bortoz/remote.git
cd remote
cargo build --release
cd target/release
./remote run echo Remote is awesome!
```

# Usage

`remote` allows you to execute a command on all Volterra computers in parallel!

Remote has 5 subcommand available:

### Run

Simple run a command.
```
remote run [commands]...
```

### Firefox

Open firefox in a specific webpage.
```
remote firefox [url]
```

### Send

Load a file.
```
remote send [file]
```
This is very useful when you want to run some executable.

### Recv

Download a file.
```
remote recv [file]
```

### Like

Put likes on forum.olinfo.it to a specific user.
```
remote like [user]
```

## Options

You can specific which target to mess up.
```
remote --target [target] [command] [args]...
remote  -t      [target] [command] [args]...
```

# Useful commands

Here are some useful and dangerous commands.
```
remote firefox https://upload.wikimedia.org/wikipedia/commons/9/9f/Gennady_Korotkevich.jpg
remote firefox https://www.youtube.com/watch?v=ZZ5LpwO-An4
remote firefox https://www.youtube.com/watch?v=G1IbRujko-A
remote firefox https://www.youtube.com/watch?v=6Dh-RL__uN4
remote run killall firefox
remote run "gnome-terminal --full-screen -- cmatrix -b -C cyan &"
remote run "git clone https://github.com/klange/nyancat; cd nyancat && make && cd src && gnome-terminal --full-screen -- ./nyancat"
```
