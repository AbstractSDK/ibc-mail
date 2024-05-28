# Walkthrough

This doc walks you through the key functionality of the IBC Mail application. The basic functionality provided by this contract is the ability to send a message to another account on the same or a different chain. We'll follow the message from creation to dispatch and delivery.

## Sending a message

Sending a message is done by executing the mail client.

```rust
{{ #include ../../packages/ibcmail/src/client/msg.rs:execute_msg }}
```

```rust
{{ #include ../../contracts/client/src/handlers/execute.rs:execute_handler}}
```

We then construct a message and send it to the server.

```rust
{{ #include ../../contracts/client/src/handlers/execute.rs:send_msg }}
```

Server receives the message and routes it.

```rust
{{ #include ../../contracts/server/src/handlers/execute.rs:execute_handler}}
```

### Recipient is Local Account

If the recipient is local the server sends the message to the mail client on the recipient Account.

```rust
{{ #include ../../contracts/server/src/handlers/execute.rs:set_acc_and_send }}
```

### Recipient is Remote Account

If the recipient is a remote account the server routes the message to a server on other chain based on the configured message route.

```rust
{{ #include ../../contracts/server/src/handlers/execute.rs:ibc_client }}
```

### Remote Server

If the message is routed to a remote server it will be propagated to the remote server through the ibc-client.

The message will then be executed by the ibc-host on the remote chain. The IBC host will call the module IBC endpoint on the remote server.

```rust
{{ #include ../../contracts/server/src/handlers/module_ibc.rs:module_ibc_handler }}
```

Here the message is either dispatched further over IBC or it is locally executed on a mail client.

## Client Receives Message

```rust
{{ #include ../../contracts/client/src/handlers/execute.rs:receive_msg}}
```
