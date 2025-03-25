# `node_modules` cleaner

Node Modules Cleaner (`nmc`) is a utility tool designed to find and remove `node_modules` directories from your projects efficiently. This helps to free up disk space and reduce clutter in your development environment.

## Installation

```bash
cargo install nmc
```

## Usage

The `nmc` command is used to search for and remove `node_modules` directories within your projects. You can customize its behavior using different flags:

- **Default Behavior**:  
  Running `nmc` without any additional flags will search for `node_modules` folders at a default depth of 2 and delete them.

  ```bash
  nmc
  ```

- **Custom Depth**:  
  Use the `-d` or `--depth` option to specify a different search depth. For example, to search at a depth of 3:

  ```bash
  nmc -d 3
  ```

- **Interactive Mode**:  
  Enable interactive mode with the `-i` or `--interactive` flag. This mode lists the found projects and allows you to manually select which ones you want to clean.

  ```bash
  nmc -i
  ```

- **Silent Mode**:  
  Run the cleaner in silent mode using the `-s` or `--silent` flag to suppress output messages during the deletion process.

  ```bash
  nmc -s
  ```

- **Combining Options**:  
  You can combine flags to suit your needs. For instance, to search at a depth of 3 in interactive mode:

  ```bash
  nmc -d 3 -i
  ```

## Options

- `-d, --depth <depth>`: Specifies the depth of the search for projects (default is 2).
- `-i, --interactive`: Enables interactive mode to manually select projects.
- `-s, --silent`: Runs the cleaner in silent mode without output.

## License

This project is licensed under the MIT License.

## Contributing

Contributions are welcome! Feel free to fork the repository and submit a pull request.


