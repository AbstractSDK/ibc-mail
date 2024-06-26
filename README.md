# IBC Mail

This project is a simple mail application that demonstrates the use of the IBC module in Abstract. The application allows users to send and receive messages to and from other users on different chains.
It supports:
- Sending messages to other users on the same chain.
- Sending messages to users on other chains one hop away

> You can get an overview of the code and how it works by following [this code overview](https://abstractsdk.github.io/ibc-mail/).

It's designed with two contracts, the Client and the Server. Every user has their own mail Client, which can send and receive messages. It routes these messages to the Server, which then forwards them to the recipient's Client. This is beneficial for a few reasons:
- Multiplexing between different versions of clients. Users will likely want their own version of the client, which can be updated independently of the server. The server will be able to support sending messages to multiple clients.
- When sending multi-hop messages, the mail Server will send the message to the recipient's chain's mail Server, which will then forward it to the recipient's Client. If there is a hop in between, it will just hop between servers. The big question is "which client should the message be sent to? for routing?"

See [this document](https://www.notion.so/abstract-money/IBC-Mail-744feaac39cb412ba8b5b4147cf8fb32?pvs=4) for more information.

**Desired Features**
- [x] Send / receive messages to users on the same chain
- [x] Send / receive messages to users on other chains
- [x] Send messages to users on other chains with hops in between
- [ ] Batch messages in the server (request the server to send certain messages by ids, and then the server will reach out to the clients to send the messages. This allows for batching messages to save on gas costs)
- [ ] messages to groups
- [ ] Sending to namespaces OR
- [ ] Sending to "namespace@remote.local"
- [ ] Sending funds in messages (attachments)
- [ ] Sending NFTs in messages (attachments)
- [ ] Contacts contract
- [ ] Support for multiple versions of the client
- [ ] (frontend) Encrypting messages with the recipient's public key
- [ ] Idea: usernames for users, so that they can be identified by their username instead of their address. These would be preferable over using the namespaces, though the issue is that the Server is an adapter and state can't be shared between diferent adapter versions.


## Using the Justfile

This repository comes with a [`justfile`](https://github.com/casey/just), which is a handy task runner that helps with building, testing, and publishing your Abstract app module.

### Installing Tools

To fully make use of the `justfile`, you need to install a few tools first. You can do this by simply running `just install-tools`. See [tools used the template](https://docs.abstract.money/3_get_started/2_installation.html?#tools-used-in-the-template) for more information.

### Available Tasks

Here are some of the tasks available in the `justfile`:

- `install-tools`: Install all the tools needed to run the tasks.
- `build`: Build everything with all features.
- `test`: Run all tests.
- `watch-test`: Watch the codebase and run tests on changes.
- `fmt`: Format the codebase (including .toml).
- `lint`: Lint-check the codebase.
- `lintfix`: Fix linting errors automatically.
- `watch`: Watch the codebase and run `cargo check` on changes.
- `check`: Check the codebase for issues.
- `publish {{chain-id}}`: Publish the App to a network.
- `wasm`: Optimize the contract.
- `schema`: Generate the json schemas for the contract
<!-- - `ts-codegen`: Generate the typescript client code for the contract -->
<!-- - `ts-publish`: Publish the typescript client code to npm -->
- `publish-schemas`: Publish the schemas by creating a PR on the Abstract [schemas](https://github.com/AbstractSDK/schemas) repository.

You can see the full list of tasks available by running `just --list`.

### Compiling

Best to run `cargo update` to have synced versions just in case.

You can compile your module by running the following command:
```sh
just wasm
```
This should result in an artifacts directory being created in your project root. Inside you will find a `my_module.wasm` file that is your module’s binary.

### Testing

You can test the module using the different provided methods.

1. **Integration testing:** We provide an integration testing setup [here](./tests/integration.rs). You should use this to set up your environment and test the different execution and query entry-points of your module. Once you are satisfied with the results you can try publishing it to a real chain.
2. **Local Daemon:** Once you have confirmed that your module works as expected you can spin up a local node and deploy Abstract + your app onto the chain. You need [Docker](https://www.docker.com/) installed for this step. You can do this by running the [test-local](./examples/test-local.rs) example, which uses a locally running juno daemon to deploy to. You can setup local juno using `just juno-local` command. At this point you can also test your front-end with the contracts.

Once testing is done you can attempt an actual deployment on test and mainnet.

### Publishing

Before attempting to publish your app you need to add your mnemonic to the `.env` file. **Don't use a mnemonic that has mainnet funds for this.**

<!-- It's also assumed that you have an account and module namespace claimed with this account before publishing. You can read how to do that [here](https://docs.abstract.money/4_get_started/5_abstract_client.html). -->
Select from a wide range of [supported chains](https://orchestrator.abstract.money/chains/index.html) before proceeding. Make sure you've some balance enough to pay gas for the transaction. If the chain does not have gas, complete at least 1 transaction with your account before proceeding.

You can now use `just publish {{chain-id}}` to run the [`examples/publish.rs`](./examples/publish.rs) script. The script will publish the app to the networks that you provided. Make sure you have enough funds in your wallet on the different networks you aim to publish on.

### Publishing Module Schemas

To publish your module schemas, we provide the `publish-schemas` command, which creates a pull request on the Abstract [schemas](https://github.com/AbstractSDK/schemas) repository.

Please install [github cli](https://cli.github.com/) before proceeding. Also login and setup your github auth by `gh auth login`. Now, we're ready to proceed.

```bash
just publish-schemas <namespace> <name> <version>
```

- `namespace`: Your module's namespace
- `name`: Your module's name
- `version`: Your module's version. Note that if you only include the minor version (e.g., `0.1`), you don't have to reupload the schemas for every patch version.

The command will automatically clone the Abstract Schemas repository, create a new branch with the given namespace, name, and version, and copy the schemas and metadata from your module to the appropriate directory.

For this command to work properly, please make sure that your `metadata.json` file is located at the root of your module's directory. This file is necessary for the Abstract Frontend to correctly interpret and display information about your module.

Example:

```bash
just publish-schemas ibcmail my-module 0.0.1
```

In the example above, `ibcmail` is the namespace, `my-module` is the module's name, and `0.1` is the minor version. If you create a patch for your module (e.g., `0.1.1`), you don't need to run `publish-schemas` again unless the schemas have changed.

## Contributing

We welcome contributions to the Abstract App Module Template! To contribute, fork this repository and submit a pull request with your changes. If you have any questions or issues, please open an issue in the repository and we will be happy to assist you.

## Community
Check out the following places for support, discussions & feedback:

- Join our [Discord server](https://discord.com/invite/uch3Tq3aym)
